//! Readiness probe.

use async_trait::async_trait;

use crate::app::port::error::PortError;

/// A dependency health check used by `/api/health`.
#[async_trait]
pub trait HealthProbe: Send + Sync {
    /// Check that the backing dependency is reachable.
    async fn check(&self) -> Result<(), PortError>;
}
