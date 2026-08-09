use maud::{html, Markup, PreEscaped, DOCTYPE};
use starter_contracts::{Note, NoteStatus};
use starter_domain::slugify;

fn safe_json_for_script(value: &serde_json::Value) -> String {
    value
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}

pub fn layout(
    title: &str,
    canonical: &str,
    body: Markup,
    json_ld: Option<&serde_json::Value>,
) -> String {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover";
                meta name="theme-color" content="#111827";
                meta name="color-scheme" content="light dark";
                title { (title) }
                link rel="manifest" href="/manifest.webmanifest";
                link rel="icon" href="/icon.svg" type="image/svg+xml";
                link rel="apple-touch-icon" href="/apple-touch-icon.png";
                link rel="stylesheet" href="/assets/vendor/pico.min.css";
                link rel="stylesheet" href="/assets/app.css";
                link rel="canonical" href=(canonical);
                @if let Some(value) = json_ld {
                    script type="application/ld+json" {
                        (PreEscaped(safe_json_for_script(value)))
                    }
                }
                script src="/assets/vendor/htmx.min.js" defer {}
                script src="/assets/vendor/response-targets.js" defer {}
                script src="/assets/app.js" defer {}
            }
            body
                hx-boost="true"
                hx-ext="response-targets"
                hx-target-error="#app-errors"
            {
                div class="app-shell" {
                    aside class="app-sidebar" aria-label="Application" {
                        a href="/" class="brand" { "Rust + HTMX Starter" }
                        nav aria-label="Primary" {
                            ul {
                                li { a href="/" { "Notes" } }
                                li { a href="/contracts" { "Contracts" } }
                            }
                        }
                        details {
                            summary { "Why this stack?" }
                            small {
                                "Rust/Wasm at the edge, semantic HTML, HTMX interactions, Pico defaults, tiny native JS."
                            }
                        }
                    }
                    header class="app-header" {
                        a href="/" class="brand mobile-only" { "Rust + HTMX Starter" }
                        nav aria-label="Utilities" {
                            ul {
                                li {
                                    button
                                        type="button"
                                        class="secondary outline"
                                        data-install-pwa
                                        hidden
                                    { "Install" }
                                }
                                li {
                                    details class="dropdown" {
                                        summary { "Help" }
                                        ul {
                                            li { a href="/contracts" { "Contracts" } }
                                            li { a href="/llms.txt" { "llms.txt" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    main class="app-main" {
                        div id="app-errors" class="app-errors" role="alert" aria-live="assertive" {}
                        div class="container" { (body) }
                    }
                }
                div class="toast-region" aria-live="polite" {
                    div id="connection-status" class="connection-status" role="status" {}
                    button type="button" class="secondary" data-update-pwa hidden { "Update available · reload" }
                }
                footer class="site-footer" {
                    "Cloudflare Workers · Rust/Wasm · Maud · HTMX · Pico · PWA"
                }
            }
        }
    }
    .into_string()
}

pub fn error_fragment(message: &str) -> Markup {
    html! {
        div class="form-errors" role="alert" {
            strong { "Could not complete that request." }
            p { (message) }
        }
    }
}

pub fn error_page(title: &str, message: &str) -> String {
    let body = html! {
        article class="card" {
            h1 { (title) }
            p { (message) }
            a href="/" role="button" { "Back home" }
        }
    };
    layout(title, "/", body, None)
}

pub fn home(notes: &[Note], csrf: &str) -> String {
    let body = html! {
        section class="hero" {
            p class="eyebrow" { "Cloneable edge starter" }
            h1 { "Evidence Notes" }
            p {
                "A small example exercising D1, R2, Queues, offline replay, HTML fragments, JSON, Markdown and structured data."
            }
        }

        section class="auto-grid" {
            form
                class="card"
                method="post"
                action="/notes"
                hx-post="/notes"
                hx-target="#notes"
                hx-swap="afterbegin"
                hx-target-422="#note-form-errors"
                data-offline-form="create-note"
            {
                h2 { "Create a note" }
                input type="hidden" name="csrf_token" value=(csrf);
                div id="note-form-errors" class="form-errors" role="alert" aria-live="assertive" {}
                label for="title" { "Title" }
                input id="title" name="title" required maxlength="120" autocomplete="off";
                label for="body" { "Body" }
                textarea id="body" name="body" required maxlength="20000" rows="7" {}
                button type="submit" { "Save note" }
                p class="hint" {
                    "Offline submissions are queued locally and replayed with the same idempotency key."
                }
            }

            article class="card" {
                h2 { "What this demonstrates" }
                ul {
                    li { "Server-rendered semantic HTML + HTMX fragments" }
                    li { "Offline-capable PWA" }
                    li { "D1 persistence" }
                    li { "R2 attachment storage" }
                    li { "Queue background jobs" }
                    li { "SEO/AEO representations only after explicit publish" }
                }
                details {
                    summary { "Native HTML first" }
                    p {
                        "This starter prefers details/summary, dialog, popover and native form validation before custom JavaScript."
                    }
                }
                form
                    method="post"
                    action="/upload"
                    enctype="multipart/form-data"
                    hx-post="/upload"
                    hx-encoding="multipart/form-data"
                    hx-target="#upload-result"
                    hx-target-error="#upload-errors"
                    hx-swap="innerHTML"
                {
                    input type="hidden" name="csrf_token" value=(csrf);
                    label for="attachment" { "Try R2 attachment upload" }
                    input id="attachment" type="file" name="file" required;
                    button type="submit" { "Upload to R2" }
                }
                div id="upload-errors" class="form-errors" role="alert" aria-live="assertive" {}
                pre id="upload-result" class="hint" aria-live="polite" {}
            }
        }

        section aria-labelledby="notes-heading" {
            h2 id="notes-heading" { "Notes" }
            div id="notes" class="stack" {
                @for note in notes {
                    (note_card(note, csrf))
                }
            }
        }
    };

    layout("Evidence Notes · Rust + HTMX Starter", "/", body, None)
}

pub fn note_card(note: &Note, csrf: &str) -> Markup {
    let slug = slugify(&note.title);
    let summarize_url = format!("/notes/{}/summarize", note.id);
    let publish_url = format!("/notes/{}/publish", note.id);
    let target = format!("#note-{}", note.id);

    html! {
        article class="card note" id=(format!("note-{}", note.id)) data-js-ready {
            h3 { (&note.title) }
            p { (&note.body) }
            p class="hint" {
                "Status: "
                (match &note.status {
                    NoteStatus::Draft => "draft",
                    NoteStatus::Published => "published",
                    NoteStatus::Archived => "archived",
                })
                " · version " (note.version)
            }
            @if let Some(summary) = &note.summary {
                p class="summary" { strong { "Summary: " } (summary) }
            }
            div class="actions" {
                form
                    method="post"
                    action=(summarize_url.clone())
                    hx-post=(summarize_url)
                    hx-target=(target.clone())
                    hx-swap="outerHTML"
                {
                    input type="hidden" name="csrf_token" value=(csrf);
                    button type="submit" { "Queue summary" }
                }
                @if matches!(&note.status, NoteStatus::Draft) {
                    form
                        method="post"
                        action=(publish_url.clone())
                        hx-post=(publish_url)
                        hx-target=(target)
                        hx-swap="outerHTML"
                    {
                        input type="hidden" name="csrf_token" value=(csrf);
                        button type="submit" class="secondary" { "Publish" }
                    }
                }
                @if matches!(&note.status, NoteStatus::Published) {
                    a href=(format!("/notes/{}/{}", note.id, slug)) { "Public HTML" }
                    a href=(format!("/notes/{}.md", note.id)) { "Markdown" }
                    a href=(format!("/notes/{}.json", note.id)) { "JSON" }
                }
            }
        }
    }
}

pub fn offline_page(csrf: &str) -> String {
    let body = html! {
        section class="hero" {
            p class="eyebrow" { "Offline mode" }
            h1 { "You are offline" }
            p { "Capture a note now. It remains in this browser until connectivity returns." }
        }
        form class="card" method="post" action="/notes" data-offline-form="create-note" {
            h2 { "Capture a note" }
            input type="hidden" name="csrf_token" value=(csrf);
            label for="offline-title" { "Title" }
            input id="offline-title" name="title" required maxlength="120" autocomplete="off";
            label for="offline-body" { "Body" }
            textarea id="offline-body" name="body" required maxlength="20000" rows="7" {}
            button type="submit" { "Save offline" }
        }
    };
    layout("Offline · Rust + HTMX Starter", "/offline", body, None)
}

pub fn public_note(note: &Note) -> String {
    let slug = slugify(&note.title);
    let canonical = format!("/notes/{}/{}", note.id, slug);
    let ld = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": &note.title,
        "articleBody": &note.body,
        "dateModified": note.updated_at_ms,
    });
    let body = html! {
        article class="card" {
            h1 { (&note.title) }
            p { (&note.body) }
            @if let Some(summary) = &note.summary {
                h2 { "Summary" }
                p { (summary) }
            }
        }
    };
    layout(&note.title, &canonical, body, Some(&ld))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_ld_cannot_break_out_of_script_tag() {
        let value = serde_json::json!({"headline": "</script><script>alert(1)</script>"});
        let encoded = safe_json_for_script(&value);
        assert!(!encoded.contains("</script>"));
        assert!(encoded.contains("\\u003c/script\\u003e"));
    }
}
