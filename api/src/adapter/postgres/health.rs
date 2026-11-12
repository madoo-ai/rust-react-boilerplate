//! `PostgreSQL` health probe.

use async_trait::async_trait;

use crate::{
    adapter::postgres::Postgres,
    app::port::{HealthProbe, PortError},
};

#[async_trait]
impl HealthProbe for Postgres {
    async fn check(&self) -> Result<(), PortError> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(PortError::backend)?;

        Ok(())
    }
}
