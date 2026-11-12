//! Sign-up and the registration-approval gate.

use chrono::Duration;

use crate::{
    app::{
        error::AppError,
        port::{error::PortError, mailer::AuthEmail},
        usecase::auth::{AuthService, VERIFICATION_HOURS},
    },
    entity::{
        error::DomainError,
        user::{
            NewUser, UserStatus, UserWithRole, normalize_email, validate_email, validate_name,
            validate_password,
        },
    },
};

/// Public registration input.
#[derive(Debug, Clone)]
pub struct SignUpCommand {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// Result of a successful sign-up. No session is created.
#[derive(Debug, Clone)]
pub struct SignUpResult {
    pub user: UserWithRole,
}

impl AuthService {
    /// Create an account and send a verification email.
    ///
    /// # Errors
    ///
    /// Returns a validation error, `EmailInUse`, or an infrastructure error.
    pub async fn sign_up(&self, command: SignUpCommand) -> Result<SignUpResult, AppError> {
        let name = command.name.trim().to_owned();
        let email = normalize_email(&command.email);

        validate_name(&name)?;
        validate_email(&email)?;
        validate_password(&command.password)?;

        if self.users.email_exists(&email, None).await? {
            return Err(DomainError::EmailInUse.into());
        }

        let role = self.default_role().await?;
        let approval = self.registration_approval_required().await?;
        let status = if approval {
            UserStatus::Pending
        } else {
            UserStatus::Active
        };

        let user_id = self.ids.new_id();
        let password_hash = self.hasher.hash(&command.password).await?;

        let new_user = NewUser {
            id: user_id.clone(),
            name: name.clone(),
            email: email.clone(),
            status,
            role_id: Some(role.id.clone()),
            phone_number: None,
            job_title: None,
            department: None,
        };

        match self.users.create(&new_user, &password_hash).await {
            Ok(()) => {}
            Err(PortError::Conflict) => return Err(DomainError::EmailInUse.into()),
            Err(error) => return Err(error.into()),
        }

        self.send_verification_email(&user_id, &email, &name).await;

        let now = self.now();

        Ok(SignUpResult {
            user: UserWithRole {
                user: crate::entity::user::User {
                    id: user_id,
                    name,
                    email,
                    email_verified: false,
                    image: None,
                    role_id: Some(role.id.clone()),
                    status,
                    phone_number: None,
                    job_title: None,
                    department: None,
                    two_factor_enabled: false,
                    created_at: now,
                    updated_at: now,
                },
                role: Some(role),
            },
        })
    }

    pub(crate) async fn send_verification_email(&self, user_id: &str, email: &str, name: &str) {
        let identifier = format!("email-verification:{user_id}");
        let expires_at = self.now() + Duration::hours(VERIFICATION_HOURS);

        let token = self.tokens.generate();

        if let Err(error) = self
            .verifications
            .replace(&identifier, &self.tokens.hash(&token), expires_at)
            .await
        {
            tracing::warn!(error = ?error, user_id, "could not store verification token");
            return;
        }

        let url = format!(
            "{}/verify-email?token={token}",
            self.config.web_url.trim_end_matches('/')
        );

        if let Err(error) = self
            .mailer
            .send_auth_email(AuthEmail::VerifyEmail, email, name, &url)
            .await
        {
            tracing::warn!(error = ?error, email, "could not send verification email");
        }
    }
}
