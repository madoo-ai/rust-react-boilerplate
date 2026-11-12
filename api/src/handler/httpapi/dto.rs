//! JSON shapes for the React client.
//!
//! DTOs are deliberately separate from entities so the API can evolve without
//! changing the domain.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::entity::{
    dashboard::DashboardData,
    role::{Role, RoleSummary, RoleWithPermissions},
    session::{AuthenticatedSession, Session},
    user::{UserStatus, UserWithRole},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDto {
    pub id: String,
    pub name: String,
    pub label: String,
    pub is_system: bool,
}

impl From<&Role> for RoleDto {
    fn from(role: &Role) -> Self {
        Self {
            id: role.id.clone(),
            name: role.name.clone(),
            label: role.label.clone(),
            is_system: role.is_system,
        }
    }
}

impl From<&RoleSummary> for RoleDto {
    fn from(role: &RoleSummary) -> Self {
        Self {
            id: role.id.clone(),
            name: role.name.clone(),
            label: role.label.clone(),
            is_system: role.is_system,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    pub status: UserStatus,
    pub phone_number: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub two_factor_enabled: bool,
    pub role: Option<RoleDto>,
    pub permissions: Vec<String>,
}

impl From<&AuthenticatedSession> for UserDto {
    fn from(session: &AuthenticatedSession) -> Self {
        Self {
            id: session.user.user.id.clone(),
            name: session.user.user.name.clone(),
            email: session.user.user.email.clone(),
            email_verified: session.user.user.email_verified,
            image: session.user.user.image.clone(),
            status: session.user.user.status,
            phone_number: session.user.user.phone_number.clone(),
            job_title: session.user.user.job_title.clone(),
            department: session.user.user.department.clone(),
            two_factor_enabled: session.user.user.two_factor_enabled,
            role: session.user.role_summary().as_ref().map(RoleDto::from),
            permissions: session.user.permission_keys(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl From<&Session> for SessionDto {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            expires_at: session.expires_at,
            ip_address: session.ip_address.clone(),
            user_agent: session.user_agent.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponseDto {
    pub session: SessionDto,
    pub user: UserDto,
    pub password_confirmed: bool,
    pub can_delete_account: bool,
    pub superadmin_count: i64,
}

#[derive(Debug, Serialize)]
pub struct SessionEnvelope {
    pub session: Option<SessionResponseDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignUpUserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub status: UserStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignUpResponse {
    pub user: SignUpUserDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInResponse {
    pub user: UserDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorRequiredResponse {
    pub two_factor_redirect: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRowDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub status: UserStatus,
    pub role: Option<RoleDto>,
    pub created_at: DateTime<Utc>,
    pub can_manage: bool,
    pub can_delete: bool,
}

impl From<&crate::app::usecase::users::UserListEntry> for UserRowDto {
    fn from(entry: &crate::app::usecase::users::UserListEntry) -> Self {
        Self {
            id: entry.user.user.id.clone(),
            name: entry.user.user.name.clone(),
            email: entry.user.user.email.clone(),
            status: entry.user.user.status,
            role: entry.user.role_summary().as_ref().map(RoleDto::from),
            created_at: entry.user.user.created_at,
            can_manage: entry.can_manage,
            can_delete: entry.can_delete,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDetailDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub role: Option<String>,
    pub status: UserStatus,
}

impl From<&UserWithRole> for UserDetailDto {
    fn from(user: &UserWithRole) -> Self {
        Self {
            id: user.user.id.clone(),
            name: user.user.name.clone(),
            email: user.user.email.clone(),
            phone_number: user.user.phone_number.clone(),
            job_title: user.user.job_title.clone(),
            department: user.user.department.clone(),
            role: user.role.as_ref().map(|role| role.name.clone()),
            status: user.user.status,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditableUserDto {
    pub user: UserDetailDto,
    pub roles: Vec<RoleDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMetaDto {
    pub roles: Vec<RoleDto>,
    pub registration_approval: bool,
    pub superadmin_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleWithPermissionsDto {
    pub id: String,
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub permissions: Vec<String>,
    pub user_count: i64,
}

impl From<&RoleWithPermissions> for RoleWithPermissionsDto {
    fn from(role: &RoleWithPermissions) -> Self {
        Self {
            id: role.role.id.clone(),
            name: role.role.name.clone(),
            label: role.role.label.clone(),
            description: role.role.description.clone(),
            is_system: role.role.is_system,
            permissions: role
                .permissions
                .iter()
                .map(|permission| permission.as_str().to_owned())
                .collect(),
            user_count: role.user_count,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionDto {
    pub id: String,
    pub key: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RolesViewDto {
    pub roles: Vec<RoleWithPermissionsDto>,
    pub permissions: Vec<PermissionDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupPointDto {
    pub date: String,
    pub users: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDatumDto {
    pub name: String,
    pub label: String,
    pub users: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentUserDto {
    pub id: String,
    pub name: String,
    pub email: String,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub role: Option<RoleDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardDto {
    pub total_users: i64,
    pub active_users: i64,
    pub pending_users: i64,
    pub roles_count: i64,
    pub role_data: Vec<RoleDatumDto>,
    pub signup_data: Vec<SignupPointDto>,
    pub recent_users: Vec<RecentUserDto>,
}

impl From<&DashboardData> for DashboardDto {
    fn from(data: &DashboardData) -> Self {
        Self {
            total_users: data.total_users,
            active_users: data.active_users,
            pending_users: data.pending_users,
            roles_count: data.roles_count,
            role_data: data
                .role_data
                .iter()
                .map(|datum| RoleDatumDto {
                    name: datum.name.clone(),
                    label: datum.label.clone(),
                    users: datum.users,
                })
                .collect(),
            signup_data: data
                .signup_data
                .iter()
                .map(|point| SignupPointDto {
                    date: point.date.format("%Y-%m-%d").to_string(),
                    users: point.users,
                })
                .collect(),
            recent_users: data
                .recent_users
                .iter()
                .map(|user| RecentUserDto {
                    id: user.user.id.clone(),
                    name: user.user.name.clone(),
                    email: user.user.email.clone(),
                    status: user.user.status,
                    created_at: user.user.created_at,
                    role: user.role_summary().as_ref().map(RoleDto::from),
                })
                .collect(),
        }
    }
}
