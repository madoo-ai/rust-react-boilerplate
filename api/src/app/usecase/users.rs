//! User management use case.

use std::sync::Arc;

use crate::{
    app::{
        error::AppError,
        port::{
            IdGenerator, PasswordHasher, PortError, RoleRepository, SettingsRepository,
            UserRepository,
        },
        usecase::auth::AuthService,
    },
    entity::{
        authorization::{Actor, TargetUser, can_assign_role, can_delete_user, can_manage_user},
        error::DomainError,
        permission::Permission,
        role::Role,
        user::{
            NewUser, UserFilters, UserStatus, UserUpdate, UserWithRole, empty_to_none,
            normalize_email, validate_email, validate_name, validate_optional_fields,
            validate_password,
        },
    },
};

/// Filter query for the user list.
#[derive(Debug, Clone)]
pub struct UserListQuery {
    pub actor: Actor,
    pub page: i64,
    pub search: Option<String>,
    pub role: Option<String>,
    pub status: Option<UserStatus>,
}

/// One row plus the policy flags for the current actor.
#[derive(Debug, Clone)]
pub struct UserListEntry {
    pub user: UserWithRole,
    pub can_manage: bool,
    pub can_delete: bool,
}

/// A page of policy-annotated rows.
#[derive(Debug, Clone)]
pub struct UserListResult {
    pub users: Vec<UserListEntry>,
    pub total: i64,
    pub page: i64,
    pub page_count: i64,
}

/// Roles and settings needed by the user screens.
#[derive(Debug, Clone)]
pub struct UserMeta {
    pub roles: Vec<Role>,
    pub registration_approval: bool,
    pub superadmin_count: i64,
}

/// An editable user plus the roles the actor may assign.
#[derive(Debug, Clone)]
pub struct EditableUser {
    pub user: UserWithRole,
    pub roles: Vec<Role>,
}

/// Create a user.
#[derive(Debug, Clone)]
pub struct CreateUserCommand {
    pub actor: Actor,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub role: String,
    pub status: UserStatus,
    pub password: String,
}

/// Update a user.
#[derive(Debug, Clone)]
pub struct UpdateUserCommand {
    pub actor: Actor,
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub role: String,
    pub status: UserStatus,
    pub password: Option<String>,
}

/// Result of an update, used to report whether verification mail was sent.
#[derive(Debug, Clone)]
pub struct UpdateUserResult {
    pub email_changed: bool,
}

/// The user-management use case.
pub struct UserService {
    users: Arc<dyn UserRepository>,
    roles: Arc<dyn RoleRepository>,
    settings: Arc<dyn SettingsRepository>,
    hasher: Arc<dyn PasswordHasher>,
    ids: Arc<dyn IdGenerator>,
    auth: Arc<AuthService>,
}

impl std::fmt::Debug for UserService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UserService")
            .finish_non_exhaustive()
    }
}

impl UserService {
    #[must_use]
    pub fn new(
        users: Arc<dyn UserRepository>,
        roles: Arc<dyn RoleRepository>,
        settings: Arc<dyn SettingsRepository>,
        hasher: Arc<dyn PasswordHasher>,
        ids: Arc<dyn IdGenerator>,
        auth: Arc<AuthService>,
    ) -> Self {
        Self {
            users,
            roles,
            settings,
            hasher,
            ids,
            auth,
        }
    }

    /// List users with policy flags.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden` when the actor lacks `manage_users`, or an
    /// infrastructure error.
    pub async fn list(&self, query: UserListQuery) -> Result<UserListResult, AppError> {
        query
            .actor
            .require(Permission::ManageUsers)
            .map_err(AppError::Domain)?;

        let filters = UserFilters {
            page: query.page,
            search: query.search.filter(|value| !value.trim().is_empty()),
            role: query.role.filter(|value| !value.trim().is_empty()),
            status: query.status,
        };

        let page = self.users.list(&filters).await?;
        let superadmin_count = self
            .users
            .count_by_role(crate::entity::role::SUPERADMIN)
            .await?;

        let users = page
            .users
            .into_iter()
            .map(|user| {
                let target: TargetUser = user.target();
                UserListEntry {
                    can_manage: can_manage_user(&query.actor, &target),
                    can_delete: can_delete_user(&query.actor, &target, superadmin_count),
                    user,
                }
            })
            .collect();

        Ok(UserListResult {
            users,
            total: page.total,
            page: page.page,
            page_count: page.page_count,
        })
    }

    /// Roles and settings for the users page.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden` or an infrastructure error.
    pub async fn meta(&self, actor: &Actor) -> Result<UserMeta, AppError> {
        actor.require(Permission::ManageUsers)?;

        let roles = self.roles.list().await?;
        let assignable = roles
            .into_iter()
            .filter(|role| can_assign_role(actor, &role.name))
            .collect();

        Ok(UserMeta {
            roles: assignable,
            registration_approval: self.registration_approval().await?,
            superadmin_count: self
                .users
                .count_by_role(crate::entity::role::SUPERADMIN)
                .await?,
        })
    }

    /// Load a user for the edit form.
    ///
    /// # Errors
    ///
    /// Returns `UserNotFound`, `Forbidden`, or an infrastructure error.
    pub async fn get_for_edit(&self, actor: &Actor, id: &str) -> Result<EditableUser, AppError> {
        actor.require(Permission::ManageUsers)?;

        let Some(user) = self.users.find_by_id(id).await? else {
            return Err(DomainError::UserNotFound.into());
        };

        if !can_manage_user(actor, &user.target()) {
            return Err(DomainError::Forbidden.into());
        }

        let roles = self
            .roles
            .list()
            .await?
            .into_iter()
            .filter(|role| can_assign_role(actor, &role.name))
            .collect();

        Ok(EditableUser { user, roles })
    }

    /// Create a user and credential account.
    ///
    /// # Errors
    ///
    /// Returns a validation error, `RoleNotFound`, `Forbidden`, `EmailInUse`
    /// or an infrastructure error.
    pub async fn create(&self, command: CreateUserCommand) -> Result<(), AppError> {
        command
            .actor
            .require(Permission::ManageUsers)
            .map_err(AppError::Domain)?;

        let name = command.name.trim().to_owned();
        let email = normalize_email(&command.email);

        validate_name(&name)?;
        validate_email(&email)?;
        validate_password(&command.password)?;
        validate_optional_fields(
            command.phone.as_deref(),
            command.job_title.as_deref(),
            command.department.as_deref(),
        )?;

        let role = self.assignable_role(&command.actor, &command.role).await?;

        if self.users.email_exists(&email, None).await? {
            return Err(DomainError::EmailInUse.into());
        }

        let id = self.ids.new_id();
        let password_hash = self.hasher.hash(&command.password).await?;

        let new_user = NewUser {
            id,
            name,
            email,
            status: command.status,
            role_id: Some(role.id),
            phone_number: empty_to_none(command.phone.as_deref()),
            job_title: empty_to_none(command.job_title.as_deref()),
            department: empty_to_none(command.department.as_deref()),
        };

        match self.users.create(&new_user, &password_hash).await {
            Ok(()) => Ok(()),
            Err(PortError::Conflict) => Err(DomainError::EmailInUse.into()),
            Err(error) => Err(error.into()),
        }
    }

    /// Update a user and optionally replace the password.
    ///
    /// # Errors
    ///
    /// Returns `UserNotFound`, `Forbidden`, `ProtectedUser`,
    /// `LastSuperadmin`, `EmailInUse`, a validation error, or an
    /// infrastructure error.
    pub async fn update(&self, command: UpdateUserCommand) -> Result<UpdateUserResult, AppError> {
        command
            .actor
            .require(Permission::ManageUsers)
            .map_err(AppError::Domain)?;

        let Some(target) = self.users.find_by_id(&command.id).await? else {
            return Err(DomainError::UserNotFound.into());
        };

        if !can_manage_user(&command.actor, &target.target()) {
            return Err(DomainError::Forbidden.into());
        }

        let role = self.assignable_role(&command.actor, &command.role).await?;
        let name = command.name.trim().to_owned();
        let email = normalize_email(&command.email);

        validate_name(&name)?;
        validate_email(&email)?;
        validate_optional_fields(
            command.phone.as_deref(),
            command.job_title.as_deref(),
            command.department.as_deref(),
        )?;

        let is_self = target.user.id == command.actor.id;

        if is_self
            && (role.name != target.role.as_ref().map_or("", |value| value.name.as_str())
                || command.status != target.user.status)
        {
            return Err(DomainError::ProtectedUser.into());
        }

        if target.role.as_ref().map(|value| value.name.as_str())
            == Some(crate::entity::role::SUPERADMIN)
            && role.name != crate::entity::role::SUPERADMIN
        {
            let count = self
                .users
                .count_by_role(crate::entity::role::SUPERADMIN)
                .await?;
            if count <= 1 {
                return Err(DomainError::LastSuperadmin.into());
            }
        }

        if self
            .users
            .email_exists(&email, Some(&target.user.id))
            .await?
        {
            return Err(DomainError::EmailInUse.into());
        }

        let email_changed = email != target.user.email;
        let password_hash = match command.password.filter(|value| !value.is_empty()) {
            Some(password) => {
                validate_password(&password)?;
                Some(self.hasher.hash(&password).await?)
            }
            None => None,
        };

        let update = UserUpdate {
            id: target.user.id.clone(),
            name: name.clone(),
            email: email.clone(),
            email_verified: if email_changed {
                false
            } else {
                target.user.email_verified
            },
            status: command.status,
            role_id: Some(role.id),
            phone_number: empty_to_none(command.phone.as_deref()),
            job_title: empty_to_none(command.job_title.as_deref()),
            department: empty_to_none(command.department.as_deref()),
            password_hash,
            revoke_sessions: command.status != UserStatus::Active,
        };

        match self.users.update(&update).await {
            Ok(()) => {}
            Err(PortError::Conflict) => return Err(DomainError::EmailInUse.into()),
            Err(error) => return Err(error.into()),
        }

        if email_changed {
            self.auth
                .send_verification_email(&target.user.id, &email, &name)
                .await;
        }

        Ok(UpdateUserResult { email_changed })
    }

    /// Delete a user.
    ///
    /// # Errors
    ///
    /// Returns `UserNotFound`, `CannotDeleteSelf`, `LastSuperadmin`,
    /// `Forbidden`, or an infrastructure error.
    pub async fn delete(&self, actor: &Actor, id: &str) -> Result<(), AppError> {
        actor.require(Permission::ManageUsers)?;

        let Some(target) = self.users.find_by_id(id).await? else {
            return Err(DomainError::UserNotFound.into());
        };

        let superadmin_count = self
            .users
            .count_by_role(crate::entity::role::SUPERADMIN)
            .await?;
        let target_user = target.target();

        if !can_delete_user(actor, &target_user, superadmin_count) {
            return Err(if actor.id == target.user.id {
                DomainError::CannotDeleteSelf.into()
            } else if target.role.as_ref().map(|value| value.name.as_str())
                == Some(crate::entity::role::SUPERADMIN)
            {
                DomainError::LastSuperadmin.into()
            } else {
                DomainError::Forbidden.into()
            });
        }

        self.users.delete(id).await?;
        Ok(())
    }

    /// Approve a pending user.
    ///
    /// # Errors
    ///
    /// Returns `UserNotFound`, `Forbidden`, or an infrastructure error.
    pub async fn verify(&self, actor: &Actor, id: &str) -> Result<(), AppError> {
        actor.require(Permission::ManageUsers)?;

        let Some(target) = self.users.find_by_id(id).await? else {
            return Err(DomainError::UserNotFound.into());
        };

        if !can_manage_user(actor, &target.target()) {
            return Err(DomainError::Forbidden.into());
        }

        if target.user.status == UserStatus::Pending {
            self.users
                .set_status(&target.user.id, UserStatus::Active)
                .await?;
        }

        Ok(())
    }

    /// Toggle the registration approval setting.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden` or an infrastructure error.
    pub async fn set_registration_approval(
        &self,
        actor: &Actor,
        enabled: bool,
    ) -> Result<(), AppError> {
        actor.require(Permission::ManageUsers)?;

        self.settings
            .set(
                &self.ids.new_id(),
                "registration_approval",
                if enabled { "true" } else { "false" },
            )
            .await?;

        Ok(())
    }

    async fn assignable_role(&self, actor: &Actor, role_name: &str) -> Result<Role, AppError> {
        let Some(role) = self.roles.find_by_name(role_name).await? else {
            return Err(DomainError::RoleNotFound.into());
        };

        if !can_assign_role(actor, &role.name) {
            return Err(DomainError::Forbidden.into());
        }

        Ok(role)
    }

    async fn registration_approval(&self) -> Result<bool, AppError> {
        Ok(self.settings.get("registration_approval").await?.as_deref() == Some("true"))
    }
}
