//! Role and permission repository implementation.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    adapter::postgres::{
        Postgres,
        rows::{PermissionRow, RoleRow, RoleWithCountRow},
    },
    app::port::{PortError, role_repository::RoleRepository},
    entity::{
        permission::{Permission, PermissionRecord},
        role::{Role, RoleWithCount, RoleWithPermissions},
    },
};

#[async_trait]
impl RoleRepository for Postgres {
    async fn list(&self) -> Result<Vec<Role>, PortError> {
        let rows = sqlx::query_as::<_, RoleRow>(
            "SELECT id, name, label, description, is_system FROM roles ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_with_permissions(&self) -> Result<Vec<RoleWithPermissions>, PortError> {
        let roles = sqlx::query_as::<_, RoleWithCountRow>(
            r"
            SELECT r.id, r.name, r.label, r.description, r.is_system,
                   (SELECT count(*) FROM users u WHERE u.role_id = r.id) AS user_count
            FROM roles r
            ORDER BY r.name
            ",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(PortError::backend)?;

        let permission_rows = sqlx::query_as::<_, (String, String)>(
            r"
            SELECT rp.role_id, p.key
            FROM role_permissions rp
            JOIN permissions p ON p.id = rp.permission_id
            ",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(PortError::backend)?;

        let mut permissions_by_role: HashMap<String, Vec<Permission>> = HashMap::new();
        for (role_id, key) in permission_rows {
            if let Some(permission) = Permission::parse(&key) {
                permissions_by_role
                    .entry(role_id)
                    .or_default()
                    .push(permission);
            }
        }

        Ok(roles
            .into_iter()
            .map(|row| {
                let permissions = permissions_by_role.remove(&row.id).unwrap_or_default();
                let count = row.user_count;

                RoleWithPermissions {
                    role: Role {
                        id: row.id,
                        name: row.name,
                        label: row.label,
                        description: row.description,
                        is_system: row.is_system,
                    },
                    permissions,
                    user_count: count,
                }
            })
            .collect())
    }

    async fn list_with_counts(&self) -> Result<Vec<RoleWithCount>, PortError> {
        let rows = sqlx::query_as::<_, RoleWithCountRow>(
            r"
            SELECT r.id, r.name, r.label, r.description, r.is_system,
                   (SELECT count(*) FROM users u WHERE u.role_id = r.id) AS user_count
            FROM roles r
            ORDER BY r.name
            ",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Role>, PortError> {
        let row = sqlx::query_as::<_, RoleRow>(
            "SELECT id, name, label, description, is_system FROM roles WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(row.map(Into::into))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, PortError> {
        let row = sqlx::query_as::<_, RoleRow>(
            "SELECT id, name, label, description, is_system FROM roles WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(row.map(Into::into))
    }

    async fn permission_ids(&self, permissions: &[Permission]) -> Result<Vec<String>, PortError> {
        if permissions.is_empty() {
            return Ok(Vec::new());
        }

        let keys = permissions
            .iter()
            .map(|permission| permission.as_str().to_owned())
            .collect::<Vec<_>>();

        sqlx::query_scalar::<_, String>("SELECT id FROM permissions WHERE key = ANY($1)")
            .bind(keys)
            .fetch_all(&self.pool)
            .await
            .map_err(PortError::backend)
    }

    async fn permissions_for_role(&self, role_id: &str) -> Result<Vec<Permission>, PortError> {
        let keys = sqlx::query_scalar::<_, String>(
            r"
            SELECT p.key
            FROM permissions p
            JOIN role_permissions rp ON rp.permission_id = p.id
            WHERE rp.role_id = $1
            ",
        )
        .bind(role_id)
        .fetch_all(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(keys
            .into_iter()
            .filter_map(|key| Permission::parse(&key))
            .collect())
    }

    async fn user_count(&self, role_id: &str) -> Result<i64, PortError> {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users WHERE role_id = $1")
            .bind(role_id)
            .fetch_one(&self.pool)
            .await
            .map_err(PortError::backend)
    }

    async fn create(&self, role: &Role, permissions: &[Permission]) -> Result<(), PortError> {
        let mut transaction = self.pool.begin().await.map_err(PortError::backend)?;

        sqlx::query(
            r"
            INSERT INTO roles (id, name, label, description, is_system)
            VALUES ($1, $2, $3, $4, $5)
            ",
        )
        .bind(&role.id)
        .bind(&role.name)
        .bind(&role.label)
        .bind(&role.description)
        .bind(role.is_system)
        .execute(&mut *transaction)
        .await
        .map_err(port_error)?;

        insert_permissions(&mut transaction, &role.id, permissions).await?;

        transaction.commit().await.map_err(PortError::backend)?;

        Ok(())
    }

    async fn update(&self, role: &Role, permissions: &[Permission]) -> Result<(), PortError> {
        let mut transaction = self.pool.begin().await.map_err(PortError::backend)?;

        let result = sqlx::query(
            r"
            UPDATE roles
            SET name = $2, label = $3, description = $4
            WHERE id = $1
            ",
        )
        .bind(&role.id)
        .bind(&role.name)
        .bind(&role.label)
        .bind(&role.description)
        .execute(&mut *transaction)
        .await
        .map_err(port_error)?;

        if result.rows_affected() == 0 {
            return Err(PortError::NotFound);
        }

        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(&role.id)
            .execute(&mut *transaction)
            .await
            .map_err(PortError::backend)?;

        insert_permissions(&mut transaction, &role.id, permissions).await?;

        transaction.commit().await.map_err(PortError::backend)?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), PortError> {
        let result = sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(PortError::backend)?;

        if result.rows_affected() == 0 {
            return Err(PortError::NotFound);
        }

        Ok(())
    }

    async fn list_permissions(&self) -> Result<Vec<PermissionRecord>, PortError> {
        let rows = sqlx::query_as::<_, PermissionRow>(
            "SELECT id, key, label, description FROM permissions ORDER BY key",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(PortError::backend)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

async fn insert_permissions(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    role_id: &str,
    permissions: &[Permission],
) -> Result<(), PortError> {
    for permission in permissions {
        sqlx::query(
            r"
            INSERT INTO role_permissions (role_id, permission_id)
            SELECT $1, id FROM permissions WHERE key = $2
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(role_id)
        .bind(permission.as_str())
        .execute(&mut **transaction)
        .await
        .map_err(PortError::backend)?;
    }

    Ok(())
}

fn port_error(error: sqlx::Error) -> PortError {
    if let sqlx::Error::Database(database) = &error
        && database.is_unique_violation()
    {
        return PortError::Conflict;
    }

    PortError::backend(error)
}
