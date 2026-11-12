//! `PostgreSQL` adapters.
//!
//! Row types never escape this module: every implementation converts to
//! entities at the boundary. The pool is shared, but each repository trait is
//! implemented separately.

mod dashboard;
mod health;
mod roles;
mod rows;
mod seed;
mod sessions;
mod settings;
mod two_factor;
mod users;
mod verifications;

use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};

/// The `PostgreSQL` adapter.
#[derive(Debug, Clone)]
pub struct Postgres {
    pub(crate) pool: PgPool,
}

impl Postgres {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Connect to `PostgreSQL` with a bounded pool.
    ///
    /// # Errors
    ///
    /// Returns an error when the database is unreachable.
    pub async fn connect(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(database_url)
            .await
            .context("could not connect to PostgreSQL")?;

        Ok(Self { pool })
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
