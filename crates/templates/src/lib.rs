mod fragments;
mod models;
mod pages;
mod seo;

pub use fragments::{render_error_fragment, render_note_card};
pub use models::{NoteView, PageMeta};
pub use pages::{render_design_system, render_error_page, render_home, render_public_note};

#[cfg(test)]
mod tests {
    use super::*;
    use starter_contracts::{Note, NoteStatus};
    use uuid::Uuid;

    fn note(status: NoteStatus, title: &str) -> Note {
        Note {
            id: Uuid::nil(),
            title: title.into(),
            body: "Evidence body".into(),
            status,
            version: 1,
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_000_000,
            summary: Some("Bounded summary".into()),
        }
    }

    #[test]
    fn public_note_has_absolute_canonical_description_and_safe_json_ld() {
        let html = render_public_note(
            &note(NoteStatus::Published, "</script><script>alert(1)</script>"),
            "https://example.test",
        )
        .unwrap();

        assert!(html.contains(
            "rel=\"canonical\" href=\"https://example.test/notes/00000000-0000-0000-0000-000000000000/"
        ));
        assert!(html.contains("name=\"description\""));
        assert!(html.contains("type=\"application/ld+json\""));
        assert!(!html.contains("</script><script>alert(1)</script>"));
        assert!(html.contains("2023-11-14T22:13:20Z"));
    }

    #[test]
    fn note_fragment_preserves_ordinary_form_fallbacks() {
        let html = render_note_card(&note(NoteStatus::Draft, "Draft"), "csrf").unwrap();

        assert!(html.contains("method=\"post\""));
        assert!(html.contains("action=\"/notes/00000000-0000-0000-0000-000000000000/summarize\""));
        assert!(html.contains("hx-post=\"/notes/00000000-0000-0000-0000-000000000000/summarize\""));
    }

    #[test]
    fn home_contains_native_and_htmx_form_contracts() {
        let html = render_home(
            &[note(NoteStatus::Draft, "Draft")],
            "csrf",
            "https://example.test",
        )
        .unwrap();
        assert!(html.contains("action=\"/notes\""));
        assert!(html.contains("hx-post=\"/notes\""));
        assert!(html.contains("href=\"/design-system\""));
        assert!(!html.contains("pico"));
    }

    #[test]
    fn home_empty_state_has_a_truly_empty_notes_container() {
        let html = render_home(&[], "csrf", "https://example.test").unwrap();

        assert!(html.contains(
            "<div id=\"notes\" class=\"stack\" data-empty-label=\"No notes yet. Create the first one above.\"></div>"
        ));
    }
}
