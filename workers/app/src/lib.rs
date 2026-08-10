mod auth;
mod http;
mod rate_limit;
mod routes;

use starter_shared::request_id;
use worker::{Context, Env, Request, Response, Result, Router, event};

#[event(fetch)]
pub async fn main(req: Request, env: Env, _context: Context) -> Result<Response> {
    Router::new()
        .get_async("/", routes::notes::home)
        .post_async("/notes", routes::notes::create)
        .post_async("/notes/:id/summarize", routes::notes::summarize)
        .post_async("/notes/:id/publish", routes::notes::publish)
        .post_async("/upload", routes::uploads::upload)
        .get("/session/csrf", |req, context| {
            auth::csrf_response(&req, &context.env, &request_id())
        })
        .get_async("/notes/:id/:slug", routes::public::page)
        .get_async("/notes/:id.ext", routes::public::data)
        .get("/contracts", |_req, _context| {
            routes::system::contracts(&request_id())
        })
        .get("/healthz", |_req, _context| {
            routes::system::health(&request_id())
        })
        .get("/offline", |req, _context| {
            routes::system::offline_redirect(&req, &request_id())
        })
        .get("/robots.txt", |req, _context| {
            routes::system::robots(&req, &request_id())
        })
        .get_async("/sitemap.xml", routes::system::sitemap)
        .get_async("/llms.txt", routes::system::llms)
        .get("/design-system", |req, _context| {
            routes::system::design_system(&req, &request_id())
        })
        .run(req, env)
        .await
}
