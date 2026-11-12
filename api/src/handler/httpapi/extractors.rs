//! HTTP extractors.

use std::collections::BTreeMap;

use axum::{
    Json,
    extract::{ConnectInfo, FromRequest, FromRequestParts, Request},
    http::request::Parts,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::{
    entity::session::AuthenticatedSession,
    handler::httpapi::{
        error::ApiError,
        state::{AppState, unauthorized},
    },
};

/// A required session. Missing or blocked sessions yield 401.
#[derive(Debug)]
pub struct CurrentSession(pub AuthenticatedSession);

impl FromRequestParts<AppState> for CurrentSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = state.session_token(&jar).ok_or_else(unauthorized)?;

        let session = state
            .auth
            .resolve_session(&token)
            .await
            .map_err(ApiError::from)?;

        session.map(CurrentSession).ok_or_else(unauthorized)
    }
}

/// An optional session. Expired or missing sessions yield `None`.
#[derive(Debug)]
pub struct OptionalSession(pub Option<AuthenticatedSession>);

impl FromRequestParts<AppState> for OptionalSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let Some(token) = state.session_token(&jar) else {
            return Ok(Self(None));
        };

        let session = state
            .auth
            .resolve_session(&token)
            .await
            .map_err(ApiError::from)?;

        Ok(Self(session))
    }
}

/// Client IP and user agent for session rows and rate limiting.
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

impl FromRequestParts<AppState> for ClientInfo {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ip = parts
            .extensions
            .get::<ConnectInfo<std::net::SocketAddr>>()
            .map(|info| info.0.ip().to_string());

        let user_agent = parts
            .headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.chars().take(400).collect());

        Ok(Self { ip, user_agent })
    }
}

/// JSON extractor that maps parse failures to the shared error envelope.
#[derive(Debug, Clone)]
pub struct JsonBody<T>(pub T);

impl<T, S> FromRequest<S> for JsonBody<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                tracing::debug!(error = %rejection, "invalid JSON request");
                ApiError::bad_request("errors.invalidRequest")
            })?;

        Ok(Self(value))
    }
}

/// JSON extractor that runs `validator` before the handler sees the value.
#[derive(Debug, Clone)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                tracing::debug!(error = %rejection, "invalid JSON request");
                ApiError::bad_request("errors.invalidRequest")
            })?;

        if let Err(errors) = value.validate() {
            let mut fields = BTreeMap::new();

            for (field, entries) in errors.field_errors() {
                let message = entries
                    .first()
                    .and_then(|entry| entry.message.as_deref())
                    .unwrap_or("validation.required");
                fields.insert(field.to_string(), message.to_owned());
            }

            return Err(ApiError::from(
                crate::entity::error::DomainError::Validation(fields),
            ));
        }

        Ok(Self(value))
    }
}

/// Empty success envelope.
#[derive(Debug, serde::Serialize)]
pub struct OkResponse {
    pub ok: bool,
}

impl Default for OkResponse {
    fn default() -> Self {
        Self { ok: true }
    }
}

impl IntoResponse for OkResponse {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
