use askama::Template;
use starter_contracts::Note;
use starter_domain::slugify;

use crate::{
    models::{NoteView, PageMeta},
    seo::{absolute_url, article_json_ld, description},
};

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomeTemplate<'a> {
    meta: PageMeta,
    notes: Vec<NoteView>,
    csrf_token: &'a str,
}

#[derive(Template)]
#[template(path = "pages/public_note.html")]
struct PublicNoteTemplate {
    meta: PageMeta,
    note: NoteView,
}

#[derive(Template)]
#[template(path = "pages/error.html")]
struct ErrorPageTemplate<'a> {
    meta: PageMeta,
    heading: &'a str,
    message: &'a str,
}

#[derive(Template)]
#[template(path = "pages/design_system.html")]
struct DesignSystemTemplate {
    meta: PageMeta,
}

pub fn render_home(notes: &[Note], csrf_token: &str, origin: &str) -> askama::Result<String> {
    HomeTemplate {
        meta: PageMeta::new(
            "Evidence Notes · Rust + HTMX Starter",
            "A production-minded Rust and HTMX starter for fast Cloudflare applications.",
            absolute_url(origin, "/"),
        ),
        notes: notes.iter().map(NoteView::from).collect(),
        csrf_token,
    }
    .render()
}

pub fn render_public_note(note: &Note, origin: &str) -> askama::Result<String> {
    let path = format!("/notes/{}/{}", note.id, slugify(&note.title));
    let mut meta = PageMeta::new(
        note.title.clone(),
        description(note.summary.as_deref().unwrap_or(&note.body)),
        absolute_url(origin, &path),
    );
    meta.og_type = "article";
    meta.json_ld = Some(article_json_ld(note, origin));
    PublicNoteTemplate {
        meta,
        note: NoteView::from(note),
    }
    .render()
}

pub fn render_error_page(heading: &str, message: &str, origin: &str) -> askama::Result<String> {
    ErrorPageTemplate {
        meta: PageMeta::new(heading, message, absolute_url(origin, "/")),
        heading,
        message,
    }
    .render()
}

pub fn render_design_system(origin: &str) -> askama::Result<String> {
    DesignSystemTemplate {
        meta: PageMeta::new(
            "Design system · Rust + HTMX Starter",
            "Quiet Product components and interaction states included with the starter.",
            absolute_url(origin, "/design-system"),
        ),
    }
    .render()
}
