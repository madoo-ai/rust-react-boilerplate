//! Private `PostgreSQL` row types and entity conversions.

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::entity::{
    permission::PermissionRecord,
    role::Role,
    session::{Session, TwoFactor},
    user::{User, UserStatus, UserWithRole},
};

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum StatusRow {
    Pending,
    Active,
    Suspended,
}

impl From<StatusRow> for UserStatus {
    fn from(status: StatusRow) -> Self {
        match status {
            StatusRow::Pending => Self::Pending,
            StatusRow::Active => Self::Active,
            StatusRow::Suspended => Self::Suspended,
        }
    }
}

impl From<UserStatus> for StatusRow {
    fn from(status: UserStatus) -> Self {
        match status {
            UserStatus::Pending => Self::Pending,
            UserStatus::Active => Self::Active,
            UserStatus::Suspended => Self::Suspended,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct RoleRow {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub is_system: bool,
}

impl From<RoleRow> for Role {
    fn from(row: RoleRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            label: row.label,
            description: row.description,
            is_system: row.is_system,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct UserWithRoleRow {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    pub role_id: Option<String>,
    pub status: StatusRow,
    pub phone_number: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub two_factor_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub joined_role_id: Option<String>,
    pub role_name: Option<String>,
    pub role_label: Option<String>,
    pub role_description: Option<String>,
    pub role_is_system: Option<bool>,
}

impl From<UserWithRoleRow> for UserWithRole {
    fn from(row: UserWithRoleRow) -> Self {
        let role = match (row.joined_role_id, row.role_name, row.role_label) {
            (Some(id), Some(name), Some(label)) => Some(Role {
                id,
                name,
                label,
                description: row.role_description,
                is_system: row.role_is_system.unwrap_or(false),
            }),
            _ => None,
        };

        Self {
            user: User {
                id: row.id,
                name: row.name,
                email: row.email,
                email_verified: row.email_verified,
                image: row.image,
                role_id: row.role_id,
                status: row.status.into(),
                phone_number: row.phone_number,
                job_title: row.job_title,
                department: row.department,
                two_factor_enabled: row.two_factor_enabled,
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            role,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct RoleWithCountRow {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub user_count: i64,
}

impl From<RoleWithCountRow> for crate::entity::role::RoleWithCount {
    fn from(row: RoleWithCountRow) -> Self {
        Self {
            role: Role {
                id: row.id,
                name: row.name,
                label: row.label,
                description: row.description,
                is_system: row.is_system,
            },
            user_count: row.user_count,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PermissionRow {
    pub id: String,
    pub key: String,
    pub label: String,
    pub description: Option<String>,
}

impl From<PermissionRow> for PermissionRecord {
    fn from(row: PermissionRow) -> Self {
        Self {
            id: row.id,
            key: row.key,
            label: row.label,
            description: row.description,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: String,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl From<SessionRow> for Session {
    fn from(row: SessionRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            expires_at: row.expires_at,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct TwoFactorRow {
    pub id: String,
    pub user_id: String,
    pub secret: String,
    pub backup_codes: sqlx::types::Json<Vec<String>>,
    pub verified: bool,
    pub failed_verification_count: i32,
    pub locked_until: Option<DateTime<Utc>>,
}

impl From<TwoFactorRow> for TwoFactor {
    fn from(row: TwoFactorRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            secret: row.secret,
            backup_codes: row.backup_codes.0,
            verified: row.verified,
            failed_verification_count: row.failed_verification_count,
            locked_until: row.locked_until,
        }
    }
}
