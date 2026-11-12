//! One-time verification storage.

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::app::port::error::PortError;

/// Storage for email verification, password reset, 2FA challenge and trusted
/// device tokens.
#[async_trait]
pub trait VerificationRepository: Send + Sync {
    /// Replace the verification for an identifier.
    async fn replace(
        &self,
        identifier: &str,
        value_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), PortError>;

    /// Find a token and return the user id stored in its identifier without
    /// consuming it.
    async fn find(&self, prefix: &str, value_hash: &str) -> Result<Option<String>, PortError>;

    /// Consume a token and return the user id stored in its identifier.
    async fn consume(&self, prefix: &str, value_hash: &str) -> Result<Option<String>, PortError>;

    /// Check whether a token exists without consuming it.
    async fn exists(&self, identifier: &str, value_hash: &str) -> Result<bool, PortError>;

    /// Delete email, reset, 2FA and trusted-device tokens for a user.
    async fn delete_all_for_user(&self, user_id: &str) -> Result<(), PortError>;

    /// Delete 2FA challenges and trusted devices for a user.
    async fn delete_two_factor_artifacts(&self, user_id: &str) -> Result<(), PortError>;
}
