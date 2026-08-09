use starter_contracts::{
    ApiError, CONTRACT_VERSION, CreateNoteRequest, JobV1, Note, NoteStatus, contract_catalog_json,
};
use starter_domain::{ALLOWED_UPLOAD_TYPES, slugify, validate_create_note, verify_upload};
use starter_observability::log_event;
use starter_shared::request_id;
use uuid::Uuid;
use worker::*;

const APP_CSS: &str = include_str!("../../../public/app.css");
const APP_JS: &str = include_str!("../../../public/app.js");
const PICO: &str = include_str!("../../../public/vendor/pico.min.css");
const RESPONSE_TARGETS: &str = include_str!("../../../public/vendor/response-targets.js");
const SERVICE_WORKER: &str = include_str!("../../../public/sw.js");
const MANIFEST: &str = include_str!("../../../public/manifest.webmanifest");
const HTMX: &str = include_str!("../../../public/vendor/htmx.min.js");
const ICON: &str = include_str!("../../../public/icon.svg");
const ICON_192: &[u8] = include_bytes!("../../../public/icon-192.png");
const ICON_512: &[u8] = include_bytes!("../../../public/icon-512.png");
const APPLE_TOUCH_ICON: &[u8] = include_bytes!("../../../public/apple-touch-icon.png");

fn now_ms() -> i64 {
    Date::now().as_millis() as i64
}

fn secured(mut response: Response, request_id: &str, private: bool) -> Result<Response> {
    let h = response.headers_mut();
    h.set("x-request-id", request_id)?;
    h.set("x-content-type-options", "nosniff")?;
    h.set("referrer-policy", "strict-origin-when-cross-origin")?;
    h.set(
        "permissions-policy",
        "camera=(), microphone=(), geolocation=()",
    )?;
    h.set("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'")?;
    h.set(
        "cache-control",
        if private {
            "no-store"
        } else {
            "public, max-age=60, stale-while-revalidate=300"
        },
    )?;
    Ok(response)
}

fn text(body: &str, content_type: &str) -> Result<Response> {
    let mut r = Response::ok(body)?;
    r.headers_mut().set("content-type", content_type)?;
    Ok(r)
}

fn binary(body: &[u8], content_type: &str) -> Result<Response> {
    let mut r = Response::from_bytes(body.to_vec())?;
    r.headers_mut().set("content-type", content_type)?;
    r.headers_mut()
        .set("cache-control", "public, max-age=86400")?;
    Ok(r)
}

fn not_found() -> Result<Response> {
    Response::error("not found", 404)
}

/// Developer placeholder for the authentication boundary (see docs/AUTH.md):
/// real auth (Cloudflare Access, OIDC, or cookie sessions) must populate this
/// same per-request context. Until then the caller presents an `x-user-id`
/// header, which is NOT a production credential. In local dev only, an empty
/// `DEV_USER_ID` var in wrangler.jsonc can be overridden via the ignored
/// `.dev.vars` so the browser demo works without header injection — never set
/// it in production.
fn current_user(req: &Request, env: &Env) -> Result<Uuid> {
    if let Some(raw) = req.headers().get("x-user-id").ok().flatten() {
        return Uuid::parse_str(&raw).map_err(|_| Error::RustError("invalid x-user-id".into()));
    }
    if let Ok(var) = env.var("DEV_USER_ID") {
        let raw = var.to_string();
        if !raw.is_empty() {
            return Uuid::parse_str(&raw)
                .map_err(|_| Error::RustError("invalid DEV_USER_ID".into()));
        }
    }
    Err(Error::RustError("missing x-user-id".into()))
}

fn unauthorized(request_id: &str) -> Result<Response> {
    let body = ApiError {
        code: "unauthorized".into(),
        message: "authentication required".into(),
        request_id: request_id.into(),
    };
    secured(
        Response::from_json(&body)?.with_status(401),
        request_id,
        true,
    )
}

const RATE_LIMIT_BINDING: &str = "RATE_LIMITER";

/// Cloudflare Rate Limiting API guard (see workers/app/wrangler.jsonc). Keys
/// on the owner id, so limits are per-user rather than per-IP. Returns
/// `Ok(Some(response))` when limited — the caller should return it
/// immediately — and `Ok(None)` when the request may proceed. Rate limits are
/// local to the Cloudflare location serving the request, so this is a coarse
/// abuse brake, not an exact accounting system.
async fn enforce_rate_limit(
    ctx: &RouteContext<()>,
    key: String,
    request_id: &str,
) -> Result<Option<Response>> {
    let limiter = ctx.rate_limiter(RATE_LIMIT_BINDING)?;
    let outcome = limiter.limit(key).await?;
    if outcome.success {
        Ok(None)
    } else {
        let body = ApiError {
            code: "rate_limited".into(),
            message: "too many requests, please slow down".into(),
            request_id: request_id.into(),
        };
        let mut response = secured(
            Response::from_json(&body)?.with_status(429),
            request_id,
            true,
        )?;
        response.headers_mut().set("retry-after", "60")?;
        Ok(Some(response))
    }
}

const CSRF_COOKIE: &str = "csrf_token";

/// Double-submit CSRF state for one request. The token rides in a cookie and
/// is mirrored into every state-changing form as a hidden field; the server
/// compares the two on POST. See docs/AUTH.md for the full flow.
struct CsrfState {
    token: String,
    minted: bool,
}

fn mint_csrf_token() -> String {
    // Two UUIDv4 values = 64 hex chars from Web Crypto's CSPRNG.
    let mut s = format!("{}{}", Uuid::new_v4(), Uuid::new_v4());
    s.retain(|c| c != '-');
    s
}

fn csrf_cookie_from(req: &Request) -> Option<String> {
    let cookies = req.headers().get("cookie").ok().flatten()?;
    cookies.split(';').find_map(|pair| {
        let mut it = pair.trim().splitn(2, '=');
        match (it.next(), it.next()) {
            (Some(name), Some(value)) if name == CSRF_COOKIE => Some(value.trim().to_string()),
            _ => None,
        }
    })
}

/// Reuse the session's token when the cookie is present; otherwise mint one
/// so the response can set it. Minting is stateless — the token is verified
/// by comparing the cookie against the form field, never looked up.
fn csrf_state(req: &Request) -> CsrfState {
    match csrf_cookie_from(req) {
        Some(token) => CsrfState {
            token,
            minted: false,
        },
        None => CsrfState {
            token: mint_csrf_token(),
            minted: true,
        },
    }
}

fn attach_csrf_cookie(mut response: Response, csrf: &CsrfState) -> Result<Response> {
    if csrf.minted {
        response.headers_mut().set(
            "set-cookie",
            &format!(
                "{CSRF_COOKIE}={}; Path=/; HttpOnly; SameSite=Lax; Secure",
                csrf.token
            ),
        )?;
    }
    Ok(response)
}

/// Enforce the double-submit check: the form's `csrf_token` field must equal
/// the request's `csrf_token` cookie. Returns `Ok(Some(403))` on mismatch.
/// Enforcement activates only when a CSRF cookie is present, which happens
/// exactly when cookie-session auth is live; with the dev `x-user-id` header
/// there is no cookie to protect, so the check is skipped. A cookie-carrying
/// request whose body failed to parse fails closed (no token == no pass).
fn csrf_guard(
    req: &Request,
    form: Option<&worker::FormData>,
    request_id: &str,
) -> Result<Option<Response>> {
    let cookie_token = match csrf_cookie_from(req) {
        Some(t) => t,
        None => return Ok(None),
    };
    // The token is a short hex string; cloning keeps this borrow-free.
    let submitted = match form {
        Some(f) => match f.get("csrf_token") {
            Some(FormEntry::Field(s)) => Some(s.clone()),
            _ => None,
        },
        None => None,
    };
    if starter_domain::csrf_token_valid(submitted.as_deref(), Some(&cookie_token)) {
        Ok(None)
    } else {
        let body = ApiError {
            code: "csrf_failed".into(),
            message: "request token mismatch".into(),
            request_id: request_id.into(),
        };
        Ok(Some(secured(
            Response::from_json(&body)?.with_status(403),
            request_id,
            true,
        )?))
    }
}

fn is_htmx(req: &Request) -> bool {
    req.headers().get("HX-Request").ok().flatten().as_deref() == Some("true")
}

/// Builds a 303 redirect with the usual security headers.
///
/// `Response::redirect*` cannot be used here: per the Fetch spec its headers
/// have an "immutable" guard, so `secured()` (which mutates headers) throws
/// "Can't modify immutable headers" in the Workers runtime. The Location
/// header is set on a plain, mutable response instead.
fn redirect_303(location: Url, request_id: &str) -> Result<Response> {
    let mut r = Response::empty()?.with_status(303);
    r.headers_mut().set("location", location.as_str())?;
    secured(r, request_id, true)
}

fn redirect_home(req: &Request, request_id: &str) -> Result<Response> {
    let url = req.url()?.join("/")?;
    redirect_303(url, request_id)
}

fn validation_response(message: &str, request_id: &str, htmx: bool) -> Result<Response> {
    let body = starter_templates::error_fragment(message).into_string();
    let mut r = if htmx {
        Response::from_html(body)?
    } else {
        Response::from_html(starter_templates::error_page("Validation error", message))?
    };
    r = r.with_status(422);
    secured(r, request_id, true)
}
async fn create_note(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let htmx = is_htmx(&req);
    let owner = match current_user(&req, &ctx.env) {
        Ok(u) => u,
        Err(_) => return unauthorized(&request_id),
    };
    if let Some(limited) = enforce_rate_limit(&ctx, owner.to_string(), &request_id).await? {
        return Ok(limited);
    }
    let csrf = csrf_state(&req);
    let fallback = req.url()?.join("/")?;
    let form = req.form_data().await?;
    if let Some(forbidden) = csrf_guard(&req, Some(&form), &request_id)? {
        return Ok(forbidden);
    }
    let title = match form.get("title") {
        Some(FormEntry::Field(v)) => v,
        _ => return Response::error("title required", 400),
    };
    let body = match form.get("body") {
        Some(FormEntry::Field(v)) => v,
        _ => return Response::error("body required", 400),
    };
    let operation_id = form
        .get("operation_id")
        .and_then(|v| match v {
            FormEntry::Field(s) => Uuid::parse_str(&s).ok(),
            _ => None,
        })
        .unwrap_or_else(Uuid::new_v4);
    let input = CreateNoteRequest {
        title: title.clone(),
        body: body.clone(),
        operation_id,
    };
    if let Err(err) = validate_create_note(&input) {
        return validation_response(&err.to_string(), &request_id, htmx);
    }
    let now = now_ms();
    let candidate = Note {
        id: Uuid::new_v4(),
        title,
        body,
        status: NoteStatus::Draft,
        version: 1,
        created_at_ms: now,
        updated_at_ms: now,
        summary: None,
    };
    let db = ctx.d1("DB")?;
    let persisted = starter_database::create_note(&db, &candidate, operation_id, owner).await?;
    log_event(
        "note.created",
        &request_id,
        &serde_json::json!({"noteId": persisted.id, "operationId": operation_id}),
    );
    if htmx {
        let r = secured(
            Response::from_html(
                starter_templates::note_card(&persisted, &csrf.token).into_string(),
            )?,
            &request_id,
            true,
        )?;
        attach_csrf_cookie(r, &csrf)
    } else {
        let r = redirect_303(fallback, &request_id)?;
        attach_csrf_cookie(r, &csrf)
    }
}

async fn summarize(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let htmx = is_htmx(&req);
    let owner = match current_user(&req, &ctx.env) {
        Ok(u) => u,
        Err(_) => return unauthorized(&request_id),
    };
    if let Some(limited) = enforce_rate_limit(&ctx, owner.to_string(), &request_id).await? {
        return Ok(limited);
    }
    let csrf = csrf_state(&req);
    // HTMX serializes the form's hidden csrf_token field into this body.
    // Bodyless POSTs (plain clients, e2e smoke) parse to None and are only
    // checked when a CSRF cookie is present.
    let form = req.form_data().await.ok();
    if let Some(forbidden) = csrf_guard(&req, form.as_ref(), &request_id)? {
        return Ok(forbidden);
    }
    let id = ctx
        .param("id")
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| Error::RustError("invalid note id".into()))?;
    // Ownership gate BEFORE enqueuing: a foreign or missing note must never
    // reach the queue (this also closes the route that let anyone summarize
    // someone else's note).
    let db = ctx.d1("DB")?;
    let Some(note) = starter_database::find_owned_note(&db, id, owner).await? else {
        return not_found();
    };
    let job = JobV1::SummarizeNote {
        contract_version: CONTRACT_VERSION,
        job_id: Uuid::new_v4(),
        note_id: id,
    };
    ctx.env.queue("JOBS")?.send(job).await?;
    if htmx {
        attach_csrf_cookie(
            secured(
                Response::from_html(
                    starter_templates::note_card(&note, &csrf.token).into_string(),
                )?,
                &request_id,
                true,
            )?,
            &csrf,
        )
    } else {
        redirect_home(&req, &request_id)
    }
}

async fn publish(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let htmx = is_htmx(&req);
    let owner = match current_user(&req, &ctx.env) {
        Ok(u) => u,
        Err(_) => return unauthorized(&request_id),
    };
    if let Some(limited) = enforce_rate_limit(&ctx, owner.to_string(), &request_id).await? {
        return Ok(limited);
    }
    let csrf = csrf_state(&req);
    let form = req.form_data().await.ok();
    if let Some(forbidden) = csrf_guard(&req, form.as_ref(), &request_id)? {
        return Ok(forbidden);
    }
    let id = ctx
        .param("id")
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| Error::RustError("invalid note id".into()))?;
    let db = ctx.d1("DB")?;
    let Some(note) = starter_database::publish_note(&db, id, owner, now_ms()).await? else {
        return not_found();
    };
    if htmx {
        attach_csrf_cookie(
            secured(
                Response::from_html(
                    starter_templates::note_card(&note, &csrf.token).into_string(),
                )?,
                &request_id,
                true,
            )?,
            &csrf,
        )
    } else {
        redirect_home(&req, &request_id)
    }
}

async fn upload(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let owner = match current_user(&req, &ctx.env) {
        Ok(u) => u,
        Err(_) => return unauthorized(&request_id),
    };
    if let Some(limited) = enforce_rate_limit(&ctx, owner.to_string(), &request_id).await? {
        return Ok(limited);
    }
    let csrf = csrf_state(&req);
    let form = req.form_data().await?;
    if let Some(forbidden) = csrf_guard(&req, Some(&form), &request_id)? {
        return Ok(forbidden);
    }
    let file = match form.get("file") {
        Some(FormEntry::File(f)) => f,
        _ => return Response::error("file required", 400),
    };
    let content_type = file.type_();
    let size = file.size();
    if size > 10 * 1024 * 1024 {
        return Response::error("file exceeds 10 MiB", 413);
    }
    if !ALLOWED_UPLOAD_TYPES.iter().any(|v| *v == content_type) {
        return Response::error("unsupported file type", 415);
    }
    let bytes = file.bytes().await?;
    if bytes.len() != size {
        return Response::error("upload size mismatch", 400);
    }
    // Magic-byte verification: the declared content type must match the
    // actual bytes, so a client cannot smuggle HTML as text/plain or
    // mislabel one binary format as another. Text formats get UTF-8/NUL/HTML
    // heuristics since they have no reliable signature.
    if let Err(err) = verify_upload(&content_type, &bytes) {
        return Response::error(err.to_string(), 415);
    }
    // Per-owner quota: reject before the R2 write so over-quota owners never
    // store anything. Check-then-write is not a strict serialization — two
    // concurrent uploads can both pass — which the 100 MiB headroom absorbs.
    let db = ctx.d1("DB")?;
    let used = starter_database::upload_bytes_used(&db, owner).await?;
    if !starter_domain::within_upload_quota(used, size as u64) {
        return Response::error("upload quota exceeded", 413);
    }
    let key = format!("attachments/{}/{}", owner, Uuid::new_v4());
    ctx.bucket("ATTACHMENTS")?
        .put(&key, bytes)
        .execute()
        .await?;
    starter_database::record_upload(&db, owner, size as u64, now_ms()).await?;
    let response = starter_contracts::AttachmentMetadataV1 {
        contract_version: CONTRACT_VERSION,
        key,
        content_type,
        size_bytes: size as u64,
    };
    let r = secured(Response::from_json(&response)?, &request_id, true)?;
    attach_csrf_cookie(r, &csrf)
}

async fn published_note(ctx: &RouteContext<()>) -> Result<Option<Note>> {
    let id = ctx
        .param("id")
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| Error::RustError("invalid note id".into()))?;
    let db = ctx.d1("DB")?;
    let note = starter_database::find_note(&db, id).await?;
    Ok(note.filter(|n| n.status == NoteStatus::Published))
}

async fn note_page(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let Some(note) = published_note(&ctx).await? else {
        return not_found();
    };
    secured(
        Response::from_html(starter_templates::public_note(&note))?,
        &request_id,
        false,
    )
}

/// Serves a published note as JSON or Markdown from one route.
///
/// matchit 0.7 (pinned by worker 0.8.5) rejects a second param-with-suffix
/// route at the same position, so `/notes/:id.json` and `/notes/:id.md`
/// cannot both be registered ("failed to register ... conflict with
/// previously registered route"). `/notes/:id.ext` captures the whole
/// segment (`<uuid>.json` / `<uuid>.md`) and we dispatch on the extension.
async fn note_data(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let Some((id_str, ext)) = ctx.param("id.ext").and_then(|s| s.rsplit_once('.')) else {
        return not_found();
    };
    let Ok(id) = Uuid::parse_str(id_str) else {
        return not_found();
    };
    let db = ctx.d1("DB")?;
    let Some(note) = starter_database::find_note(&db, id)
        .await?
        .filter(|n| n.status == NoteStatus::Published)
    else {
        return not_found();
    };
    match ext {
        "json" => secured(Response::from_json(&note)?, &request_id, false),
        "md" => {
            let body = format!(
                "# {}\n\n{}\n\n{}",
                note.title,
                note.body,
                note.summary
                    .as_ref()
                    .map(|s| format!("## Summary\n\n{s}"))
                    .unwrap_or_default()
            );
            secured(
                text(&body, "text/markdown; charset=utf-8")?,
                &request_id,
                false,
            )
        }
        _ => not_found(),
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/", |req, ctx| async move {
            let request_id = request_id();
            let owner = match current_user(&req, &ctx.env) {
                Ok(u) => u,
                Err(_) => return unauthorized(&request_id),
            };
            let csrf = csrf_state(&req);
            let notes = starter_database::list_notes(&ctx.d1("DB")?, owner).await?;
            let r = secured(
                Response::from_html(starter_templates::home(&notes, &csrf.token))?,
                &request_id,
                true,
            )?;
            attach_csrf_cookie(r, &csrf)
        })
        .post_async("/notes", create_note)
        .post_async("/notes/:id/summarize", summarize)
        .post_async("/notes/:id/publish", publish)
        .post_async("/upload", upload)
        .get_async("/notes/:id/:slug", note_page)
        .get_async("/notes/:id.ext", note_data)
        .get("/contracts", |_req, _ctx| {
            let rid = request_id();
            secured(Response::from_json(&contract_catalog_json())?, &rid, false)
        })
        .get("/healthz", |_req, _ctx| {
            let rid = request_id();
            secured(
                Response::from_json(&serde_json::json!({"ok":true,"version":env!("CARGO_PKG_VERSION")}))?,
                &rid,
                true,
            )
        })
        .get("/offline", |req, _ctx| {
            let rid = request_id();
            let csrf = csrf_state(&req);
            let r = secured(Response::from_html(starter_templates::offline_page(&csrf.token))?, &rid, true)?;
            attach_csrf_cookie(r, &csrf)
        })
        .get("/robots.txt", |req, _ctx| {
            let origin = req.url()?.origin().ascii_serialization();
            text(
                &format!("User-agent: *\nAllow: /\nSitemap: {origin}/sitemap.xml\n"),
                "text/plain; charset=utf-8",
            )
        })
        .get_async("/sitemap.xml", |req, ctx| async move {
            let origin = req.url()?.origin().ascii_serialization();
            let notes = starter_database::list_published_notes(&ctx.d1("DB")?).await?;
            let mut urls = format!("<url><loc>{origin}/</loc></url>");
            for note in notes {
                urls.push_str(&format!(
                    "<url><loc>{origin}/notes/{}/{}</loc></url>",
                    note.id,
                    slugify(&note.title)
                ));
            }
            text(
                &format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{urls}</urlset>"),
                "application/xml; charset=utf-8",
            )
        })
        .get("/llms.txt", |_req, _ctx| {
            text(
                "# Rust + HTMX Starter\n\nA lightweight Cloudflare starter demonstrating HTML, Markdown, JSON, PWA/offline sync, D1, R2, and Queues.\n",
                "text/plain; charset=utf-8",
            )
        })
        .get("/manifest.webmanifest", |_req, _ctx| {
            text(MANIFEST, "application/manifest+json")
        })
        .get("/sw.js", |_req, _ctx| {
            let mut response = text(SERVICE_WORKER, "text/javascript; charset=utf-8")?;
            response.headers_mut().set("cache-control", "no-cache")?;
            Ok(response)
        })
        .get("/icon.svg", |_req, _ctx| text(ICON, "image/svg+xml; charset=utf-8"))
        .get("/icon-192.png", |_req, _ctx| binary(ICON_192, "image/png"))
        .get("/icon-512.png", |_req, _ctx| binary(ICON_512, "image/png"))
        .get("/apple-touch-icon.png", |_req, _ctx| binary(APPLE_TOUCH_ICON, "image/png"))
        .get("/assets/vendor/pico.min.css", |_req, _ctx| text(PICO, "text/css; charset=utf-8"))
        .get("/assets/app.css", |_req, _ctx| text(APP_CSS, "text/css; charset=utf-8"))
        .get("/assets/app.js", |_req, _ctx| text(APP_JS, "text/javascript; charset=utf-8"))
        .get("/assets/vendor/response-targets.js", |_req, _ctx| text(RESPONSE_TARGETS, "text/javascript; charset=utf-8"))
        .get("/assets/vendor/htmx.min.js", |_req, _ctx| {
            text(HTMX, "text/javascript; charset=utf-8")
        })
        .run(req, env)
        .await
}
