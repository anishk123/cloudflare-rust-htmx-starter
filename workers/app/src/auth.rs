use starter_contracts::ApiError;
use uuid::Uuid;
use worker::{Env, Error, FormData, FormEntry, Request, Response, Result};

use crate::http::{CachePolicy, secured};

const CSRF_COOKIE: &str = "csrf_token";

/// Temporary authentication adapter. Production applications should replace
/// this boundary with Access, OIDC, or a cookie session (see docs/AUTH.md).
pub(crate) fn current_user(req: &Request, env: &Env) -> Result<Uuid> {
    if let Some(raw) = req.headers().get("x-user-id").ok().flatten() {
        return Uuid::parse_str(&raw).map_err(|_| Error::RustError("invalid x-user-id".into()));
    }
    if let Ok(value) = env.var("DEV_USER_ID") {
        let raw = value.to_string();
        if !raw.is_empty() {
            return Uuid::parse_str(&raw)
                .map_err(|_| Error::RustError("invalid DEV_USER_ID".into()));
        }
    }
    Err(Error::RustError("missing x-user-id".into()))
}

pub(crate) fn unauthorized(request_id: &str) -> Result<Response> {
    let body = ApiError {
        code: "unauthorized".into(),
        message: "authentication required".into(),
        request_id: request_id.into(),
    };
    secured(
        Response::from_json(&body)?.with_status(401),
        request_id,
        CachePolicy::Private,
    )
}

pub(crate) struct CsrfState {
    pub(crate) token: String,
    minted: bool,
}

fn mint_csrf_token() -> String {
    let mut token = format!("{}{}", Uuid::new_v4(), Uuid::new_v4());
    token.retain(|character| character != '-');
    token
}

fn csrf_cookie_from(req: &Request) -> Option<String> {
    let cookies = req.headers().get("cookie").ok().flatten()?;
    cookies.split(';').find_map(|pair| {
        let mut parts = pair.trim().splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some(name), Some(value)) if name == CSRF_COOKIE => Some(value.trim().to_string()),
            _ => None,
        }
    })
}

pub(crate) fn csrf_state(req: &Request) -> CsrfState {
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

pub(crate) fn attach_csrf_cookie(mut response: Response, csrf: &CsrfState) -> Result<Response> {
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

pub(crate) fn csrf_guard(
    req: &Request,
    form: Option<&FormData>,
    request_id: &str,
) -> Result<Option<Response>> {
    let cookie_token = csrf_cookie_from(req);
    let submitted = form.and_then(|data| match data.get("csrf_token") {
        Some(FormEntry::Field(value)) => Some(value.clone()),
        _ => None,
    });
    if starter_domain::csrf_token_valid(submitted.as_deref(), cookie_token.as_deref()) {
        return Ok(None);
    }
    let body = ApiError {
        code: "csrf_failed".into(),
        message: "request token mismatch".into(),
        request_id: request_id.into(),
    };
    Ok(Some(secured(
        Response::from_json(&body)?.with_status(403),
        request_id,
        CachePolicy::Private,
    )?))
}

pub(crate) fn csrf_response(req: &Request, env: &Env, request_id: &str) -> Result<Response> {
    if current_user(req, env).is_err() {
        return unauthorized(request_id);
    }
    let csrf = csrf_state(req);
    let response = secured(
        Response::from_json(&serde_json::json!({"csrf_token": csrf.token}))?,
        request_id,
        CachePolicy::Private,
    )?;
    attach_csrf_cookie(response, &csrf)
}
