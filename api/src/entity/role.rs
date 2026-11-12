//! Roles and the seeded role definitions.

use crate::entity::{error::DomainError, permission::Permission};

/// System role names.
pub const SUPERADMIN: &str = "superadmin";
pub const HR: &str = "hr";
pub const USER: &str = "user";

/// A role row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub is_system: bool,
}

/// The small role shape attached to a session user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleSummary {
    pub id: String,
    pub name: String,
    pub label: String,
    pub is_system: bool,
}

impl From<&Role> for RoleSummary {
    fn from(role: &Role) -> Self {
        Self {
            id: role.id.clone(),
            name: role.name.clone(),
            label: role.label.clone(),
            is_system: role.is_system,
        }
    }
}

/// A role with its permission set and user count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleWithPermissions {
    pub role: Role,
    pub permissions: Vec<Permission>,
    pub user_count: i64,
}

/// A role plus the number of users assigned to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleWithCount {
    pub role: Role,
    pub user_count: i64,
}

/// A system role definition and its default permissions.
#[derive(Debug, Clone, Copy)]
pub struct RoleDefinition {
    pub name: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub is_system: bool,
    pub permissions: &'static [Permission],
}

/// Roles seeded on every fresh database.
pub const ROLE_DEFINITIONS: [RoleDefinition; 3] = [
    RoleDefinition {
        name: SUPERADMIN,
        label: "Super Admin",
        description: "Full access, including role management.",
        is_system: true,
        permissions: &[
            Permission::ViewAdmin,
            Permission::ManageUsers,
            Permission::ManageRoles,
        ],
    },
    RoleDefinition {
        name: HR,
        label: "HR Staff",
        description: "Can manage users but cannot manage roles.",
        is_system: false,
        permissions: &[Permission::ViewAdmin, Permission::ManageUsers],
    },
    RoleDefinition {
        name: USER,
        label: "User",
        description: "Self-service access only.",
        is_system: false,
        permissions: &[],
    },
];

/// Validate a role name: lowercase letters, digits and underscores.
///
/// # Errors
///
/// Returns a validation error keyed to the `name` field.
pub fn validate_name(value: &str) -> Result<(), DomainError> {
    let valid_length = (2..=60).contains(&value.len());
    let valid_characters = value.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    });

    if valid_length && valid_characters {
        Ok(())
    } else {
        Err(DomainError::validation("name", "validation.roleName"))
    }
}

/// Validate a role label.
///
/// # Errors
///
/// Returns a validation error keyed to the `label` field.
pub fn validate_label(value: &str) -> Result<(), DomainError> {
    if value.is_empty() || value.len() > 80 {
        Err(DomainError::validation("label", "validation.max"))
    } else {
        Ok(())
    }
}

/// Validate an optional role description.
///
/// # Errors
///
/// Returns a validation error keyed to the `description` field.
pub fn validate_description(value: Option<&str>) -> Result<(), DomainError> {
    match value {
        Some(value) if value.len() > 240 => {
            Err(DomainError::validation("description", "validation.max"))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_names_are_lower_snake_case() {
        assert!(validate_name("support_lead").is_ok());
        assert!(validate_name("Support Lead").is_err());
        assert!(validate_name("a").is_err());
    }

    #[test]
    fn role_labels_are_bounded() {
        assert!(validate_label("Support Lead").is_ok());
        assert!(validate_label("").is_err());
    }
}
