//! HTTP error mapping for the driving adapter.

use std::collections::BTreeMap;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::{app::error::AppError, entity::error::DomainError};

/// JSON error envelope.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

/// Machine-readable error detail.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<BTreeMap<String, String>>,
}

/// A mapped API error.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    fields: Option<BTreeMap<String, String>>,
}

impl ApiError {
    #[must_use]
    pub const fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self {
            status,
            code,
            message,
            fields: None,
        }
    }

    #[must_use]
    pub fn too_many_requests() -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "too_many_requests",
            "errors.tooManyRequests",
        )
    }

    #[must_use]
    pub fn bad_request(message: &'static str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }
}

impl From<AppError> for ApiError {
    fn from(error: AppError) -> Self {
        match error {
            AppError::Unexpected(error) => {
                tracing::error!(error = ?error, "unexpected error");
                Self::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "errors.unexpected",
                )
            }
            AppError::Domain(domain) => domain.into(),
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::Unauthorized => Self::new(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "errors.unauthorized",
            ),
            DomainError::Forbidden => {
                Self::new(StatusCode::FORBIDDEN, "forbidden", "errors.forbidden")
            }
            DomainError::AccountInactive => Self::new(
                StatusCode::FORBIDDEN,
                "account_inactive",
                "errors.accountInactive",
            ),
            DomainError::AccountPending => Self::new(
                StatusCode::FORBIDDEN,
                "ACCOUNT_PENDING",
                "auth.messages.pendingApproval",
            ),
            DomainError::AccountSuspended => Self::new(
                StatusCode::FORBIDDEN,
                "ACCOUNT_SUSPENDED",
                "auth.messages.suspended",
            ),
            DomainError::InvalidCredentials => Self::new(
                StatusCode::UNAUTHORIZED,
                "INVALID_EMAIL_OR_PASSWORD",
                "errors.invalidCredentials",
            ),
            DomainError::EmailInUse => {
                Self::new(StatusCode::CONFLICT, "email_in_use", "errors.emailInUse")
            }
            DomainError::UserNotFound => Self::new(
                StatusCode::NOT_FOUND,
                "user_not_found",
                "errors.userNotFound",
            ),
            DomainError::RoleNotFound => Self::new(
                StatusCode::NOT_FOUND,
                "role_not_found",
                "errors.roleNotFound",
            ),
            DomainError::RoleInUse => {
                Self::new(StatusCode::CONFLICT, "role_in_use", "errors.roleInUse")
            }
            DomainError::ImmutableRole => Self::new(
                StatusCode::CONFLICT,
                "immutable_role",
                "errors.immutableRole",
            ),
            DomainError::DefaultRole => {
                Self::new(StatusCode::CONFLICT, "default_role", "errors.defaultRole")
            }
            DomainError::LastSuperadmin => Self::new(
                StatusCode::CONFLICT,
                "last_superadmin",
                "errors.lastSuperadmin",
            ),
            DomainError::ProtectedUser => Self::new(
                StatusCode::FORBIDDEN,
                "protected_user",
                "errors.protectedUser",
            ),
            DomainError::CannotDeleteSelf => Self::new(
                StatusCode::CONFLICT,
                "cannot_delete_self",
                "errors.cannotDeleteSelf",
            ),
            DomainError::InvalidToken => Self::new(
                StatusCode::BAD_REQUEST,
                "INVALID_TOKEN",
                "errors.invalidToken",
            ),
            DomainError::InvalidCode => Self::new(
                StatusCode::BAD_REQUEST,
                "INVALID_CODE",
                "errors.invalidCode",
            ),
            DomainError::TwoFactorNotEnabled => Self::new(
                StatusCode::BAD_REQUEST,
                "two_factor_not_enabled",
                "errors.twoFactorNotEnabled",
            ),
            DomainError::TwoFactorAlreadyEnabled => Self::new(
                StatusCode::CONFLICT,
                "two_factor_already_enabled",
                "errors.twoFactorAlreadyEnabled",
            ),
            DomainError::IncorrectPassword => Self::new(
                StatusCode::BAD_REQUEST,
                "incorrect_password",
                "errors.incorrectPassword",
            ),
            DomainError::ConfirmPasswordRequired => Self::new(
                StatusCode::FORBIDDEN,
                "confirm_password_required",
                "errors.confirmPasswordRequired",
            ),
            DomainError::NotFound => {
                Self::new(StatusCode::NOT_FOUND, "not_found", "errors.notFound")
            }
            DomainError::Conflict => {
                Self::new(StatusCode::CONFLICT, "conflict", "errors.unexpected")
            }
            DomainError::Validation(fields) => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: "validation_failed",
                message: "errors.invalidRequest",
                fields: Some(fields),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: ErrorDetail {
                    code: self.code.to_owned(),
                    message: self.message.to_owned(),
                    fields: self.fields,
                },
            }),
        )
            .into_response()
    }
}
