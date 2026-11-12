//! Outbound mail.

use async_trait::async_trait;

use crate::app::port::error::PortError;

/// The two transactional auth messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthEmail {
    VerifyEmail,
    ResetPassword,
}

/// Sends transactional auth mail. Implementations log the link instead of
/// failing when SMTP is not configured.
#[async_trait]
pub trait Mailer: Send + Sync {
    /// Send an auth email.
    async fn send_auth_email(
        &self,
        kind: AuthEmail,
        to: &str,
        name: &str,
        url: &str,
    ) -> Result<(), PortError>;
}
