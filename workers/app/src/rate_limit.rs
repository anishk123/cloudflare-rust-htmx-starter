use starter_contracts::ApiError;
use worker::{Response, Result, RouteContext};

use crate::http::{CachePolicy, secured};

const RATE_LIMIT_BINDING: &str = "RATE_LIMITER";

pub(crate) async fn enforce_rate_limit(
    context: &RouteContext<()>,
    key: String,
    request_id: &str,
) -> Result<Option<Response>> {
    let outcome = context.rate_limiter(RATE_LIMIT_BINDING)?.limit(key).await?;
    if outcome.success {
        return Ok(None);
    }
    let body = ApiError {
        code: "rate_limited".into(),
        message: "too many requests, please slow down".into(),
        request_id: request_id.into(),
    };
    let mut response = secured(
        Response::from_json(&body)?.with_status(429),
        request_id,
        CachePolicy::Private,
    )?;
    response.headers_mut().set("retry-after", "60")?;
    Ok(Some(response))
}
