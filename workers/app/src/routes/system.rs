use starter_contracts::contract_catalog_json;
use starter_domain::slugify;
use starter_shared::request_id;
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
    let mut urls = format!("<url><loc>{origin}/</loc></url>");
    for note in notes {
        urls.push_str(&format!(
            "<url><loc>{origin}/notes/{}/{}</loc></url>",
            note.id,
            slugify(&note.title)
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

pub(crate) fn llms(request_id: &str) -> Result<Response> {
    secured(
        text(
            "# Rust + HTMX Starter\n\nA lightweight Cloudflare starter demonstrating HTML, Markdown, JSON, PWA/offline sync, D1, R2, and Queues.\n",
            "text/plain; charset=utf-8",
        )?,
        request_id,
        CachePolicy::PublicDiscovery,
    )
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
