//! Session repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    adapter::postgres::{Postgres, rows::SessionRow},
    app::port::{PortError, session_repository::SessionRepository},
    entity::session::Session,
};

#[async_trait]
impl SessionRepository for Postgres {
    async fn create(&self, session: &Session, token_hash: &str) -> Result<(), PortError> {
        sqlx::query(
            r"
            INSERT INTO sessions (id, user_id, expires_at, token_hash, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6)
            ",
        )
        .bind(&session.id)
        .bind(&session.user_id)
        .bind(session.expires_at)
        .bind(token_hash)
        .bind(&session.ip_address)
        .bind(&session.user_agent)
        .execute(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(())
    }

    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, PortError> {
        let row = sqlx::query_as::<_, SessionRow>(
            r"
            SELECT id, created_at, updated_at, user_id, expires_at, ip_address, user_agent
            FROM sessions
            WHERE token_hash = $1
            ",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(row.map(Into::into))
    }

    async fn touch(&self, id: &str, expires_at: DateTime<Utc>) -> Result<(), PortError> {
        sqlx::query("UPDATE sessions SET expires_at = $1 WHERE id = $2")
            .bind(expires_at)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(PortError::backend)?;

        Ok(())
    }

    async fn list(&self, user_id: &str) -> Result<Vec<Session>, PortError> {
        let rows = sqlx::query_as::<_, SessionRow>(
            r"
            SELECT id, created_at, updated_at, user_id, expires_at, ip_address, user_agent
            FROM sessions
            WHERE user_id = $1
            ORDER BY created_at DESC
            ",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn revoke(&self, id: &str) -> Result<(), PortError> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(PortError::backend)?;

        Ok(())
    }

    async fn revoke_others(&self, user_id: &str, keep_id: &str) -> Result<(), PortError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND id <> $2")
            .bind(user_id)
            .bind(keep_id)
            .execute(&self.pool)
            .await
            .map_err(PortError::backend)?;

        Ok(())
    }

    async fn revoke_all(&self, user_id: &str) -> Result<(), PortError> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(PortError::backend)?;

        Ok(())
    }
}
