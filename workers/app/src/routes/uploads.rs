use starter_contracts::CONTRACT_VERSION;
use starter_domain::{ALLOWED_UPLOAD_TYPES, verify_upload};
use starter_shared::request_id;
use uuid::Uuid;
use worker::{Date, FormEntry, Request, Response, Result, RouteContext};

use crate::auth::{attach_csrf_cookie, csrf_guard, csrf_state, current_user, unauthorized};
use crate::http::{CachePolicy, client_error, secured};
use crate::rate_limit::enforce_rate_limit;

pub(crate) async fn upload(mut req: Request, context: RouteContext<()>) -> Result<Response> {
    let request_id = request_id();
    let owner = match current_user(&req, &context.env) {
        Ok(owner) => owner,
        Err(_) => return unauthorized(&request_id),
    };
    if let Some(response) = enforce_rate_limit(&context, owner.to_string(), &request_id).await? {
        return Ok(response);
    }
    let csrf = csrf_state(&req);
    let form = req.form_data().await?;
    if let Some(response) = csrf_guard(&req, Some(&form), &request_id)? {
        return Ok(response);
    }
    let file = match form.get("file") {
        Some(FormEntry::File(file)) => file,
        _ => return client_error("file required", 400, &request_id),
    };
    let content_type = file.type_();
    let size = file.size();
    if size > 10 * 1024 * 1024 {
        return client_error("file exceeds 10 MiB", 413, &request_id);
    }
    if !ALLOWED_UPLOAD_TYPES
        .iter()
        .any(|allowed| *allowed == content_type)
    {
        return client_error("unsupported file type", 415, &request_id);
    }
    let bytes = file.bytes().await?;
    if bytes.len() != size {
        return client_error("upload size mismatch", 400, &request_id);
    }
    if let Err(error) = verify_upload(&content_type, &bytes) {
        return client_error(error.to_string(), 415, &request_id);
    }
    let database = context.d1("DB")?;
    let used = starter_database::upload_bytes_used(&database, owner).await?;
    if !starter_domain::within_upload_quota(used, size as u64) {
        return client_error("upload quota exceeded", 413, &request_id);
    }
    let key = format!("attachments/{}/{}", owner, Uuid::new_v4());
    context
        .bucket("ATTACHMENTS")?
        .put(&key, bytes)
        .execute()
        .await?;
    starter_database::record_upload(
        &database,
        owner,
        size as u64,
        Date::now().as_millis() as i64,
    )
    .await?;
    let metadata = starter_contracts::AttachmentMetadataV1 {
        contract_version: CONTRACT_VERSION,
        key,
        content_type,
        size_bytes: size as u64,
    };
    let response = secured(
        Response::from_json(&metadata)?,
        &request_id,
        CachePolicy::Private,
    )?;
    attach_csrf_cookie(response, &csrf)
}
