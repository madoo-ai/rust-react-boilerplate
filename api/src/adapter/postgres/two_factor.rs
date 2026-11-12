//! Two-factor repository implementation.

use async_trait::async_trait;

use crate::{
    adapter::postgres::{Postgres, rows::TwoFactorRow},
    app::port::{PortError, two_factor_repository::TwoFactorRepository},
    entity::session::TwoFactor,
};

#[async_trait]
impl TwoFactorRepository for Postgres {
    async fn find_by_user(&self, user_id: &str) -> Result<Option<TwoFactor>, PortError> {
        let row = sqlx::query_as::<_, TwoFactorRow>(
            r"
            SELECT id, user_id, secret, backup_codes, verified,
                   failed_verification_count, locked_until
            FROM two_factors
            WHERE user_id = $1
            ",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(row.map(Into::into))
    }

    async fn save(&self, two_factor: &TwoFactor) -> Result<(), PortError> {
        sqlx::query(
            r"
            INSERT INTO two_factors (
                id, user_id, secret, backup_codes, verified,
                failed_verification_count, locked_until
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id) DO UPDATE SET
                secret = EXCLUDED.secret,
                backup_codes = EXCLUDED.backup_codes,
                verified = EXCLUDED.verified,
                failed_verification_count = EXCLUDED.failed_verification_count,
                locked_until = EXCLUDED.locked_until
            ",
        )
        .bind(&two_factor.id)
        .bind(&two_factor.user_id)
        .bind(&two_factor.secret)
        .bind(sqlx::types::Json(&two_factor.backup_codes))
        .bind(two_factor.verified)
        .bind(two_factor.failed_verification_count)
        .bind(two_factor.locked_until)
        .execute(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(())
    }

    async fn set_verified(&self, user_id: &str, verified: bool) -> Result<(), PortError> {
        sqlx::query("UPDATE two_factors SET verified = $1 WHERE user_id = $2")
            .bind(verified)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(PortError::backend)?;

        Ok(())
    }

    async fn update_backup_codes(&self, user_id: &str, codes: &[String]) -> Result<(), PortError> {
        sqlx::query("UPDATE two_factors SET backup_codes = $1 WHERE user_id = $2")
            .bind(sqlx::types::Json(codes))
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(PortError::backend)?;

        Ok(())
    }

    async fn delete(&self, user_id: &str) -> Result<(), PortError> {
        sqlx::query("DELETE FROM two_factors WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(PortError::backend)?;

        Ok(())
    }
}
