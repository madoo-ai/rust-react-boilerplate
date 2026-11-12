//! Role management HTTP handlers.

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;

use crate::{
    app::usecase::roles::RoleInput,
    entity::permission::Permission,
    handler::httpapi::{
        dto::{PermissionDto, RoleWithPermissionsDto, RolesViewDto},
        error::ApiError,
        extractors::{CurrentSession, JsonBody, OkResponse},
        state::AppState,
    },
};

#[derive(Debug, Deserialize)]
pub struct RoleRequest {
    name: String,
    label: String,
    description: Option<String>,
    #[serde(default)]
    permissions: Vec<String>,
}

/// `GET /api/roles`
pub async fn list(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
) -> Result<Json<RolesViewDto>, ApiError> {
    let view = state.roles.list(&session.user.actor()).await?;

    Ok(Json(RolesViewDto {
        roles: view
            .roles
            .iter()
            .map(RoleWithPermissionsDto::from)
            .collect(),
        permissions: view
            .permissions
            .into_iter()
            .map(|permission| PermissionDto {
                id: permission.id,
                key: permission.key,
                label: permission.label,
                description: permission.description,
            })
            .collect(),
    }))
}

/// `POST /api/roles`
pub async fn create(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<RoleRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state
        .roles
        .create(&session.user.actor(), role_input(request))
        .await?;
    Ok(Json(OkResponse::default()))
}

/// `PATCH /api/roles/{id}`
pub async fn update(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    Path(id): Path<String>,
    JsonBody(request): JsonBody<RoleRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state
        .roles
        .update(&session.user.actor(), &id, role_input(request))
        .await?;
    Ok(Json(OkResponse::default()))
}

/// `DELETE /api/roles/{id}`
pub async fn delete(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    Path(id): Path<String>,
) -> Result<Json<OkResponse>, ApiError> {
    state.roles.delete(&session.user.actor(), &id).await?;
    Ok(Json(OkResponse::default()))
}

fn role_input(request: RoleRequest) -> RoleInput {
    let mut permissions = request
        .permissions
        .iter()
        .filter_map(|key| Permission::parse(key))
        .collect::<Vec<_>>();
    permissions.sort_unstable();
    permissions.dedup();

    RoleInput {
        name: request.name.trim().to_owned(),
        label: request.label.trim().to_owned(),
        description: request.description,
        permissions,
    }
}
