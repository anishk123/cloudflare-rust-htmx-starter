use worker::{Error, Request, Response, Result, Url};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CachePolicy {
    Private,
    PublicShort,
    PublicDiscovery,
}

impl CachePolicy {
    pub(crate) const fn browser_control(self) -> &'static str {
        match self {
            Self::Private => "no-store",
            Self::PublicShort => "public, max-age=60, stale-while-revalidate=300",
            Self::PublicDiscovery => "public, max-age=300, stale-while-revalidate=3600",
        }
    }

    pub(crate) const fn cdn_control(self) -> Option<&'static str> {
        match self {
            Self::Private => None,
            Self::PublicShort | Self::PublicDiscovery => {
                Some("public, max-age=3600, stale-while-revalidate=86400")
            }
        }
    }
}

pub(crate) fn secured(
    mut response: Response,
    request_id: &str,
    policy: CachePolicy,
) -> Result<Response> {
    let headers = response.headers_mut();
    headers.set("x-request-id", request_id)?;
    headers.set("x-content-type-options", "nosniff")?;
    headers.set("referrer-policy", "strict-origin-when-cross-origin")?;
    headers.set(
        "permissions-policy",
        "camera=(), microphone=(), geolocation=()",
    )?;
    headers.set("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'")?;
    headers.set("cache-control", policy.browser_control())?;
    if let Some(value) = policy.cdn_control() {
        headers.set("cloudflare-cdn-cache-control", value)?;
    }
    Ok(response)
}

pub(crate) fn text(body: &str, content_type: &str) -> Result<Response> {
    let mut response = Response::ok(body)?;
    response.headers_mut().set("content-type", content_type)?;
    Ok(response)
}

pub(crate) fn not_found(request_id: &str, policy: CachePolicy) -> Result<Response> {
    secured(Response::error("not found", 404)?, request_id, policy)
}

pub(crate) fn client_error(
    message: impl Into<String>,
    status: u16,
    request_id: &str,
) -> Result<Response> {
    secured(
        Response::error(message, status)?,
        request_id,
        CachePolicy::Private,
    )
}

pub(crate) fn is_htmx(req: &Request) -> bool {
    req.headers().get("HX-Request").ok().flatten().as_deref() == Some("true")
}

/// Build a mutable 303 response. Fetch redirect responses have immutable
/// headers in Workers, which prevents applying the security policy afterward.
pub(crate) fn redirect_303(
    location: Url,
    request_id: &str,
    policy: CachePolicy,
) -> Result<Response> {
    let mut response = Response::empty()?.with_status(303);
    response.headers_mut().set("location", location.as_str())?;
    secured(response, request_id, policy)
}

pub(crate) fn redirect_home(req: &Request, request_id: &str) -> Result<Response> {
    redirect_303(req.url()?.join("/")?, request_id, CachePolicy::Private)
}

pub(crate) fn validation_response(
    message: &str,
    request_id: &str,
    htmx: bool,
    origin: &str,
) -> Result<Response> {
    let body = starter_templates::render_error_fragment(message)
        .map_err(|error| Error::RustError(format!("template render failed: {error}")))?;
    let mut response = if htmx {
        Response::from_html(body)?
    } else {
        Response::from_html(
            starter_templates::render_error_page("Validation error", message, origin)
                .map_err(|error| Error::RustError(format!("template render failed: {error}")))?,
        )?
    };
    response = response.with_status(422);
    secured(response, request_id, CachePolicy::Private)
}

#[cfg(test)]
mod tests {
    use super::CachePolicy;

    #[test]
    fn private_policy_is_never_cacheable() {
        assert_eq!(CachePolicy::Private.browser_control(), "no-store");
        assert_eq!(CachePolicy::Private.cdn_control(), None);
    }

    #[test]
    fn published_policy_separates_browser_and_edge_freshness() {
        assert_eq!(
            CachePolicy::PublicShort.browser_control(),
            "public, max-age=60, stale-while-revalidate=300"
        );
        assert_eq!(
            CachePolicy::PublicShort.cdn_control(),
            Some("public, max-age=3600, stale-while-revalidate=86400")
        );
    }

    #[test]
    fn discovery_policy_is_explicit() {
        assert_eq!(
            CachePolicy::PublicDiscovery.browser_control(),
            "public, max-age=300, stale-while-revalidate=3600"
        );
    }
}
