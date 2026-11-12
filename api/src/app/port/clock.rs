//! Clock port for deterministic use-case tests.

use chrono::{DateTime, Utc};

/// Returns the current time. Use cases never call `Utc::now` directly.
pub trait Clock: Send + Sync {
    /// Current UTC time.
    fn now(&self) -> DateTime<Utc>;
}
