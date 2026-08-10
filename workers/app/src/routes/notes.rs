use starter_contracts::{CONTRACT_VERSION, CreateNoteRequest, JobV1, Note, NoteStatus};
use starter_domain::validate_create_note;
use starter_observability::log_event;
use starter_shared::request_id;
use uuid::Uuid;
use worker::{Date, Error, FormEntry, Request, Response, Result, RouteContext};

use crate::auth::{attach_csrf_cookie, csrf_guard, csrf_state, current_user, unauthorized};
use crate::http::{
    CachePolicy, client_error, is_htmx, not_found, redirect_303, redirect_home, secured,
    validation_response,
};
use crate::rate_limit::enforce_rate_limit;

fn now_ms() -> i64 {
    Date::now().as_millis() as i64
}

pub(crate) async fn home(req: Request, context: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let owner = match current_user(&req, &context.env) {
        Ok(owner) => owner,
        Err(_) => return unauthorized(&request_id),
    };
    let csrf = csrf_state(&req);
    let notes = starter_database::list_notes(&context.d1("DB")?, owner).await?;
    let origin = req.url()?.origin().ascii_serialization();
    let html = starter_templates::render_home(&notes, &csrf.token, &origin)
        .map_err(|error| Error::RustError(format!("template render failed: {error}")))?;
    let response = secured(
        Response::from_html(html)?,
        &request_id,
        CachePolicy::Private,
    )?;
    attach_csrf_cookie(response, &csrf)
}

pub(crate) async fn create(mut req: Request, context: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let htmx = is_htmx(&req);
    let owner = match current_user(&req, &context.env) {
        Ok(owner) => owner,
        Err(_) => return unauthorized(&request_id),
    };
    if let Some(response) = enforce_rate_limit(&context, owner.to_string(), &request_id).await? {
        return Ok(response);
    }
    let csrf = csrf_state(&req);
    let origin = req.url()?.origin().ascii_serialization();
    let fallback = req.url()?.join("/")?;
    let form = req.form_data().await?;
    if let Some(response) = csrf_guard(&req, Some(&form), &request_id)? {
        return Ok(response);
    }
    let title = match form.get("title") {
        Some(FormEntry::Field(value)) => value,
        _ => return client_error("title required", 400, &request_id),
    };
    let body = match form.get("body") {
        Some(FormEntry::Field(value)) => value,
        _ => return client_error("body required", 400, &request_id),
    };
    let operation_id = form
        .get("operation_id")
        .and_then(|entry| match entry {
            FormEntry::Field(value) => Uuid::parse_str(&value).ok(),
            _ => None,
        })
        .unwrap_or_else(Uuid::new_v4);
    let input = CreateNoteRequest {
        title: title.clone(),
        body: body.clone(),
        operation_id,
    };
    if let Err(error) = validate_create_note(&input) {
        return validation_response(&error.to_string(), &request_id, htmx, &origin);
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
    let persisted =
        starter_database::create_note(&context.d1("DB")?, &candidate, operation_id, owner).await?;
    log_event(
        "note.created",
        &request_id,
        &serde_json::json!({"noteId": persisted.id, "operationId": operation_id}),
    );
    let response = if htmx {
        let html = starter_templates::render_note_card(&persisted, &csrf.token)
            .map_err(|error| Error::RustError(format!("template render failed: {error}")))?;
        secured(
            Response::from_html(html)?,
            &request_id,
            CachePolicy::Private,
        )?
    } else {
        redirect_303(fallback, &request_id, CachePolicy::Private)?
    };
    attach_csrf_cookie(response, &csrf)
}

pub(crate) async fn summarize(mut req: Request, context: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let htmx = is_htmx(&req);
    let owner = match current_user(&req, &context.env) {
        Ok(owner) => owner,
        Err(_) => return unauthorized(&request_id),
    };
    if let Some(response) = enforce_rate_limit(&context, owner.to_string(), &request_id).await? {
        return Ok(response);
    }
    let csrf = csrf_state(&req);
    let form = req.form_data().await.ok();
    if let Some(response) = csrf_guard(&req, form.as_ref(), &request_id)? {
        return Ok(response);
    }
    let id = route_id(&context)?;
    let database = context.d1("DB")?;
    let Some(note) = starter_database::find_owned_note(&database, id, owner).await? else {
        return not_found(&request_id);
    };
    context
        .env
        .queue("JOBS")?
        .send(JobV1::SummarizeNote {
            contract_version: CONTRACT_VERSION,
            job_id: Uuid::new_v4(),
            note_id: id,
        })
        .await?;
    if !htmx {
        return redirect_home(&req, &request_id);
    }
    let html = starter_templates::render_note_card(&note, &csrf.token)
        .map_err(|error| Error::RustError(format!("template render failed: {error}")))?;
    let response = secured(
        Response::from_html(html)?,
        &request_id,
        CachePolicy::Private,
    )?;
    attach_csrf_cookie(response, &csrf)
}

pub(crate) async fn publish(mut req: Request, context: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let htmx = is_htmx(&req);
    let owner = match current_user(&req, &context.env) {
        Ok(owner) => owner,
        Err(_) => return unauthorized(&request_id),
    };
    if let Some(response) = enforce_rate_limit(&context, owner.to_string(), &request_id).await? {
        return Ok(response);
    }
    let csrf = csrf_state(&req);
    let form = req.form_data().await.ok();
    if let Some(response) = csrf_guard(&req, form.as_ref(), &request_id)? {
        return Ok(response);
    }
    let database = context.d1("DB")?;
    let Some(note) =
        starter_database::publish_note(&database, route_id(&context)?, owner, now_ms()).await?
    else {
        return not_found(&request_id);
    };
    if !htmx {
        return redirect_home(&req, &request_id);
    }
    let html = starter_templates::render_note_card(&note, &csrf.token)
        .map_err(|error| Error::RustError(format!("template render failed: {error}")))?;
    let response = secured(
        Response::from_html(html)?,
        &request_id,
        CachePolicy::Private,
    )?;
    attach_csrf_cookie(response, &csrf)
}

fn route_id(context: &RouteContext<()>) -> Result<Uuid> {
    context
        .param("id")
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| Error::RustError("invalid note id".into()))
}
