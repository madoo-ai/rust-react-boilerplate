//! Sign-in, the pending/suspended gate and the two-factor challenge.

use chrono::Duration;

use crate::{
    app::{
        error::AppError,
        usecase::auth::{AuthService, CHALLENGE_MINUTES, TRUSTED_DEVICE_DAYS},
    },
    entity::{
        error::DomainError,
        session::AuthenticatedSession,
        user::{normalize_email, validate_email},
    },
};

/// Credentials and request metadata.
#[derive(Debug, Clone)]
pub struct SignInCommand {
    pub email: String,
    pub password: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub trusted_device_token: Option<String>,
}

/// Either a session or a 2FA challenge.
#[derive(Debug, Clone)]
pub enum SignInResult {
    SignedIn {
        session: Box<AuthenticatedSession>,
        token: String,
    },
    TwoFactorRequired {
        challenge_token: String,
    },
}

impl AuthService {
    /// Verify credentials and start a session or a two-factor challenge.
    ///
    /// # Errors
    ///
    /// Returns `InvalidCredentials`, `AccountPending`, `AccountSuspended` or
    /// an infrastructure error.
    pub async fn sign_in(&self, command: SignInCommand) -> Result<SignInResult, AppError> {
        let email = normalize_email(&command.email);
        validate_email(&email)?;

        let Some(credential) = self.users.find_credentials_by_email(&email).await? else {
            // Spend comparable time on unknown accounts.
            let _ = self.hasher.hash(&command.password).await;
            return Err(DomainError::InvalidCredentials.into());
        };

        let Some(password_hash) = credential.password_hash.as_deref() else {
            let _ = self.hasher.hash(&command.password).await;
            return Err(DomainError::InvalidCredentials.into());
        };

        if !self.hasher.verify(&command.password, password_hash).await? {
            return Err(DomainError::InvalidCredentials.into());
        }

        if !credential.user.user.status.is_active() {
            return Err(match credential.user.user.status {
                crate::entity::user::UserStatus::Pending => DomainError::AccountPending.into(),
                crate::entity::user::UserStatus::Suspended => DomainError::AccountSuspended.into(),
                crate::entity::user::UserStatus::Active => DomainError::AccountInactive.into(),
            });
        }

        if credential.user.user.two_factor_enabled
            && !self
                .is_trusted_device(
                    &credential.user.user.id,
                    command.trusted_device_token.as_deref(),
                )
                .await?
        {
            let token = self.tokens.generate();
            let expires_at = self.now() + Duration::minutes(CHALLENGE_MINUTES);

            self.verifications
                .replace(
                    &format!("2fa-challenge:{}", credential.user.user.id),
                    &self.tokens.hash(&token),
                    expires_at,
                )
                .await?;

            return Ok(SignInResult::TwoFactorRequired {
                challenge_token: token,
            });
        }

        let (session, token) = self
            .issue_session_for_user(&credential.user, command.ip_address, command.user_agent)
            .await?;

        Ok(SignInResult::SignedIn {
            session: Box::new(session),
            token,
        })
    }

    async fn is_trusted_device(
        &self,
        user_id: &str,
        token: Option<&str>,
    ) -> Result<bool, AppError> {
        let Some(token) = token else {
            return Ok(false);
        };

        Ok(self
            .verifications
            .exists(
                &format!("trusted-device:{user_id}"),
                &self.tokens.hash(token),
            )
            .await?)
    }

    /// Trust a device after a successful two-factor check.
    ///
    /// # Errors
    ///
    /// Returns an error when the token cannot be stored.
    pub(crate) async fn create_trusted_device(&self, user_id: &str) -> Result<String, AppError> {
        let token = self.tokens.generate();
        let expires_at = self.now() + Duration::days(TRUSTED_DEVICE_DAYS);

        self.verifications
            .replace(
                &format!("trusted-device:{user_id}"),
                &self.tokens.hash(&token),
                expires_at,
            )
            .await?;

        Ok(token)
    }
}
