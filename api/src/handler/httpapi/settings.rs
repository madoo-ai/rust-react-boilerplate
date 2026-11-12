//! Settings HTTP handlers.

use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::{
    app::usecase::settings::UpdateProfileCommand,
    handler::httpapi::{
        cookies,
        error::ApiError,
        extractors::{CurrentSession, JsonBody, OkResponse},
        state::AppState,
    },
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRequest {
    name: String,
    email: String,
    phone: Option<String>,
    job_title: Option<String>,
    department: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResponse {
    email_changed: bool,
}

/// `PATCH /api/settings/profile`
pub async fn update_profile(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<ProfileRequest>,
) -> Result<Json<ProfileResponse>, ApiError> {
    let result = state
        .settings
        .update_profile(
            &session,
            UpdateProfileCommand {
                name: request.name,
                email: request.email,
                phone: request.phone,
                job_title: request.job_title,
                department: request.department,
            },
        )
        .await?;

    Ok(Json(ProfileResponse {
        email_changed: result.email_changed,
    }))
}

/// `DELETE /api/settings/account`
pub async fn delete_account(
    State(state): State<AppState>,
    jar: CookieJar,
    CurrentSession(session): CurrentSession,
) -> Result<Response, ApiError> {
    let confirmed = state
        .confirm_value(&jar)
        .is_some_and(|value| state.auth.has_recent_confirmation(&value));

    state.settings.delete_account(&session, confirmed).await?;

    let jar = cookies::clear_session(jar, &state);
    Ok((jar, Json(OkResponse::default())).into_response())
}
