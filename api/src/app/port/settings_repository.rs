//! Settings persistence.

use async_trait::async_trait;

use crate::app::port::error::PortError;

/// Key/value application settings.
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Read a setting.
    async fn get(&self, key: &str) -> Result<Option<String>, PortError>;

    /// Upsert a setting.
    async fn set(&self, id: &str, key: &str, value: &str) -> Result<(), PortError>;
}
