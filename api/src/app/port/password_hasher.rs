//! Password hashing.

use async_trait::async_trait;

use crate::app::port::error::PortError;

/// Hashes and verifies passwords. Implementations must use a memory-hard
/// password hash with a per-password salt.
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Hash a new password.
    async fn hash(&self, password: &str) -> Result<String, PortError>;

    /// Verify a password against a stored hash.
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, PortError>;
}
