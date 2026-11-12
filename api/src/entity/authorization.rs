//! Authorization policies.
//!
//! Authentication answers "who is this?". This module answers "may they do
//! that?". The same rules power the API guards, the user-management use cases
//! and the sidebar.

use crate::entity::{error::DomainError, permission::Permission, role::SUPERADMIN};

/// The actor is built from the database on every request. Never trust a
/// client-supplied role or permission.
#[derive(Debug, Clone)]
pub struct Actor {
    pub id: String,
    pub role_name: Option<String>,
    pub permissions: Vec<Permission>,
}

impl Actor {
    #[must_use]
    pub fn can(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    #[must_use]
    pub fn can_any(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|permission| self.can(*permission))
    }

    #[must_use]
    pub fn is_superadmin(&self) -> bool {
        self.role_name.as_deref() == Some(SUPERADMIN)
    }

    #[must_use]
    pub fn has_role(&self, roles: &[&str]) -> bool {
        self.role_name
            .as_deref()
            .is_some_and(|name| roles.contains(&name))
    }

    /// Guard an operation with a permission.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden` when the permission is missing.
    pub fn require(&self, permission: Permission) -> Result<(), DomainError> {
        if self.can(permission) {
            Ok(())
        } else {
            Err(DomainError::Forbidden)
        }
    }
}

/// The target of a user-management action.
#[derive(Debug, Clone)]
pub struct TargetUser {
    pub id: String,
    pub role_name: Option<String>,
}

/// HR can manage regular users, but only a superadmin can touch another
/// superadmin.
#[must_use]
pub fn can_manage_user(actor: &Actor, target: &TargetUser) -> bool {
    if !actor.can(Permission::ManageUsers) {
        return false;
    }

    if target.role_name.as_deref() == Some(SUPERADMIN) && !actor.is_superadmin() {
        return false;
    }

    true
}

/// A user cannot delete themselves, and the last superadmin cannot be
/// deleted.
#[must_use]
pub fn can_delete_user(actor: &Actor, target: &TargetUser, superadmin_count: i64) -> bool {
    if !can_manage_user(actor, target) {
        return false;
    }

    if actor.id == target.id {
        return false;
    }

    if target.role_name.as_deref() == Some(SUPERADMIN) && superadmin_count <= 1 {
        return false;
    }

    true
}

/// Only actors with `manage_roles` may assign the superadmin role. This is the
/// validation-level guard that prevents privilege escalation.
#[must_use]
pub fn can_assign_role(actor: &Actor, role_name: &str) -> bool {
    if role_name == SUPERADMIN {
        return actor.can(Permission::ManageRoles);
    }

    actor.can(Permission::ManageUsers)
}

/// The superadmin role is immutable.
#[must_use]
pub fn can_manage_role(actor: &Actor, role_name: &str) -> bool {
    actor.can(Permission::ManageRoles) && role_name != SUPERADMIN
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::role::{HR, USER};

    fn actor(id: &str, role: &str, permissions: Vec<Permission>) -> Actor {
        Actor {
            id: id.to_owned(),
            role_name: Some(role.to_owned()),
            permissions,
        }
    }

    fn superadmin() -> Actor {
        actor(
            "user-superadmin",
            SUPERADMIN,
            vec![
                Permission::ViewAdmin,
                Permission::ManageUsers,
                Permission::ManageRoles,
            ],
        )
    }

    fn hr() -> Actor {
        actor(
            "user-hr",
            HR,
            vec![Permission::ViewAdmin, Permission::ManageUsers],
        )
    }

    fn regular() -> Actor {
        actor("user-regular", USER, Vec::new())
    }

    fn target(id: &str, role: &str) -> TargetUser {
        TargetUser {
            id: id.to_owned(),
            role_name: Some(role.to_owned()),
        }
    }

    #[test]
    fn checks_permissions() {
        assert!(superadmin().can(Permission::ManageRoles));
        assert!(!hr().can(Permission::ManageRoles));
        assert!(!regular().can(Permission::ViewAdmin));
    }

    #[test]
    fn checks_roles() {
        assert!(superadmin().has_role(&[SUPERADMIN]));
        assert!(hr().has_role(&[HR, SUPERADMIN]));
        assert!(!regular().is_superadmin());
    }

    #[test]
    fn hr_manages_regular_users_but_not_superadmins() {
        assert!(can_manage_user(&hr(), &target("user-target", USER)));
        assert!(!can_manage_user(
            &hr(),
            &target("user-other-superadmin", SUPERADMIN)
        ));
        assert!(can_manage_user(
            &superadmin(),
            &target("user-other-superadmin", SUPERADMIN)
        ));
    }

    #[test]
    fn prevents_privilege_escalation() {
        assert!(can_assign_role(&hr(), USER));
        assert!(!can_assign_role(&hr(), SUPERADMIN));
        assert!(can_assign_role(&superadmin(), SUPERADMIN));
    }

    #[test]
    fn prevents_self_deletion_and_deleting_the_last_superadmin() {
        let admin = superadmin();
        assert!(can_delete_user(&admin, &target("user-target", USER), 1));
        assert!(!can_delete_user(
            &admin,
            &TargetUser {
                id: admin.id.clone(),
                role_name: Some(SUPERADMIN.to_owned()),
            },
            2
        ));
        assert!(!can_delete_user(
            &admin,
            &target("user-other-superadmin", SUPERADMIN),
            1
        ));
        assert!(can_delete_user(&hr(), &target("user-target", USER), 1));
    }
}
