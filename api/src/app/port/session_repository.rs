//! Session persistence.

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{app::port::error::PortError, entity::session::Session};

/// Storage for opaque sessions.
#[async_trait]
pub trait SessionRepository: Send + Sync {
    /// Insert a session row. The token hash is already computed.
    async fn create(&self, session: &Session, token_hash: &str) -> Result<(), PortError>;

    /// Find a session by token hash.
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, PortError>;

    /// Extend the session expiry and update its timestamp.
    async fn touch(&self, id: &str, expires_at: DateTime<Utc>) -> Result<(), PortError>;

    /// List a user's sessions, newest first.
    async fn list(&self, user_id: &str) -> Result<Vec<Session>, PortError>;

    /// Delete one session.
    async fn revoke(&self, id: &str) -> Result<(), PortError>;

    /// Delete every session for a user except one.
    async fn revoke_others(&self, user_id: &str, keep_id: &str) -> Result<(), PortError>;

    /// Delete every session for a user.
    async fn revoke_all(&self, user_id: &str) -> Result<(), PortError>;
}
