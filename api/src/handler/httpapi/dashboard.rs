//! Dashboard HTTP handler.

use axum::{Json, extract::State};

use crate::handler::httpapi::{
    dto::DashboardDto, error::ApiError, extractors::CurrentSession, state::AppState,
};

/// `GET /api/dashboard`
pub async fn get(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
) -> Result<Json<DashboardDto>, ApiError> {
    let data = state.dashboard.load(&session.user.actor()).await?;
    Ok(Json(DashboardDto::from(&data)))
}
