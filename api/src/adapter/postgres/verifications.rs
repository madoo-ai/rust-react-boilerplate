//! Verification token repository implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ulid::Ulid;

use crate::{
    adapter::postgres::Postgres,
    app::port::{PortError, verification_repository::VerificationRepository},
};

#[async_trait]
impl VerificationRepository for Postgres {
    async fn replace(
        &self,
        identifier: &str,
        value_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), PortError> {
        let mut transaction = self.pool.begin().await.map_err(PortError::backend)?;

        sqlx::query("DELETE FROM verifications WHERE identifier = $1")
            .bind(identifier)
            .execute(&mut *transaction)
            .await
            .map_err(PortError::backend)?;

        sqlx::query(
            r"
            INSERT INTO verifications (id, identifier, value, expires_at)
            VALUES ($1, $2, $3, $4)
            ",
        )
        .bind(Ulid::new().to_string())
        .bind(identifier)
        .bind(value_hash)
        .bind(expires_at)
        .execute(&mut *transaction)
        .await
        .map_err(PortError::backend)?;

        transaction.commit().await.map_err(PortError::backend)?;

        Ok(())
    }

    async fn find(&self, prefix: &str, value_hash: &str) -> Result<Option<String>, PortError> {
        let identifier = sqlx::query_scalar::<_, String>(
            r"
            SELECT identifier FROM verifications
            WHERE value = $1 AND identifier LIKE $2 AND expires_at > now()
            LIMIT 1
            ",
        )
        .bind(value_hash)
        .bind(format!("{prefix}%"))
        .fetch_optional(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(identifier.and_then(|value| value.strip_prefix(prefix).map(str::to_owned)))
    }

    async fn consume(&self, prefix: &str, value_hash: &str) -> Result<Option<String>, PortError> {
        let identifier = sqlx::query_scalar::<_, String>(
            r"
            DELETE FROM verifications
            WHERE id = (
                SELECT id FROM verifications
                WHERE value = $1 AND identifier LIKE $2 AND expires_at > now()
                LIMIT 1
            )
            RETURNING identifier
            ",
        )
        .bind(value_hash)
        .bind(format!("{prefix}%"))
        .fetch_optional(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(identifier.and_then(|value| value.strip_prefix(prefix).map(str::to_owned)))
    }

    async fn exists(&self, identifier: &str, value_hash: &str) -> Result<bool, PortError> {
        sqlx::query_scalar::<_, bool>(
            r"
            SELECT EXISTS(
                SELECT 1 FROM verifications
                WHERE identifier = $1 AND value = $2 AND expires_at > now()
            )
            ",
        )
        .bind(identifier)
        .bind(value_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(PortError::backend)
    }

    async fn delete_all_for_user(&self, user_id: &str) -> Result<(), PortError> {
        sqlx::query(
            r"
            DELETE FROM verifications
            WHERE identifier IN ($1, $2, $3, $4)
            ",
        )
        .bind(format!("email-verification:{user_id}"))
        .bind(format!("reset-password:{user_id}"))
        .bind(format!("2fa-challenge:{user_id}"))
        .bind(format!("trusted-device:{user_id}"))
        .execute(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(())
    }

    async fn delete_two_factor_artifacts(&self, user_id: &str) -> Result<(), PortError> {
        sqlx::query("DELETE FROM verifications WHERE identifier IN ($1, $2)")
            .bind(format!("2fa-challenge:{user_id}"))
            .bind(format!("trusted-device:{user_id}"))
            .execute(&self.pool)
            .await
            .map_err(PortError::backend)?;

        Ok(())
    }
}
