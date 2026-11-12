//! User entity, filters and field validation.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::ValidateEmail;

use crate::entity::{authorization::TargetUser, error::DomainError, role::RoleSummary};

/// Rows per page in the user list.
pub const PAGE_SIZE: i64 = 10;

/// Account status. Mirrors the `PostgreSQL` `user_status` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Pending,
    Active,
    Suspended,
}

impl UserStatus {
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "active" => Some(Self::Active),
            "suspended" => Some(Self::Suspended),
            _ => None,
        }
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A user row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    pub role_id: Option<String>,
    pub status: UserStatus,
    pub phone_number: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub two_factor_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A user and their role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserWithRole {
    pub user: User,
    pub role: Option<crate::entity::role::Role>,
}

impl UserWithRole {
    #[must_use]
    pub fn target(&self) -> TargetUser {
        TargetUser {
            id: self.user.id.clone(),
            role_name: self.role.as_ref().map(|role| role.name.clone()),
        }
    }

    #[must_use]
    pub fn role_summary(&self) -> Option<RoleSummary> {
        self.role.as_ref().map(RoleSummary::from)
    }
}

/// A credential row used by sign-in.
#[derive(Debug, Clone)]
pub struct CredentialUser {
    pub user: UserWithRole,
    pub password_hash: Option<String>,
}

/// Fields required to create a user.
#[derive(Debug, Clone)]
pub struct NewUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub status: UserStatus,
    pub role_id: Option<String>,
    pub phone_number: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
}

/// Fields an authorized actor may change.
#[derive(Debug, Clone)]
pub struct UserUpdate {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub status: UserStatus,
    pub role_id: Option<String>,
    pub phone_number: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub password_hash: Option<String>,
    pub revoke_sessions: bool,
}

/// User list filters.
#[derive(Debug, Clone, Default)]
pub struct UserFilters {
    pub page: i64,
    pub search: Option<String>,
    pub role: Option<String>,
    pub status: Option<UserStatus>,
}

impl UserFilters {
    /// Clamp the page to the available range.
    #[must_use]
    pub fn normalized_page(&self, total: i64) -> i64 {
        let page_count = page_count(total);
        self.page.clamp(1, page_count)
    }
}

/// A page of users.
#[derive(Debug, Clone)]
pub struct UserPage {
    pub users: Vec<UserWithRole>,
    pub total: i64,
    pub page: i64,
    pub page_count: i64,
}

/// Number of pages for a total.
#[must_use]
pub fn page_count(total: i64) -> i64 {
    if total <= 0 {
        1
    } else {
        (total + PAGE_SIZE - 1) / PAGE_SIZE
    }
}

/// Normalize an email for storage and lookup.
#[must_use]
pub fn normalize_email(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Convert an empty string to `None`.
#[must_use]
pub fn empty_to_none(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// Validate a profile name.
///
/// # Errors
///
/// Returns a validation error keyed to `name`.
pub fn validate_name(value: &str) -> Result<(), DomainError> {
    let length = value.trim().chars().count();

    if (2..=120).contains(&length) {
        Ok(())
    } else {
        Err(DomainError::validation("name", "validation.nameLength"))
    }
}

/// Validate an email address.
///
/// # Errors
///
/// Returns a validation error keyed to `email`.
pub fn validate_email(value: &str) -> Result<(), DomainError> {
    if value.trim().validate_email() {
        Ok(())
    } else {
        Err(DomainError::validation("email", "validation.email"))
    }
}

/// Validate a password.
///
/// # Errors
///
/// Returns a validation error keyed to `password`.
pub fn validate_password(value: &str) -> Result<(), DomainError> {
    if (8..=128).contains(&value.chars().count()) {
        Ok(())
    } else {
        Err(DomainError::validation(
            "password",
            "validation.passwordLength",
        ))
    }
}

/// Validate the optional profile fields and return the combined error map.
///
/// # Errors
///
/// Returns a validation map when any field is too long.
pub fn validate_optional_fields(
    phone: Option<&str>,
    job_title: Option<&str>,
    department: Option<&str>,
) -> Result<(), DomainError> {
    let mut fields = BTreeMap::new();

    if phone.is_some_and(|value| value.trim().len() > 40) {
        fields.insert("phone".to_owned(), "validation.max".to_owned());
    }
    if job_title.is_some_and(|value| value.trim().len() > 120) {
        fields.insert("jobTitle".to_owned(), "validation.max".to_owned());
    }
    if department.is_some_and(|value| value.trim().len() > 120) {
        fields.insert("department".to_owned(), "validation.max".to_owned());
    }

    if fields.is_empty() {
        Ok(())
    } else {
        Err(DomainError::Validation(fields))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_email() {
        assert_eq!(normalize_email(" Ada@Example.COM "), "ada@example.com");
    }

    #[test]
    fn validates_profile_fields() {
        assert!(validate_name("Ada Lovelace").is_ok());
        assert!(validate_name("A").is_err());
        assert!(validate_email("ada@example.com").is_ok());
        assert!(validate_email("not-an-email").is_err());
        assert!(validate_password("password123").is_ok());
        assert!(validate_password("short").is_err());
    }

    #[test]
    fn page_count_is_at_least_one() {
        assert_eq!(page_count(0), 1);
        assert_eq!(page_count(10), 1);
        assert_eq!(page_count(11), 2);
    }

    #[test]
    fn filters_clamp_page() {
        let filters = UserFilters {
            page: 99,
            ..UserFilters::default()
        };
        assert_eq!(filters.normalized_page(21), 3);
    }
}
