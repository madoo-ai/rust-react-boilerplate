//! Dashboard aggregate implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    adapter::postgres::{
        Postgres,
        rows::{RoleWithCountRow, StatusRow, UserWithRoleRow},
    },
    app::port::{PortError, dashboard_repository::DashboardRepository},
    entity::{dashboard::DashboardSnapshot, user::UserStatus},
};

#[async_trait]
impl DashboardRepository for Postgres {
    async fn snapshot(&self, since: DateTime<Utc>) -> Result<DashboardSnapshot, PortError> {
        let total_users = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users");
        let active_users =
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users WHERE status = $1")
                .bind(StatusRow::from(UserStatus::Active));
        let pending_users =
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users WHERE status = $1")
                .bind(StatusRow::from(UserStatus::Pending));
        let roles = sqlx::query_as::<_, RoleWithCountRow>(
            r"
            SELECT r.id, r.name, r.label, r.description, r.is_system,
                   (SELECT count(*) FROM users u WHERE u.role_id = r.id) AS user_count
            FROM roles r
            ORDER BY r.name
            ",
        );
        let recent_users = sqlx::query_as::<_, UserWithRoleRow>(
            r"
            SELECT u.id, u.name, u.email, u.email_verified, u.image, u.role_id, u.status,
                   u.phone_number, u.job_title, u.department, u.two_factor_enabled,
                   u.created_at, u.updated_at,
                   r.id AS joined_role_id, r.name AS role_name, r.label AS role_label,
                   r.description AS role_description, r.is_system AS role_is_system
            FROM users u
            LEFT JOIN roles r ON r.id = u.role_id
            ORDER BY u.created_at DESC
            LIMIT 5
            ",
        );
        let signup_times = sqlx::query_scalar::<_, DateTime<Utc>>(
            "SELECT created_at FROM users WHERE created_at >= $1",
        )
        .bind(since);

        let (total_users, active_users, pending_users, roles, recent_users, signup_times) =
            tokio::try_join!(
                total_users.fetch_one(&self.pool),
                active_users.fetch_one(&self.pool),
                pending_users.fetch_one(&self.pool),
                roles.fetch_all(&self.pool),
                recent_users.fetch_all(&self.pool),
                signup_times.fetch_all(&self.pool),
            )
            .map_err(PortError::backend)?;

        Ok(DashboardSnapshot {
            total_users,
            active_users,
            pending_users,
            roles: roles.into_iter().map(Into::into).collect(),
            recent_users: recent_users.into_iter().map(Into::into).collect(),
            signup_times,
        })
    }
}
