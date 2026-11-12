//! Driven-port error type.
//!
//! Adapters report facts: a missing row, a uniqueness conflict, or a backend
//! failure. They do not decide what the API says.

/// A failure reported by a driven adapter.
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("not found")]
    NotFound,
    #[error("conflict")]
    Conflict,
    #[error(transparent)]
    Backend(anyhow::Error),
}

impl PortError {
    #[must_use]
    pub fn backend(error: impl Into<anyhow::Error>) -> Self {
        Self::Backend(error.into())
    }
}
