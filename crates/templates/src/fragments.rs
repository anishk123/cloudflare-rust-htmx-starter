use askama::Template;
use starter_contracts::Note;

use crate::models::NoteView;

#[derive(Template)]
#[template(path = "fragments/note_card.html")]
struct NoteCardTemplate<'a> {
    note: NoteView,
    csrf_token: &'a str,
}

#[derive(Template)]
#[template(path = "fragments/error.html")]
struct ErrorFragmentTemplate<'a> {
    message: &'a str,
}

pub fn render_note_card(note: &Note, csrf_token: &str) -> askama::Result<String> {
    NoteCardTemplate {
        note: NoteView::from(note),
        csrf_token,
    }
    .render()
}

pub fn render_error_fragment(message: &str) -> askama::Result<String> {
    ErrorFragmentTemplate { message }.render()
}
