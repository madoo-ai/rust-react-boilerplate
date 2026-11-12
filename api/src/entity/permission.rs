//! Permission registry.

use serde::{Deserialize, Serialize};

/// Permission keys stored in the `permissions` table.
pub const VIEW_ADMIN: &str = "view_admin";
pub const MANAGE_USERS: &str = "manage_users";
pub const MANAGE_ROLES: &str = "manage_roles";

/// A typed permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    ViewAdmin,
    ManageUsers,
    ManageRoles,
}

impl Permission {
    /// All known permissions, in registry order.
    pub const ALL: [Self; 3] = [Self::ViewAdmin, Self::ManageUsers, Self::ManageRoles];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ViewAdmin => VIEW_ADMIN,
            Self::ManageUsers => MANAGE_USERS,
            Self::ManageRoles => MANAGE_ROLES,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ViewAdmin => "Access the admin area",
            Self::ManageUsers => "Create, edit, verify and delete users",
            Self::ManageRoles => "Create, edit and delete roles",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            VIEW_ADMIN => Some(Self::ViewAdmin),
            MANAGE_USERS => Some(Self::ManageUsers),
            MANAGE_ROLES => Some(Self::ManageRoles),
            _ => None,
        }
    }
}

/// A permission row, including its display label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRecord {
    pub id: String,
    pub key: String,
    pub label: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_keys_only() {
        assert_eq!(Permission::parse("view_admin"), Some(Permission::ViewAdmin));
        assert_eq!(
            Permission::parse("manage_users"),
            Some(Permission::ManageUsers)
        );
        assert_eq!(
            Permission::parse("manage_roles"),
            Some(Permission::ManageRoles)
        );
        assert_eq!(Permission::parse("root"), None);
    }
}
