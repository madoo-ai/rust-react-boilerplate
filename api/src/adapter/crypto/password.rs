//! Argon2id password hasher.

use anyhow::{Context, Result};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher as _, PasswordVerifier as _, phc::PasswordHash},
};
use async_trait::async_trait;

use crate::app::port::{PasswordHasher as PasswordHasherPort, PortError};

/// Argon2id hashing offloaded to a blocking thread.
#[derive(Debug, Clone, Copy, Default)]
pub struct Argon2Hasher;

#[async_trait]
impl PasswordHasherPort for Argon2Hasher {
    async fn hash(&self, password: &str) -> Result<String, PortError> {
        let password = password.to_owned();

        tokio::task::spawn_blocking(move || hash_sync(&password))
            .await
            .map_err(PortError::backend)?
            .map_err(PortError::backend)
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool, PortError> {
        let password = password.to_owned();
        let hash = hash.to_owned();

        tokio::task::spawn_blocking(move || verify_sync(&password, &hash))
            .await
            .map_err(PortError::backend)?
            .map_err(PortError::backend)
    }
}

/// Synchronous hash helper for the seed and adapter tests.
///
/// # Errors
///
/// Returns an error when Argon2 cannot produce a PHC string.
pub fn hash_sync(password: &str) -> Result<String> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .context("could not hash password")
}

fn verify_sync(password: &str, hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(hash).context("stored password hash is malformed")?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hashes_and_verifies() {
        let hasher = Argon2Hasher;
        let hash = hasher
            .hash("correct horse battery staple")
            .await
            .expect("hash");
        assert!(
            hasher
                .verify("correct horse battery staple", &hash)
                .await
                .expect("verify")
        );
        assert!(!hasher.verify("wrong", &hash).await.expect("verify"));
    }
}
