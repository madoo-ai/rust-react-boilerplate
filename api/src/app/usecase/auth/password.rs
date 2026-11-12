//! Password reset, change and recent confirmation.

use chrono::Duration;

use crate::{
    app::{
        error::AppError,
        port::mailer::AuthEmail,
        usecase::auth::{AuthService, RESET_HOURS},
    },
    entity::{
        error::DomainError,
        session::AuthenticatedSession,
        user::{normalize_email, validate_email, validate_password},
    },
};

/// Reset a password with a one-time token.
#[derive(Debug, Clone)]
pub struct ResetPasswordCommand {
    pub token: String,
    pub new_password: String,
}

/// Change the current user's password.
#[derive(Debug, Clone)]
pub struct ChangePasswordCommand {
    pub current_password: String,
    pub new_password: String,
}

impl AuthService {
    /// Request a reset link. Always succeeds for unknown accounts so the
    /// endpoint cannot be used to enumerate users.
    ///
    /// # Errors
    ///
    /// Returns a validation error for a malformed email or an infrastructure
    /// error when the token cannot be stored.
    pub async fn request_password_reset(&self, email: &str) -> Result<(), AppError> {
        let email = normalize_email(email);
        validate_email(&email)?;

        let Some(user) = self.users.find_by_email(&email).await? else {
            return Ok(());
        };

        let token = self.tokens.generate();
        let expires_at = self.now() + Duration::hours(RESET_HOURS);

        self.verifications
            .replace(
                &format!("reset-password:{}", user.user.id),
                &self.tokens.hash(&token),
                expires_at,
            )
            .await?;

        let url = format!(
            "{}/reset-password?token={token}",
            self.config.web_url.trim_end_matches('/')
        );

        if let Err(error) = self
            .mailer
            .send_auth_email(
                AuthEmail::ResetPassword,
                &user.user.email,
                &user.user.name,
                &url,
            )
            .await
        {
            tracing::warn!(error = ?error, email, "could not send reset email");
        }

        Ok(())
    }

    /// Reset a password and revoke every session for the user.
    ///
    /// # Errors
    ///
    /// Returns `InvalidToken` for an expired link, a validation error, or an
    /// infrastructure error.
    pub async fn reset_password(&self, command: ResetPasswordCommand) -> Result<(), AppError> {
        validate_password(&command.new_password)?;

        let Some(user_id) = self
            .verifications
            .consume("reset-password:", &self.tokens.hash(&command.token))
            .await?
        else {
            return Err(DomainError::InvalidToken.into());
        };

        let password_hash = self.hasher.hash(&command.new_password).await?;

        self.users.update_password(&user_id, &password_hash).await?;
        self.sessions.revoke_all(&user_id).await?;

        Ok(())
    }

    /// Change the current user's password and keep only this session.
    ///
    /// # Errors
    ///
    /// Returns `IncorrectPassword`, a validation error, or an infrastructure
    /// error.
    pub async fn change_password(
        &self,
        session: &AuthenticatedSession,
        command: ChangePasswordCommand,
    ) -> Result<(), AppError> {
        validate_password(&command.new_password)?;

        let Some(credential) = self
            .users
            .find_credentials_by_id(&session.user.user.id)
            .await?
        else {
            return Err(DomainError::UserNotFound.into());
        };

        let Some(current_hash) = credential.password_hash.as_deref() else {
            return Err(DomainError::IncorrectPassword.into());
        };

        if !self
            .hasher
            .verify(&command.current_password, current_hash)
            .await?
        {
            return Err(DomainError::IncorrectPassword.into());
        }

        let password_hash = self.hasher.hash(&command.new_password).await?;

        self.users
            .update_password(&session.user.user.id, &password_hash)
            .await?;
        self.sessions
            .revoke_others(&session.user.user.id, &session.session.id)
            .await?;

        Ok(())
    }

    /// Verify the current password for a sensitive action.
    ///
    /// # Errors
    ///
    /// Returns `IncorrectPassword` or an infrastructure error.
    pub async fn confirm_password(
        &self,
        session: &AuthenticatedSession,
        password: &str,
    ) -> Result<(), AppError> {
        let Some(credential) = self
            .users
            .find_credentials_by_id(&session.user.user.id)
            .await?
        else {
            return Err(DomainError::UserNotFound.into());
        };

        let Some(hash) = credential.password_hash.as_deref() else {
            return Err(DomainError::IncorrectPassword.into());
        };

        if self.hasher.verify(password, hash).await? {
            Ok(())
        } else {
            Err(DomainError::IncorrectPassword.into())
        }
    }
}
