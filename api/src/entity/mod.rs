//! Pure domain types and rules.
//!
//! This layer must not import `axum`, `sqlx`, mail clients or any adapter.
//! State transitions and policies live here so they can be tested without a
//! database or an HTTP server.

pub mod authorization;
pub mod dashboard;
pub mod error;
pub mod permission;
pub mod role;
pub mod session;
pub mod user;
