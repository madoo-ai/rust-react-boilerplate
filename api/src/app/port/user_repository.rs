//! User persistence.

use async_trait::async_trait;

use crate::{
    app::port::error::PortError,
    entity::user::{CredentialUser, NewUser, UserFilters, UserPage, UserUpdate, UserWithRole},
};

/// Storage for users and their credential account.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Load a user and their role by id.
    async fn find_by_id(&self, id: &str) -> Result<Option<UserWithRole>, PortError>;

    /// Load a user and their role by email.
    async fn find_by_email(&self, email: &str) -> Result<Option<UserWithRole>, PortError>;

    /// Load a user, their role and the credential password hash.
    async fn find_credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<CredentialUser>, PortError>;

    /// Load a user, their role and the credential password hash by id.
    async fn find_credentials_by_id(&self, id: &str) -> Result<Option<CredentialUser>, PortError>;

    /// True when the email belongs to a different user.
    async fn email_exists(&self, email: &str, exclude_id: Option<&str>) -> Result<bool, PortError>;

    /// List users for the admin table.
    async fn list(&self, filters: &UserFilters) -> Result<UserPage, PortError>;

    /// Insert a user and their credential account in one transaction.
    async fn create(&self, user: &NewUser, password_hash: &str) -> Result<(), PortError>;

    /// Update a user, optionally replace the password and revoke sessions.
    async fn update(&self, update: &UserUpdate) -> Result<(), PortError>;

    /// Delete a user. Relations cascade.
    async fn delete(&self, id: &str) -> Result<(), PortError>;

    /// Mark the user's current email address as verified.
    async fn mark_email_verified(&self, id: &str) -> Result<(), PortError>;

    /// Change only the account status.
    async fn set_status(
        &self,
        id: &str,
        status: crate::entity::user::UserStatus,
    ) -> Result<(), PortError>;

    /// Replace the credential password.
    async fn update_password(&self, id: &str, password_hash: &str) -> Result<(), PortError>;

    /// Set the denormalized two-factor flag on the user.
    async fn set_two_factor_enabled(&self, id: &str, enabled: bool) -> Result<(), PortError>;

    /// Count users assigned to a role.
    async fn count_by_role(&self, role_name: &str) -> Result<i64, PortError>;

    /// Load all users with their roles. Used by the dashboard.
    async fn list_all_with_roles(&self) -> Result<Vec<UserWithRole>, PortError>;
}
