//! Two-factor enrollment persistence.

use async_trait::async_trait;

use crate::{app::port::error::PortError, entity::session::TwoFactor};

/// Storage for TOTP secrets and backup codes.
#[async_trait]
pub trait TwoFactorRepository: Send + Sync {
    /// Load the enrollment for a user.
    async fn find_by_user(&self, user_id: &str) -> Result<Option<TwoFactor>, PortError>;

    /// Insert or replace the enrollment.
    async fn save(&self, two_factor: &TwoFactor) -> Result<(), PortError>;

    /// Mark an enrollment as verified.
    async fn set_verified(&self, user_id: &str, verified: bool) -> Result<(), PortError>;

    /// Replace the hashed backup codes.
    async fn update_backup_codes(&self, user_id: &str, codes: &[String]) -> Result<(), PortError>;

    /// Delete the enrollment.
    async fn delete(&self, user_id: &str) -> Result<(), PortError>;
}
