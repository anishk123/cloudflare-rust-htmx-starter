use starter_contracts::contract_catalog_json;
use starter_domain::slugify;
use starter_shared::request_id;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use worker::{Error, Request, Response, Result, RouteContext};

use crate::http::{CachePolicy, redirect_303, secured, text};

pub(crate) fn contracts(request_id: &str) -> Result<Response> {
    secured(
        Response::from_json(&contract_catalog_json())?,
        request_id,
        CachePolicy::PublicDiscovery,
    )
}

pub(crate) fn health(request_id: &str) -> Result<Response> {
    secured(
        Response::from_json(
            &serde_json::json!({"ok": true, "version": env!("CARGO_PKG_VERSION")}),
        )?,
        request_id,
        CachePolicy::Private,
    )
}

pub(crate) fn offline_redirect(req: &Request, request_id: &str) -> Result<Response> {
    redirect_303(
        req.url()?.join("/offline.html")?,
        request_id,
        CachePolicy::Private,
    )
}

pub(crate) fn robots(req: &Request, request_id: &str) -> Result<Response> {
    let origin = req.url()?.origin().ascii_serialization();
    secured(
        text(
            &format!("User-agent: *\nAllow: /\nSitemap: {origin}/sitemap.xml\n"),
            "text/plain; charset=utf-8",
        )?,
        request_id,
        CachePolicy::PublicDiscovery,
    )
}

pub(crate) async fn sitemap(req: Request, context: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let origin = req.url()?.origin().ascii_serialization();
    let notes = starter_database::list_published_notes(&context.d1("DB")?).await?;
    let mut urls = String::new();
    for note in notes {
        urls.push_str(&format!(
            "<url><loc>{origin}/notes/{}/{}</loc><lastmod>{}</lastmod></url>",
            note.id,
            slugify(&note.title),
            rfc3339_from_millis(note.updated_at_ms),
        ));
    }
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{urls}</urlset>"
    );
    secured(
        text(&xml, "application/xml; charset=utf-8")?,
        &request_id,
        CachePolicy::PublicDiscovery,
    )
}

pub(crate) async fn llms(req: Request, context: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let origin = req.url()?.origin().ascii_serialization();
    let notes = starter_database::list_published_notes(&context.d1("DB")?).await?;
    let mut body = format!(
        "# Rust + HTMX Starter\n\n> Fast, progressively enhanced applications on Cloudflare with Rust, Askama, and HTMX.\n\n## Discovery\n\n- [Contract catalog]({origin}/contracts)\n- [Sitemap]({origin}/sitemap.xml)\n\n## Published notes\n"
    );
    if notes.is_empty() {
        body.push_str("\nNo notes have been published yet.\n");
    }
    for note in notes {
        let id = note.id;
        let label = markdown_label(&note.title);
        body.push_str(&format!(
            "\n- [{label}]({origin}/notes/{id}/{})\n  - [Markdown]({origin}/notes/{id}.md)\n  - [JSON]({origin}/notes/{id}.json)\n",
            slugify(&note.title)
        ));
    }
    secured(
        text(&body, "text/plain; charset=utf-8")?,
        &request_id,
        CachePolicy::PublicDiscovery,
    )
}

fn rfc3339_from_millis(value: i64) -> String {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value) * 1_000_000)
        .ok()
        .and_then(|timestamp| timestamp.format(&Rfc3339).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".into())
}

fn markdown_label(value: &str) -> String {
    value
        .replace(['\r', '\n'], " ")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

pub(crate) fn design_system(req: &Request, request_id: &str) -> Result<Response> {
    let origin = req.url()?.origin().ascii_serialization();
    let html = starter_templates::render_design_system(&origin)
        .map_err(|error| Error::RustError(format!("template render failed: {error}")))?;
    secured(
        Response::from_html(html)?,
        request_id,
        CachePolicy::PublicDiscovery,
    )
}
