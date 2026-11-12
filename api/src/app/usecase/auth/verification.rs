//! Email verification.

use crate::{
    app::{error::AppError, usecase::auth::AuthService},
    entity::{error::DomainError, session::AuthenticatedSession},
};

impl AuthService {
    /// Consume an email verification token.
    ///
    /// # Errors
    ///
    /// Returns `InvalidToken` for an expired or unknown link, or an
    /// infrastructure error.
    pub async fn verify_email(&self, token: &str) -> Result<(), AppError> {
        let Some(user_id) = self
            .verifications
            .consume("email-verification:", &self.tokens.hash(token))
            .await?
        else {
            return Err(DomainError::InvalidToken.into());
        };

        self.users.mark_email_verified(&user_id).await?;

        Ok(())
    }

    /// Send another verification email for the current user.
    ///
    /// # Errors
    ///
    /// Returns an infrastructure error when the token cannot be stored.
    /// Returns `false` when the email is already verified.
    pub async fn resend_verification(
        &self,
        session: &AuthenticatedSession,
    ) -> Result<bool, AppError> {
        if session.user.user.email_verified {
            return Ok(false);
        }

        self.send_verification_email(
            &session.user.user.id,
            &session.user.user.email,
            &session.user.user.name,
        )
        .await;

        Ok(true)
    }
}
