//! Settings repository implementation.

use async_trait::async_trait;

use crate::{
    adapter::postgres::Postgres,
    app::port::{PortError, settings_repository::SettingsRepository},
};

#[async_trait]
impl SettingsRepository for Postgres {
    async fn get(&self, key: &str) -> Result<Option<String>, PortError> {
        sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(PortError::backend)
    }

    async fn set(&self, id: &str, key: &str, value: &str) -> Result<(), PortError> {
        sqlx::query(
            r"
            INSERT INTO settings (id, key, value)
            VALUES ($1, $2, $3)
            ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value
            ",
        )
        .bind(id)
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(())
    }
}
