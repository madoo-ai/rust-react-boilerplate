//! HTTP middleware.

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderName, HeaderValue, Method, header},
    middleware::Next,
    response::Response,
};

use crate::handler::httpapi::{error::ApiError, state::AppState};

/// Reject cross-origin state-changing requests.
///
/// SameSite=Lax on the session cookie is the first line of defence. This
/// middleware adds an explicit Origin check for browsers that send one.
pub async fn same_origin(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if matches!(
        *request.method(),
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    ) && let Some(origin) = request.headers().get(header::ORIGIN)
    {
        let origin = origin
            .to_str()
            .map_err(|_| ApiError::bad_request("errors.invalidRequest"))?;

        if !state.origins.is_allowed(origin) {
            tracing::warn!(origin, "rejected cross-origin request");
            return Err(ApiError::from(crate::entity::error::DomainError::Forbidden));
        }
    }

    Ok(next.run(request).await)
}

/// Add the baseline security headers used by the source boilerplate.
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("SAMEORIGIN"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );

    response
}

/// Rate-limit mutating auth requests.
pub async fn rate_limit_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if *request.method() != Method::GET {
        let key = request
            .extensions()
            .get::<ConnectInfo<std::net::SocketAddr>>()
            .map_or_else(|| "unknown".to_owned(), |info| info.0.ip().to_string());

        if !state.limiter.allow(&key) {
            tracing::warn!(%key, "auth rate limit exceeded");
            return Err(ApiError::too_many_requests());
        }
    }

    Ok(next.run(request).await)
}
