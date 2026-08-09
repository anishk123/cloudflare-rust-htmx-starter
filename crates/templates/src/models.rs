use starter_contracts::{Note, NoteStatus};
use starter_domain::slugify;

#[derive(Debug, Clone)]
pub struct PageMeta {
    pub title: String,
    pub description: String,
    pub canonical_url: String,
    pub og_type: &'static str,
    pub json_ld: Option<String>,
}

impl PageMeta {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        canonical_url: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            canonical_url: canonical_url.into(),
            og_type: "website",
            json_ld: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoteView {
    pub id: String,
    pub title: String,
    pub body: String,
    pub summary: Option<String>,
    pub status_label: &'static str,
    pub version: u32,
    pub is_draft: bool,
    pub is_published: bool,
    pub summarize_url: String,
    pub publish_url: String,
    pub public_html_url: String,
    pub markdown_url: String,
    pub json_url: String,
}

impl From<&Note> for NoteView {
    fn from(note: &Note) -> Self {
        let id = note.id.to_string();
        let status_label = match note.status {
            NoteStatus::Draft => "Draft",
            NoteStatus::Published => "Published",
            NoteStatus::Archived => "Archived",
        };
        Self {
            id: id.clone(),
            title: note.title.clone(),
            body: note.body.clone(),
            summary: note.summary.clone(),
            status_label,
            version: note.version,
            is_draft: note.status == NoteStatus::Draft,
            is_published: note.status == NoteStatus::Published,
            summarize_url: format!("/notes/{id}/summarize"),
            publish_url: format!("/notes/{id}/publish"),
            public_html_url: format!("/notes/{id}/{}", slugify(&note.title)),
            markdown_url: format!("/notes/{id}.md"),
            json_url: format!("/notes/{id}.json"),
        }
    }
}
