//! Axum API for the Rust + React boilerplate.
//!
//! Hexagonal layout:
//!
//! ```text
//! handler/httpapi → app/usecase → app/port ← adapter/*
//!                        ↓
//!                     entity
//! ```
//!
//! `xmodule` is the composition root and the only module that knows concrete
//! adapters exist.

pub mod adapter;
pub mod app;
pub mod entity;
pub mod handler;
pub mod xmodule;

use axum::{
    Router,
    http::{Method, header},
    middleware as axum_middleware,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};

use crate::handler::httpapi::state::AppState;

/// Build the complete HTTP router.
///
/// `/api/*` routes always answer with JSON. Everything else is served from
/// the built React application, with a SPA fallback to `index.html`.
pub fn router(state: &AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT])
        .allow_origin(AllowOrigin::list(state.origins.allowed_origins()));

    Router::new()
        .merge(handler::httpapi::router(state.clone()))
        .fallback_service(state.static_files())
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            handler::httpapi::middleware::same_origin,
        ))
        .layer(axum_middleware::from_fn(
            handler::httpapi::middleware::security_headers,
        ))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(TraceLayer::new_for_http())
}
