//! Application-level errors.
//!
//! Use cases return `AppError`: a domain rule failure or an unexpected
//! infrastructure failure. Only the driving adapter decides how to expose it.

use crate::{app::port::error::PortError, entity::error::DomainError};

/// A use-case failure.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<PortError> for AppError {
    fn from(error: PortError) -> Self {
        match error {
            PortError::NotFound => Self::Domain(DomainError::NotFound),
            PortError::Conflict => Self::Domain(DomainError::Conflict),
            PortError::Backend(error) => Self::Unexpected(error),
        }
    }
}

impl AppError {
    #[must_use]
    pub fn unexpected(error: impl Into<anyhow::Error>) -> Self {
        Self::Unexpected(error.into())
    }
}
