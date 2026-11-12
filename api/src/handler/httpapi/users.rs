//! User management HTTP handlers.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::{
    app::usecase::users::{CreateUserCommand, UpdateUserCommand, UserListQuery},
    entity::{error::DomainError, user::UserStatus},
    handler::httpapi::{
        dto::{EditableUserDto, RoleDto, UserDetailDto, UserMetaDto, UserRowDto},
        error::ApiError,
        extractors::{CurrentSession, JsonBody, OkResponse},
        state::AppState,
    },
};

#[derive(Debug, Deserialize)]
pub struct UserFilterQuery {
    page: Option<i64>,
    search: Option<String>,
    role: Option<String>,
    status: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListResponse {
    users: Vec<UserRowDto>,
    total: i64,
    page: i64,
    page_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    name: String,
    email: String,
    phone: Option<String>,
    job_title: Option<String>,
    department: Option<String>,
    role: String,
    status: String,
    password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    name: String,
    email: String,
    phone: Option<String>,
    job_title: Option<String>,
    department: Option<String>,
    role: String,
    status: String,
    password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationApprovalRequest {
    enabled: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateResponse {
    email_changed: bool,
}

/// `GET /api/users`
pub async fn list(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    Query(query): Query<UserFilterQuery>,
) -> Result<Json<UserListResponse>, ApiError> {
    let status = match query.status.as_deref() {
        None | Some("" | "all") => None,
        Some(value) => Some(UserStatus::parse(value).ok_or_else(|| {
            ApiError::from(DomainError::validation(
                "status",
                "validation.invalidStatus",
            ))
        })?),
    };

    let result = state
        .users
        .list(UserListQuery {
            actor: session.user.actor(),
            page: query.page.unwrap_or(1),
            search: query.search,
            role: query.role,
            status,
        })
        .await?;

    Ok(Json(UserListResponse {
        users: result.users.iter().map(UserRowDto::from).collect(),
        total: result.total,
        page: result.page,
        page_count: result.page_count,
    }))
}

/// `GET /api/users/meta`
pub async fn meta(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
) -> Result<Json<UserMetaDto>, ApiError> {
    let meta = state.users.meta(&session.user.actor()).await?;

    Ok(Json(UserMetaDto {
        roles: meta.roles.iter().map(RoleDto::from).collect(),
        registration_approval: meta.registration_approval,
        superadmin_count: meta.superadmin_count,
    }))
}

/// `GET /api/users/{id}`
pub async fn get(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    Path(id): Path<String>,
) -> Result<Json<EditableUserDto>, ApiError> {
    let editable = state.users.get_for_edit(&session.user.actor(), &id).await?;

    Ok(Json(EditableUserDto {
        user: UserDetailDto::from(&editable.user),
        roles: editable.roles.iter().map(RoleDto::from).collect(),
    }))
}

/// `POST /api/users`
pub async fn create(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<CreateUserRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state
        .users
        .create(CreateUserCommand {
            actor: session.user.actor(),
            name: request.name,
            email: request.email,
            phone: request.phone,
            job_title: request.job_title,
            department: request.department,
            role: request.role,
            status: parse_status(&request.status)?,
            password: request.password,
        })
        .await?;

    Ok(Json(OkResponse::default()))
}

/// `PATCH /api/users/{id}`
pub async fn update(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<UpdateUserRequest>,
) -> Result<Json<UserUpdateResponse>, ApiError> {
    let result = state
        .users
        .update(UpdateUserCommand {
            actor: session.user.actor(),
            id,
            name: request.name,
            email: request.email,
            phone: request.phone,
            job_title: request.job_title,
            department: request.department,
            role: request.role,
            status: parse_status(&request.status)?,
            password: request.password,
        })
        .await?;

    Ok(Json(UserUpdateResponse {
        email_changed: result.email_changed,
    }))
}

/// `DELETE /api/users/{id}`
pub async fn delete(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    Path(id): Path<String>,
) -> Result<Json<OkResponse>, ApiError> {
    state.users.delete(&session.user.actor(), &id).await?;
    Ok(Json(OkResponse::default()))
}

/// `POST /api/users/{id}/verify`
pub async fn verify(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    Path(id): Path<String>,
) -> Result<Json<OkResponse>, ApiError> {
    state.users.verify(&session.user.actor(), &id).await?;
    Ok(Json(OkResponse::default()))
}

/// `PUT /api/users/registration-approval`
pub async fn set_registration_approval(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<RegistrationApprovalRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state
        .users
        .set_registration_approval(&session.user.actor(), request.enabled)
        .await?;
    Ok(Json(OkResponse::default()))
}

fn parse_status(value: &str) -> Result<UserStatus, ApiError> {
    UserStatus::parse(value).ok_or_else(|| {
        ApiError::from(DomainError::validation(
            "status",
            "validation.invalidStatus",
        ))
    })
}
