use starter_contracts::{Note, NoteStatus};
use starter_domain::slugify;
use starter_shared::request_id;
use uuid::Uuid;
use worker::{Error, Request, Response, Result, RouteContext};

use crate::http::{CachePolicy, not_found, redirect_308, secured, text};

async fn published_note(context: &RouteContext<()>) -> Result<Option<Note>> {
    let Some(id) = context
        .param("id")
        .and_then(|value| Uuid::parse_str(value).ok())
    else {
        return Ok(None);
    };
    let note = starter_database::find_note(&context.d1("DB")?, id).await?;
    Ok(note.filter(|note| note.status == NoteStatus::Published))
}

pub(crate) async fn page(req: Request, context: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let Some(note) = published_note(&context).await? else {
        return not_found(&request_id);
    };
    let canonical_slug = slugify(&note.title);
    if context.param("slug").map(String::as_str) != Some(canonical_slug.as_str()) {
        let path = format!("/notes/{}/{}", note.id, canonical_slug);
        return redirect_308(
            req.url()?.join(&path)?,
            &request_id,
            CachePolicy::PublicShort,
        );
    }
    let origin = req.url()?.origin().ascii_serialization();
    let html = starter_templates::render_public_note(&note, &origin)
        .map_err(|error| Error::RustError(format!("template render failed: {error}")))?;
    secured(
        Response::from_html(html)?,
        &request_id,
        CachePolicy::PublicShort,
    )
}

/// `matchit` cannot register `.json` and `.md` suffix routes at the same
/// position, so this route captures `<uuid>.<extension>` and dispatches here.
pub(crate) async fn data(_req: Request, context: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let Some((id_text, extension)) = context
        .param("id.ext")
        .and_then(|value| value.rsplit_once('.'))
    else {
        return not_found(&request_id);
    };
    let Ok(id) = Uuid::parse_str(id_text) else {
        return not_found(&request_id);
    };
    let Some(note) = starter_database::find_note(&context.d1("DB")?, id)
        .await?
        .filter(|note| note.status == NoteStatus::Published)
    else {
        return not_found(&request_id);
    };
    let response = match extension {
        "json" => Response::from_json(&note)?,
        "md" => text(
            &format!(
                "# {}\n\n{}\n\n{}",
                note.title,
                note.body,
                note.summary
                    .as_ref()
                    .map(|summary| format!("## Summary\n\n{summary}"))
                    .unwrap_or_default()
            ),
            "text/markdown; charset=utf-8",
        )?,
        _ => return not_found(&request_id),
    };
    secured(response, &request_id, CachePolicy::PublicShort)
}
