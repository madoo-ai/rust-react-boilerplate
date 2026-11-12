//! Domain errors.
//!
//! Entities and use cases return these variants. They carry meaning, not HTTP
//! status codes or i18n text; the driving adapter maps them at the boundary.

use std::collections::BTreeMap;

/// A business-rule failure.
#[derive(Debug, Clone, thiserror::Error)]
pub enum DomainError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("account inactive")]
    AccountInactive,
    #[error("account pending approval")]
    AccountPending,
    #[error("account suspended")]
    AccountSuspended,
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("email already in use")]
    EmailInUse,
    #[error("user not found")]
    UserNotFound,
    #[error("role not found")]
    RoleNotFound,
    #[error("role is assigned to users")]
    RoleInUse,
    #[error("role is immutable")]
    ImmutableRole,
    #[error("the default role cannot be renamed or removed")]
    DefaultRole,
    #[error("the last superadmin cannot be removed")]
    LastSuperadmin,
    #[error("this account is protected")]
    ProtectedUser,
    #[error("you cannot delete your own account here")]
    CannotDeleteSelf,
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("invalid two-factor code")]
    InvalidCode,
    #[error("two-factor authentication is not enabled")]
    TwoFactorNotEnabled,
    #[error("two-factor authentication is already enabled")]
    TwoFactorAlreadyEnabled,
    #[error("the password is incorrect")]
    IncorrectPassword,
    #[error("a recent password confirmation is required")]
    ConfirmPasswordRequired,
    #[error("the requested record was not found")]
    NotFound,
    #[error("the request conflicts with the current state")]
    Conflict,
    #[error("validation failed")]
    Validation(BTreeMap<String, String>),
}

impl DomainError {
    #[must_use]
    pub fn validation(field: &str, message: &str) -> Self {
        let mut fields = BTreeMap::new();
        fields.insert(field.to_owned(), message.to_owned());
        Self::Validation(fields)
    }
}
