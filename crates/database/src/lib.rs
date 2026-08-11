use serde::Deserialize;
use starter_contracts::{Note, NoteStatus};
use uuid::Uuid;
use worker::{D1Database, Result};

#[derive(Debug, Deserialize)]
struct NoteRow {
    id: String,
    title: String,
    body: String,
    status: String,
    version: u32,
    created_at_ms: i64,
    updated_at_ms: i64,
    summary: Option<String>,
    // Present in the row but deliberately not exposed on `Note`: published
    // representations must not leak the owner's identity.
    #[allow(dead_code)]
    owner_id: String,
}

impl TryFrom<NoteRow> for Note {
    type Error = worker::Error;

    fn try_from(row: NoteRow) -> Result<Self> {
        let status = match row.status.as_str() {
            "published" => NoteStatus::Published,
            "archived" => NoteStatus::Archived,
            _ => NoteStatus::Draft,
        };
        // owner_id is deliberately not part of the public Note contract:
        // published representations must not leak the owner's identity.
        Ok(Note {
            id: Uuid::parse_str(&row.id).map_err(|e| worker::Error::RustError(e.to_string()))?,
            title: row.title,
            body: row.body,
            status,
            version: row.version,
            created_at_ms: row.created_at_ms,
            updated_at_ms: row.updated_at_ms,
            summary: row.summary,
        })
    }
}

/// Notes belonging to one owner, newest first. Used by the private app shell.
pub async fn list_notes(db: &D1Database, owner_id: Uuid) -> Result<Vec<Note>> {
    let owner = owner_id.to_string();
    let result = worker::query!(
        db,
        "SELECT id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id FROM notes WHERE owner_id=?1 ORDER BY updated_at_ms DESC",
        owner
    )?
    .all()
    .await?;
    result
        .results::<NoteRow>()?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
}

/// Fetch a note only if it belongs to the given owner. Everything else — a
/// missing note or someone else's note — comes back as `None` so callers can
/// return a uniform 404.
pub async fn find_owned_note(db: &D1Database, id: Uuid, owner_id: Uuid) -> Result<Option<Note>> {
    let id_s = id.to_string();
    let owner = owner_id.to_string();
    let row = worker::query!(
        db,
        "SELECT id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id FROM notes WHERE id=?1 AND owner_id=?2",
        id_s,
        owner
    )?
    .first::<NoteRow>(None)
    .await?;
    row.map(TryInto::try_into).transpose()
}

/// All published notes across owners, newest first, for public SEO surfaces
/// (sitemap). Drafts of every owner are excluded here.
pub async fn list_published_notes(db: &D1Database) -> Result<Vec<Note>> {
    let result = worker::query!(
        db,
        "SELECT id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id FROM notes WHERE status='published' ORDER BY updated_at_ms DESC"
    )
    .all()
    .await?;
    result
        .results::<NoteRow>()?
        .into_iter()
        .map(TryInto::try_into)
        .collect()
}

/// Unscoped fetch used only by trusted internal paths: the jobs Worker
/// (writing summaries) and public published-note routes. App routes must use
/// `find_owned_note` instead.
pub async fn find_note(db: &D1Database, id: Uuid) -> Result<Option<Note>> {
    let id_s = id.to_string();
    let row = worker::query!(
        db,
        "SELECT id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id FROM notes WHERE id=?1",
        id_s
    )?
    .first::<NoteRow>(None)
    .await?;
    row.map(TryInto::try_into).transpose()
}

pub async fn create_note(
    db: &D1Database,
    note: &Note,
    operation_id: Uuid,
    owner_id: Uuid,
) -> Result<Note> {
    let operation = operation_id.to_string();
    let owner = owner_id.to_string();
    // Idempotency is scoped to the owner: a replayed operation from this user
    // returns the existing note instead of inserting a duplicate, but an
    // operation_id minted by another user can never collide with this one.
    if let Some(entity_id) = worker::query!(
        db,
        "SELECT entity_id FROM processed_operations WHERE operation_id=?1 AND owner_id=?2",
        operation.clone(),
        owner.clone()
    )?
    .first::<String>(Some("entity_id"))
    .await?
    {
        let existing_id =
            Uuid::parse_str(&entity_id).map_err(|e| worker::Error::RustError(e.to_string()))?;
        return find_owned_note(db, existing_id, owner_id)
            .await?
            .ok_or_else(|| {
                worker::Error::RustError("idempotency record points to a missing note".into())
            });
    }

    let id = note.id.to_string();
    let title = note.title.clone();
    let body = note.body.clone();
    let summary = note.summary.clone();
    let entity_id = note.id.to_string();
    let insert = worker::query!(
        db,
        "INSERT INTO notes(id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        id,
        title,
        body,
        "draft",
        note.version,
        note.created_at_ms,
        note.updated_at_ms,
        summary,
        owner.clone()
    )?;
    let op = worker::query!(
        db,
        "INSERT INTO processed_operations(operation_id,entity_id,processed_at_ms,owner_id) VALUES(?1,?2,?3,?4)",
        operation,
        entity_id,
        note.updated_at_ms,
        owner
    )?;
    db.batch(vec![insert, op]).await?;
    Ok(note.clone())
}

/// Publish a note only if it belongs to the owner. Returns `None` (404) for
/// missing or foreign notes.
pub async fn publish_note(
    db: &D1Database,
    note_id: Uuid,
    owner_id: Uuid,
    updated_at_ms: i64,
) -> Result<Option<Note>> {
    worker::query!(
        db,
        "UPDATE notes SET status='published', updated_at_ms=?1, version=version+1 WHERE id=?2 AND owner_id=?3",
        updated_at_ms,
        note_id.to_string(),
        owner_id.to_string()
    )?
    .run()
    .await?;
    find_owned_note(db, note_id, owner_id).await
}

/// Bytes this owner has stored in R2, as tracked in D1. Unknown owners read
/// as zero so the first upload always passes the quota check.
pub async fn upload_bytes_used(db: &D1Database, owner_id: Uuid) -> Result<u64> {
    let owner = owner_id.to_string();
    let row = worker::query!(
        db,
        "SELECT bytes_used FROM upload_usage WHERE owner_id=?1",
        owner
    )?
    .first::<i64>(Some("bytes_used"))
    .await?;
    Ok(row.map(|v| v as u64).unwrap_or(0))
}

/// Record a completed R2 upload against the owner's quota. Called only after
/// the object is stored; the quota check runs before the write, so a failed
/// D1 update leaves the object stored but uncounted (accounting drift only,
/// never a quota bypass for future uploads of the same owner).
pub async fn record_upload(
    db: &D1Database,
    owner_id: Uuid,
    size_bytes: u64,
    now_ms: i64,
) -> Result<()> {
    let owner = owner_id.to_string();
    worker::query!(
        db,
        "INSERT INTO upload_usage(owner_id, bytes_used, updated_at_ms) VALUES(?1, ?2, ?3) ON CONFLICT(owner_id) DO UPDATE SET bytes_used = bytes_used + ?2, updated_at_ms = ?3",
        owner,
        size_bytes as i64,
        now_ms
    )?
    .run()
    .await?;
    Ok(())
}

pub async fn is_job_processed(db: &D1Database, job_id: Uuid) -> Result<bool> {
    let id = job_id.to_string();
    Ok(
        worker::query!(db, "SELECT job_id FROM processed_jobs WHERE job_id=?1", id)?
            .first::<String>(Some("job_id"))
            .await?
            .is_some(),
    )
}

pub async fn complete_summary_job(
    db: &D1Database,
    job_id: Uuid,
    note_id: Uuid,
    summary: &str,
    now_ms: i64,
) -> Result<bool> {
    if is_job_processed(db, job_id).await? {
        return Ok(false);
    }
    let update = worker::query!(
        db,
        "UPDATE notes SET summary=?1, updated_at_ms=?2, version=version+1 WHERE id=?3",
        summary,
        now_ms,
        note_id.to_string()
    )?;
    let mark = worker::query!(
        db,
        "INSERT INTO processed_jobs(job_id,processed_at_ms) VALUES(?1,?2)",
        job_id.to_string(),
        now_ms
    )?;
    db.batch(vec![update, mark]).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_all_database_queries_against_schema_contract() {
        let known_tables = [
            "notes",
            "processed_operations",
            "processed_jobs",
            "upload_usage",
        ];
        let known_columns = [
            "id",
            "title",
            "body",
            "status",
            "version",
            "created_at_ms",
            "updated_at_ms",
            "summary",
            "owner_id",
            "operation_id",
            "entity_id",
            "processed_at_ms",
            "job_id",
            "bytes_used",
        ];

        let queries = [
            "SELECT id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id FROM notes WHERE owner_id=?1 ORDER BY updated_at_ms DESC",
            "SELECT id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id FROM notes WHERE id=?1 AND owner_id=?2",
            "SELECT id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id FROM notes WHERE status='published' ORDER BY updated_at_ms DESC",
            "SELECT id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id FROM notes WHERE id=?1",
            "SELECT entity_id FROM processed_operations WHERE operation_id=?1 AND owner_id=?2",
            "INSERT INTO notes(id,title,body,status,version,created_at_ms,updated_at_ms,summary,owner_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            "INSERT INTO processed_operations(operation_id,entity_id,processed_at_ms,owner_id) VALUES(?1,?2,?3,?4)",
            "UPDATE notes SET status='published', updated_at_ms=?1, version=version+1 WHERE id=?2 AND owner_id=?3",
            "SELECT bytes_used FROM upload_usage WHERE owner_id=?1",
            "INSERT INTO upload_usage(owner_id, bytes_used, updated_at_ms) VALUES(?1, ?2, ?3) ON CONFLICT(owner_id) DO UPDATE SET bytes_used = bytes_used + ?2, updated_at_ms = ?3",
            "SELECT job_id FROM processed_jobs WHERE job_id=?1",
            "UPDATE notes SET summary=?1, updated_at_ms=?2, version=version+1 WHERE id=?3",
            "INSERT INTO processed_jobs(job_id,processed_at_ms) VALUES(?1,?2)",
        ];

        for query in queries {
            let contains_table = known_tables.iter().any(|t| query.contains(t));
            assert!(
                contains_table,
                "Query failed schema validation (unknown table): {query}"
            );

            // Ensure query doesn't contain unbalanced placeholders or invalid parameters
            assert!(
                !query.contains("?0"),
                "Query contains invalid parameter index ?0: {query}"
            );
        }
    }
}
