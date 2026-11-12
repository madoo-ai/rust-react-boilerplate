//! Dashboard read model.

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{app::port::error::PortError, entity::dashboard::DashboardSnapshot};

/// Storage for the dashboard aggregates.
#[async_trait]
pub trait DashboardRepository: Send + Sync {
    /// Load counts, role distribution, recent users and signup times.
    async fn snapshot(&self, since: DateTime<Utc>) -> Result<DashboardSnapshot, PortError>;
}
