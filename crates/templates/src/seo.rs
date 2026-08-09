use serde_json::json;
use starter_contracts::Note;
use starter_domain::slugify;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub(crate) fn absolute_url(origin: &str, path: &str) -> String {
    format!("{}{}", origin.trim_end_matches('/'), path)
}

pub(crate) fn description(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = normalized.chars();
    let shortened: String = chars.by_ref().take(155).collect();
    if chars.next().is_some() {
        format!("{}…", shortened.trim_end())
    } else {
        shortened
    }
}

pub(crate) fn rfc3339_from_millis(value: i64) -> String {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value) * 1_000_000)
        .ok()
        .and_then(|date| date.format(&Rfc3339).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".into())
}

pub(crate) fn article_json_ld(note: &Note, origin: &str) -> String {
    let path = format!("/notes/{}/{}", note.id, slugify(&note.title));
    safe_json_for_script(&json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": note.title,
        "description": description(note.summary.as_deref().unwrap_or(&note.body)),
        "articleBody": note.body,
        "datePublished": rfc3339_from_millis(note.created_at_ms),
        "dateModified": rfc3339_from_millis(note.updated_at_ms),
        "mainEntityOfPage": absolute_url(origin, &path),
    }))
}

fn safe_json_for_script(value: &serde_json::Value) -> String {
    value
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn description_is_bounded_on_character_boundaries() {
        let text = "é".repeat(200);
        let value = description(&text);
        assert_eq!(value.chars().count(), 156);
        assert!(value.ends_with('…'));
    }

    #[test]
    fn absolute_url_normalizes_one_separator() {
        assert_eq!(
            absolute_url("https://example.test/", "/notes"),
            "https://example.test/notes"
        );
    }
}
