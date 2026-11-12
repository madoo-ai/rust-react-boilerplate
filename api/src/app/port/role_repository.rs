//! Role and permission persistence.

use async_trait::async_trait;

use crate::{
    app::port::error::PortError,
    entity::{
        permission::{Permission, PermissionRecord},
        role::{Role, RoleWithCount, RoleWithPermissions},
    },
};

/// Storage for roles, permissions and their join rows.
#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// List roles ordered by name.
    async fn list(&self) -> Result<Vec<Role>, PortError>;

    /// List roles with their permissions and user counts.
    async fn list_with_permissions(&self) -> Result<Vec<RoleWithPermissions>, PortError>;

    /// List roles with user counts.
    async fn list_with_counts(&self) -> Result<Vec<RoleWithCount>, PortError>;

    /// Load a role by id.
    async fn find_by_id(&self, id: &str) -> Result<Option<Role>, PortError>;

    /// Load a role by name.
    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, PortError>;

    /// Resolve permission keys to database ids, ignoring unknown keys.
    async fn permission_ids(&self, permissions: &[Permission]) -> Result<Vec<String>, PortError>;

    /// Load the permission set for a role.
    async fn permissions_for_role(&self, role_id: &str) -> Result<Vec<Permission>, PortError>;

    /// Count users assigned to a role.
    async fn user_count(&self, role_id: &str) -> Result<i64, PortError>;

    /// Insert a role and its permission rows in one transaction.
    async fn create(&self, role: &Role, permissions: &[Permission]) -> Result<(), PortError>;

    /// Update a role and replace its permission rows in one transaction.
    async fn update(&self, role: &Role, permissions: &[Permission]) -> Result<(), PortError>;

    /// Delete a role. Permission rows cascade.
    async fn delete(&self, id: &str) -> Result<(), PortError>;

    /// List all permission records ordered by key.
    async fn list_permissions(&self) -> Result<Vec<PermissionRecord>, PortError>;
}
