//! Role management use case.

use std::sync::Arc;

use crate::{
    app::{
        error::AppError,
        port::{IdGenerator, RoleRepository},
    },
    entity::{
        authorization::{Actor, can_manage_role},
        error::DomainError,
        permission::{Permission, PermissionRecord},
        role::{
            Role, RoleWithPermissions, USER, validate_description, validate_label, validate_name,
        },
    },
};

/// The role page read model.
#[derive(Debug, Clone)]
pub struct RolesView {
    pub roles: Vec<RoleWithPermissions>,
    pub permissions: Vec<PermissionRecord>,
}

/// A role form payload.
#[derive(Debug, Clone)]
pub struct RoleInput {
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
}

/// The role-management use case.
pub struct RoleService {
    roles: Arc<dyn RoleRepository>,
    ids: Arc<dyn IdGenerator>,
}

impl std::fmt::Debug for RoleService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RoleService")
            .finish_non_exhaustive()
    }
}

impl RoleService {
    #[must_use]
    pub fn new(roles: Arc<dyn RoleRepository>, ids: Arc<dyn IdGenerator>) -> Self {
        Self { roles, ids }
    }

    /// Load roles and the permission registry.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden` or an infrastructure error.
    pub async fn list(&self, actor: &Actor) -> Result<RolesView, AppError> {
        actor.require(Permission::ManageRoles)?;

        Ok(RolesView {
            roles: self.roles.list_with_permissions().await?,
            permissions: self.roles.list_permissions().await?,
        })
    }

    /// Create a role.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden`, a validation error, `RoleInUse`, or an
    /// infrastructure error.
    pub async fn create(&self, actor: &Actor, input: RoleInput) -> Result<(), AppError> {
        actor.require(Permission::ManageRoles)?;

        validate_input(&input)?;

        if self.roles.find_by_name(&input.name).await?.is_some() {
            return Err(DomainError::RoleInUse.into());
        }

        let role = Role {
            id: self.ids.new_id(),
            name: input.name,
            label: input.label,
            description: input.description.filter(|value| !value.is_empty()),
            is_system: false,
        };

        match self.roles.create(&role, &input.permissions).await {
            Ok(()) => Ok(()),
            Err(crate::app::port::PortError::Conflict) => Err(DomainError::RoleInUse.into()),
            Err(error) => Err(error.into()),
        }
    }

    /// Update a role and replace its permissions.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden`, `RoleNotFound`, `ImmutableRole`, `DefaultRole`,
    /// `RoleInUse`, a validation error, or an infrastructure error.
    pub async fn update(&self, actor: &Actor, id: &str, input: RoleInput) -> Result<(), AppError> {
        actor.require(Permission::ManageRoles)?;

        let Some(role) = self.roles.find_by_id(id).await? else {
            return Err(DomainError::RoleNotFound.into());
        };

        if !can_manage_role(actor, &role.name) {
            return Err(DomainError::ImmutableRole.into());
        }

        if role.name == USER && input.name != USER {
            return Err(DomainError::DefaultRole.into());
        }

        validate_input(&input)?;

        if let Some(duplicate) = self.roles.find_by_name(&input.name).await?
            && duplicate.id != role.id
        {
            return Err(DomainError::RoleInUse.into());
        }

        let updated = Role {
            id: role.id,
            name: input.name,
            label: input.label,
            description: input.description.filter(|value| !value.is_empty()),
            is_system: role.is_system,
        };

        match self.roles.update(&updated, &input.permissions).await {
            Ok(()) => Ok(()),
            Err(crate::app::port::PortError::Conflict) => Err(DomainError::RoleInUse.into()),
            Err(error) => Err(error.into()),
        }
    }

    /// Delete a role that is not assigned to users.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden`, `RoleNotFound`, `ImmutableRole`, `DefaultRole`,
    /// `RoleInUse`, or an infrastructure error.
    pub async fn delete(&self, actor: &Actor, id: &str) -> Result<(), AppError> {
        actor.require(Permission::ManageRoles)?;

        let Some(role) = self.roles.find_by_id(id).await? else {
            return Err(DomainError::RoleNotFound.into());
        };

        if role.name == crate::entity::role::SUPERADMIN {
            return Err(DomainError::ImmutableRole.into());
        }

        if role.name == USER {
            return Err(DomainError::DefaultRole.into());
        }

        if self.roles.user_count(&role.id).await? > 0 {
            return Err(DomainError::RoleInUse.into());
        }

        self.roles.delete(&role.id).await?;
        Ok(())
    }
}

fn validate_input(input: &RoleInput) -> Result<(), AppError> {
    validate_name(&input.name)?;
    validate_label(&input.label)?;
    validate_description(input.description.as_deref())?;

    // Deduplicate while preserving registry order.
    let unique = input
        .permissions
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();

    if unique.len() != input.permissions.len() {
        return Err(DomainError::validation("permissions", "validation.invalidPermission").into());
    }

    Ok(())
}
