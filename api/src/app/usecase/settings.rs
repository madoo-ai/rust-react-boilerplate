//! Profile and account settings use case.

use std::sync::Arc;

use crate::{
    app::{
        error::AppError,
        port::{PortError, UserRepository},
        usecase::auth::AuthService,
    },
    entity::{
        error::DomainError,
        session::AuthenticatedSession,
        user::{
            UserStatus, UserUpdate, empty_to_none, normalize_email, validate_email, validate_name,
            validate_optional_fields,
        },
    },
};

/// Update the current user's profile.
#[derive(Debug, Clone)]
pub struct UpdateProfileCommand {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
}

/// Result of a profile update.
#[derive(Debug, Clone)]
pub struct UpdateProfileResult {
    pub email_changed: bool,
}

/// The settings use case.
pub struct SettingsService {
    users: Arc<dyn UserRepository>,
    auth: Arc<AuthService>,
}

impl std::fmt::Debug for SettingsService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SettingsService")
            .finish_non_exhaustive()
    }
}

impl SettingsService {
    #[must_use]
    pub fn new(users: Arc<dyn UserRepository>, auth: Arc<AuthService>) -> Self {
        Self { users, auth }
    }

    /// Update the current user's profile.
    ///
    /// # Errors
    ///
    /// Returns a validation error, `EmailInUse`, or an infrastructure error.
    pub async fn update_profile(
        &self,
        session: &AuthenticatedSession,
        command: UpdateProfileCommand,
    ) -> Result<UpdateProfileResult, AppError> {
        let name = command.name.trim().to_owned();
        let email = normalize_email(&command.email);

        validate_name(&name)?;
        validate_email(&email)?;
        validate_optional_fields(
            command.phone.as_deref(),
            command.job_title.as_deref(),
            command.department.as_deref(),
        )?;

        let email_changed = email != session.user.user.email;

        if email_changed
            && self
                .users
                .email_exists(&email, Some(&session.user.user.id))
                .await?
        {
            return Err(DomainError::EmailInUse.into());
        }

        let update = UserUpdate {
            id: session.user.user.id.clone(),
            name: name.clone(),
            email: email.clone(),
            email_verified: if email_changed {
                false
            } else {
                session.user.user.email_verified
            },
            status: session.user.user.status,
            role_id: session.user.role.as_ref().map(|role| role.id.clone()),
            phone_number: empty_to_none(command.phone.as_deref()),
            job_title: empty_to_none(command.job_title.as_deref()),
            department: empty_to_none(command.department.as_deref()),
            password_hash: None,
            revoke_sessions: false,
        };

        match self.users.update(&update).await {
            Ok(()) => {}
            Err(PortError::Conflict) => return Err(DomainError::EmailInUse.into()),
            Err(error) => return Err(error.into()),
        }

        if email_changed {
            self.auth
                .send_verification_email(&session.user.user.id, &email, &name)
                .await;
        }

        Ok(UpdateProfileResult { email_changed })
    }

    /// Delete the current account after a recent password confirmation.
    ///
    /// # Errors
    ///
    /// Returns `ConfirmPasswordRequired`, `LastSuperadmin`, or an
    /// infrastructure error.
    pub async fn delete_account(
        &self,
        session: &AuthenticatedSession,
        password_confirmed: bool,
    ) -> Result<(), AppError> {
        if !password_confirmed {
            return Err(DomainError::ConfirmPasswordRequired.into());
        }

        if session.actor().is_superadmin() && self.auth.superadmin_count().await? <= 1 {
            return Err(DomainError::LastSuperadmin.into());
        }

        self.users.delete(&session.user.user.id).await?;

        Ok(())
    }

    /// Re-read the current user after a settings write.
    ///
    /// # Errors
    ///
    /// Returns an infrastructure error.
    pub async fn current_status(
        &self,
        session: &AuthenticatedSession,
    ) -> Result<Option<UserStatus>, AppError> {
        Ok(self
            .users
            .find_by_id(&session.user.user.id)
            .await?
            .map(|user| user.user.status))
    }
}
