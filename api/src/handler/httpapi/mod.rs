//! The HTTP driving adapter.
//!
//! Handlers validate transport-level input, call use cases and map entities to
//! JSON. They do not contain business rules.

pub mod auth;
pub mod cookies;
pub mod dashboard;
pub mod dto;
pub mod error;
pub mod extractors;
pub mod middleware;
pub mod roles;
pub mod settings;
pub mod state;
pub mod users;

use axum::{
    Json, Router,
    http::StatusCode,
    middleware as axum_middleware,
    response::{IntoResponse, Response},
    routing::{any, delete, get, patch, post, put},
};

use crate::handler::httpapi::{state::AppState, users as user_handlers};

/// Build the API router.
pub fn router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/api/auth/session", get(auth::session))
        .route("/api/auth/sign-up/email", post(auth::sign_up))
        .route("/api/auth/sign-in/email", post(auth::sign_in))
        .route("/api/auth/sign-out", post(auth::sign_out))
        .route("/api/auth/verify-email", post(auth::verify_email))
        .route(
            "/api/auth/send-verification-email",
            post(auth::send_verification_email),
        )
        .route("/api/auth/forgot-password", post(auth::forgot_password))
        .route("/api/auth/reset-password", post(auth::reset_password))
        .route("/api/auth/change-password", post(auth::change_password))
        .route("/api/auth/confirm-password", post(auth::confirm_password))
        .route("/api/auth/sessions", get(auth::list_sessions))
        .route(
            "/api/auth/sessions/revoke-others",
            post(auth::revoke_other_sessions),
        )
        .route("/api/auth/sessions/{id}", delete(auth::revoke_session))
        .route("/api/auth/two-factor/enable", post(auth::enable_two_factor))
        .route(
            "/api/auth/two-factor/verify",
            post(auth::verify_two_factor_enable),
        )
        .route(
            "/api/auth/two-factor/disable",
            post(auth::disable_two_factor),
        )
        .route(
            "/api/auth/two-factor/backup-codes",
            post(auth::regenerate_backup_codes),
        )
        .route(
            "/api/auth/two-factor/verify-totp",
            post(auth::verify_two_factor_totp),
        )
        .route(
            "/api/auth/two-factor/verify-backup-code",
            post(auth::verify_two_factor_backup_code),
        )
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit_auth,
        ));

    Router::new()
        .merge(auth_routes)
        .route(
            "/api/users",
            get(user_handlers::list).post(user_handlers::create),
        )
        .route("/api/users/meta", get(user_handlers::meta))
        .route(
            "/api/users/registration-approval",
            put(user_handlers::set_registration_approval),
        )
        .route(
            "/api/users/{id}",
            get(user_handlers::get)
                .patch(user_handlers::update)
                .delete(user_handlers::delete),
        )
        .route("/api/users/{id}/verify", post(user_handlers::verify))
        .route("/api/roles", get(roles::list).post(roles::create))
        .route(
            "/api/roles/{id}",
            patch(roles::update).delete(roles::delete),
        )
        .route("/api/settings/profile", patch(settings::update_profile))
        .route("/api/settings/account", delete(settings::delete_account))
        .route("/api/dashboard", get(dashboard::get))
        .route("/api/health", get(health))
        .route("/api", any(api_not_found))
        .route("/api/{*rest}", any(api_not_found))
        .with_state(state)
}

async fn api_not_found() -> crate::handler::httpapi::error::ApiError {
    crate::entity::error::DomainError::NotFound.into()
}

async fn health(axum::extract::State(state): axum::extract::State<AppState>) -> Response {
    match state.health.check().await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))).into_response(),
        Err(error) => {
            tracing::error!(error = ?error, "health check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "status": "error" })),
            )
                .into_response()
        }
    }
}
