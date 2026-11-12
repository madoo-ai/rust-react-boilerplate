//! User repository implementation.

use async_trait::async_trait;
use sqlx::{AssertSqlSafe, FromRow, Postgres as SqlxPostgres, QueryBuilder};
use ulid::Ulid;

use crate::{
    adapter::postgres::{Postgres, rows::UserWithRoleRow},
    app::port::{PortError, user_repository::UserRepository},
    entity::user::{
        CredentialUser, NewUser, UserFilters, UserPage, UserStatus, UserUpdate, UserWithRole,
        page_count,
    },
};

const SELECT_USER: &str = r"
    SELECT u.id, u.name, u.email, u.email_verified, u.image, u.role_id, u.status,
           u.phone_number, u.job_title, u.department, u.two_factor_enabled,
           u.created_at, u.updated_at,
           r.id AS joined_role_id, r.name AS role_name, r.label AS role_label,
           r.description AS role_description, r.is_system AS role_is_system
    FROM users u
    LEFT JOIN roles r ON r.id = u.role_id
";

#[derive(Debug, FromRow)]
struct CredentialRow {
    #[sqlx(flatten)]
    user: UserWithRoleRow,
    password: Option<String>,
}

#[async_trait]
impl UserRepository for Postgres {
    async fn find_by_id(&self, id: &str) -> Result<Option<UserWithRole>, PortError> {
        let row = sqlx::query_as::<_, UserWithRoleRow>(AssertSqlSafe(format!(
            "{SELECT_USER} WHERE u.id = $1"
        )))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(port_error)?;

        Ok(row.map(Into::into))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<UserWithRole>, PortError> {
        let row = sqlx::query_as::<_, UserWithRoleRow>(AssertSqlSafe(format!(
            "{SELECT_USER} WHERE lower(u.email) = lower($1)"
        )))
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(port_error)?;

        Ok(row.map(Into::into))
    }

    async fn find_credentials_by_email(
        &self,
        email: &str,
    ) -> Result<Option<CredentialUser>, PortError> {
        let row = sqlx::query_as::<_, CredentialRow>(
            r"
            SELECT u.id, u.name, u.email, u.email_verified, u.image, u.role_id, u.status,
                   u.phone_number, u.job_title, u.department, u.two_factor_enabled,
                   u.created_at, u.updated_at,
                   r.id AS joined_role_id, r.name AS role_name, r.label AS role_label,
                   r.description AS role_description, r.is_system AS role_is_system,
                   a.password
            FROM users u
            LEFT JOIN roles r ON r.id = u.role_id
            LEFT JOIN accounts a ON a.user_id = u.id AND a.provider_id = 'credential'
            WHERE lower(u.email) = lower($1)
            ",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(port_error)?;

        Ok(row.map(|row| CredentialUser {
            user: row.user.into(),
            password_hash: row.password,
        }))
    }

    async fn find_credentials_by_id(&self, id: &str) -> Result<Option<CredentialUser>, PortError> {
        let row = sqlx::query_as::<_, CredentialRow>(
            r"
            SELECT u.id, u.name, u.email, u.email_verified, u.image, u.role_id, u.status,
                   u.phone_number, u.job_title, u.department, u.two_factor_enabled,
                   u.created_at, u.updated_at,
                   r.id AS joined_role_id, r.name AS role_name, r.label AS role_label,
                   r.description AS role_description, r.is_system AS role_is_system,
                   a.password
            FROM users u
            LEFT JOIN roles r ON r.id = u.role_id
            LEFT JOIN accounts a ON a.user_id = u.id AND a.provider_id = 'credential'
            WHERE u.id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(port_error)?;

        Ok(row.map(|row| CredentialUser {
            user: row.user.into(),
            password_hash: row.password,
        }))
    }

    async fn email_exists(&self, email: &str, exclude_id: Option<&str>) -> Result<bool, PortError> {
        sqlx::query_scalar::<_, bool>(
            r"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE lower(email) = lower($1)
                  AND ($2::text IS NULL OR id <> $2)
            )
            ",
        )
        .bind(email)
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await
        .map_err(port_error)
    }

    async fn list(&self, filters: &UserFilters) -> Result<UserPage, PortError> {
        let mut count = QueryBuilder::<SqlxPostgres>::new(
            "SELECT count(*) FROM users u LEFT JOIN roles r ON r.id = u.role_id WHERE TRUE",
        );
        apply_filters(&mut count, filters);

        let total = count
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_err(port_error)?;

        let page_count = page_count(total);
        let page = filters.normalized_page(total);

        let mut query = QueryBuilder::<SqlxPostgres>::new(SELECT_USER);
        query.push(" WHERE TRUE");
        apply_filters(&mut query, filters);
        query.push(" ORDER BY u.created_at DESC LIMIT ");
        query.push_bind(crate::entity::user::PAGE_SIZE);
        query.push(" OFFSET ");
        query.push_bind((page - 1) * crate::entity::user::PAGE_SIZE);

        let rows = query
            .build_query_as::<UserWithRoleRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(port_error)?;

        Ok(UserPage {
            users: rows.into_iter().map(Into::into).collect(),
            total,
            page,
            page_count,
        })
    }

    async fn create(&self, user: &NewUser, password_hash: &str) -> Result<(), PortError> {
        let mut transaction = self.pool.begin().await.map_err(port_error)?;

        sqlx::query(
            r"
            INSERT INTO users (
                id, name, email, email_verified, status, role_id,
                phone_number, job_title, department
            )
            VALUES ($1, $2, $3, false, $4, $5, $6, $7, $8)
            ",
        )
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.email)
        .bind(crate::adapter::postgres::rows::StatusRow::from(user.status))
        .bind(&user.role_id)
        .bind(&user.phone_number)
        .bind(&user.job_title)
        .bind(&user.department)
        .execute(&mut *transaction)
        .await
        .map_err(port_error)?;

        sqlx::query(
            r"
            INSERT INTO accounts (id, provider_id, account_id, user_id, password)
            VALUES ($1, 'credential', $2, $2, $3)
            ",
        )
        .bind(Ulid::new().to_string())
        .bind(&user.id)
        .bind(password_hash)
        .execute(&mut *transaction)
        .await
        .map_err(port_error)?;

        transaction.commit().await.map_err(port_error)?;

        Ok(())
    }

    async fn update(&self, update: &UserUpdate) -> Result<(), PortError> {
        let mut transaction = self.pool.begin().await.map_err(port_error)?;

        let result = sqlx::query(
            r"
            UPDATE users
            SET name = $2, email = $3, email_verified = $4, status = $5,
                role_id = $6, phone_number = $7, job_title = $8, department = $9
            WHERE id = $1
            ",
        )
        .bind(&update.id)
        .bind(&update.name)
        .bind(&update.email)
        .bind(update.email_verified)
        .bind(crate::adapter::postgres::rows::StatusRow::from(
            update.status,
        ))
        .bind(&update.role_id)
        .bind(&update.phone_number)
        .bind(&update.job_title)
        .bind(&update.department)
        .execute(&mut *transaction)
        .await
        .map_err(port_error)?;

        if result.rows_affected() == 0 {
            return Err(PortError::NotFound);
        }

        if let Some(password_hash) = &update.password_hash {
            let updated = sqlx::query(
                r"
                UPDATE accounts SET password = $1
                WHERE user_id = $2 AND provider_id = 'credential'
                ",
            )
            .bind(password_hash)
            .bind(&update.id)
            .execute(&mut *transaction)
            .await
            .map_err(port_error)?;

            if updated.rows_affected() == 0 {
                sqlx::query(
                    r"
                    INSERT INTO accounts (id, provider_id, account_id, user_id, password)
                    VALUES ($1, 'credential', $2, $2, $3)
                    ",
                )
                .bind(Ulid::new().to_string())
                .bind(&update.id)
                .bind(password_hash)
                .execute(&mut *transaction)
                .await
                .map_err(port_error)?;
            }
        }

        if update.revoke_sessions {
            sqlx::query("DELETE FROM sessions WHERE user_id = $1")
                .bind(&update.id)
                .execute(&mut *transaction)
                .await
                .map_err(port_error)?;
        }

        transaction.commit().await.map_err(port_error)?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), PortError> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(port_error)?;

        if result.rows_affected() == 0 {
            return Err(PortError::NotFound);
        }

        Ok(())
    }

    async fn mark_email_verified(&self, id: &str) -> Result<(), PortError> {
        let result = sqlx::query("UPDATE users SET email_verified = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(port_error)?;

        if result.rows_affected() == 0 {
            return Err(PortError::NotFound);
        }

        Ok(())
    }

    async fn set_status(&self, id: &str, status: UserStatus) -> Result<(), PortError> {
        let result = sqlx::query("UPDATE users SET status = $1 WHERE id = $2")
            .bind(crate::adapter::postgres::rows::StatusRow::from(status))
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(port_error)?;

        if result.rows_affected() == 0 {
            return Err(PortError::NotFound);
        }

        Ok(())
    }

    async fn update_password(&self, id: &str, password_hash: &str) -> Result<(), PortError> {
        let updated = sqlx::query(
            r"
            UPDATE accounts SET password = $1
            WHERE user_id = $2 AND provider_id = 'credential'
            ",
        )
        .bind(password_hash)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(port_error)?;

        if updated.rows_affected() == 0 {
            sqlx::query(
                r"
                INSERT INTO accounts (id, provider_id, account_id, user_id, password)
                VALUES ($1, 'credential', $2, $2, $3)
                ",
            )
            .bind(Ulid::new().to_string())
            .bind(id)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map_err(port_error)?;
        }

        Ok(())
    }

    async fn set_two_factor_enabled(&self, id: &str, enabled: bool) -> Result<(), PortError> {
        let result = sqlx::query("UPDATE users SET two_factor_enabled = $1 WHERE id = $2")
            .bind(enabled)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(port_error)?;

        if result.rows_affected() == 0 {
            return Err(PortError::NotFound);
        }

        Ok(())
    }

    async fn count_by_role(&self, role_name: &str) -> Result<i64, PortError> {
        sqlx::query_scalar::<_, i64>(
            r"
            SELECT count(*) FROM users u
            JOIN roles r ON r.id = u.role_id
            WHERE r.name = $1
            ",
        )
        .bind(role_name)
        .fetch_one(&self.pool)
        .await
        .map_err(port_error)
    }

    async fn list_all_with_roles(&self) -> Result<Vec<UserWithRole>, PortError> {
        let rows = sqlx::query_as::<_, UserWithRoleRow>(AssertSqlSafe(format!(
            "{SELECT_USER} ORDER BY u.name"
        )))
        .fetch_all(&self.pool)
        .await
        .map_err(port_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

fn apply_filters(query: &mut QueryBuilder<SqlxPostgres>, filters: &UserFilters) {
    if let Some(search) = &filters.search {
        query.push(" AND (u.name ILIKE ");
        query.push_bind(format!("%{}%", escape_like(search)));
        query.push(" OR u.email ILIKE ");
        query.push_bind(format!("%{}%", escape_like(search)));
        query.push(")");
    }

    if let Some(role) = &filters.role {
        query.push(" AND r.name = ");
        query.push_bind(role.clone());
    }

    if let Some(status) = filters.status {
        query.push(" AND u.status = ");
        query.push_bind(crate::adapter::postgres::rows::StatusRow::from(status));
    }
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn port_error(error: sqlx::Error) -> PortError {
    if let sqlx::Error::Database(database) = &error
        && database.is_unique_violation()
    {
        return PortError::Conflict;
    }

    PortError::backend(error)
}
