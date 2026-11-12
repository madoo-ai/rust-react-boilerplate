//! Session and two-factor entities.

use chrono::{DateTime, Utc};

use crate::entity::{
    authorization::Actor,
    error::DomainError,
    permission::Permission,
    role::{Role, RoleSummary},
    user::User,
};

/// A persisted session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Session {
    #[must_use]
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        self.expires_at <= now
    }

    /// Sliding renewal: at most once per day.
    #[must_use]
    pub fn needs_touch(&self, now: DateTime<Utc>) -> bool {
        self.updated_at < now - chrono::Duration::hours(24)
    }
}

/// The user and authorization context loaded with a session.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user: User,
    pub role: Option<Role>,
    pub permissions: Vec<Permission>,
}

impl AuthenticatedUser {
    #[must_use]
    pub fn actor(&self) -> Actor {
        Actor {
            id: self.user.id.clone(),
            role_name: self.role.as_ref().map(|role| role.name.clone()),
            permissions: self.permissions.clone(),
        }
    }

    #[must_use]
    pub fn role_summary(&self) -> Option<RoleSummary> {
        self.role.as_ref().map(RoleSummary::from)
    }

    #[must_use]
    pub fn permission_keys(&self) -> Vec<String> {
        self.permissions
            .iter()
            .map(|permission| permission.as_str().to_owned())
            .collect()
    }

    /// Guard an operation with a permission.
    ///
    /// # Errors
    ///
    /// Returns `Forbidden` when the permission is missing.
    pub fn require(&self, permission: Permission) -> Result<(), DomainError> {
        if self.permissions.contains(&permission) {
            Ok(())
        } else {
            Err(DomainError::Forbidden)
        }
    }
}

/// A session plus its user and permissions.
#[derive(Debug, Clone)]
pub struct AuthenticatedSession {
    pub session: Session,
    pub user: AuthenticatedUser,
}

impl AuthenticatedSession {
    #[must_use]
    pub fn actor(&self) -> Actor {
        self.user.actor()
    }
}

/// A TOTP enrollment.
#[derive(Debug, Clone)]
pub struct TwoFactor {
    pub id: String,
    pub user_id: String,
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub verified: bool,
    pub failed_verification_count: i32,
    pub locked_until: Option<DateTime<Utc>>,
}

impl TwoFactor {
    #[must_use]
    pub fn is_locked(&self, now: DateTime<Utc>) -> bool {
        self.locked_until
            .is_some_and(|locked_until| locked_until > now)
    }
}
