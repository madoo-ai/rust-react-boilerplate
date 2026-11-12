//! Idempotent database seed.

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use ulid::Ulid;

use crate::{
    adapter::postgres::{Postgres, rows::StatusRow},
    entity::{
        permission::Permission,
        role::{ROLE_DEFINITIONS, RoleDefinition},
        user::UserStatus,
    },
};

/// What the seed changed.
#[derive(Debug, Clone, Copy)]
pub struct SeedSummary {
    pub permissions: usize,
    pub roles: usize,
    pub users_created: usize,
}

struct DemoUser {
    name: &'static str,
    email: &'static str,
    role: &'static str,
    status: UserStatus,
    job_title: &'static str,
    department: &'static str,
}

const DEMO_USERS: [DemoUser; 14] = [
    DemoUser {
        name: "Super Admin",
        email: "superadmin@example.com",
        role: "superadmin",
        status: UserStatus::Active,
        job_title: "Platform Owner",
        department: "Operations",
    },
    DemoUser {
        name: "HR Staff",
        email: "hr@example.com",
        role: "hr",
        status: UserStatus::Active,
        job_title: "People Operations",
        department: "People",
    },
    DemoUser {
        name: "Regular User",
        email: "user@example.com",
        role: "user",
        status: UserStatus::Active,
        job_title: "Software Engineer",
        department: "Engineering",
    },
    DemoUser {
        name: "Andi Pratama",
        email: "andi@example.com",
        role: "user",
        status: UserStatus::Active,
        job_title: "Backend Engineer",
        department: "Engineering",
    },
    DemoUser {
        name: "Siti Rahma",
        email: "siti@example.com",
        role: "hr",
        status: UserStatus::Active,
        job_title: "Recruiter",
        department: "People",
    },
    DemoUser {
        name: "Budi Santoso",
        email: "budi@example.com",
        role: "user",
        status: UserStatus::Active,
        job_title: "Finance Analyst",
        department: "Finance",
    },
    DemoUser {
        name: "Dewi Lestari",
        email: "dewi@example.com",
        role: "user",
        status: UserStatus::Active,
        job_title: "Product Designer",
        department: "Design",
    },
    DemoUser {
        name: "Rizky Hidayat",
        email: "rizky@example.com",
        role: "user",
        status: UserStatus::Active,
        job_title: "Account Executive",
        department: "Sales",
    },
    DemoUser {
        name: "Maya Putri",
        email: "maya@example.com",
        role: "user",
        status: UserStatus::Active,
        job_title: "Support Lead",
        department: "Support",
    },
    DemoUser {
        name: "Fajar Nugroho",
        email: "fajar@example.com",
        role: "user",
        status: UserStatus::Active,
        job_title: "Data Engineer",
        department: "Engineering",
    },
    DemoUser {
        name: "Nabila Syah",
        email: "nabila@example.com",
        role: "user",
        status: UserStatus::Pending,
        job_title: "Marketing Associate",
        department: "Marketing",
    },
    DemoUser {
        name: "Agus Wijaya",
        email: "agus@example.com",
        role: "user",
        status: UserStatus::Pending,
        job_title: "Operations Associate",
        department: "Operations",
    },
    DemoUser {
        name: "Putri Anggraini",
        email: "putri@example.com",
        role: "user",
        status: UserStatus::Pending,
        job_title: "QA Engineer",
        department: "Engineering",
    },
    DemoUser {
        name: "Hendra Kusuma",
        email: "hendra@example.com",
        role: "user",
        status: UserStatus::Suspended,
        job_title: "Sales Representative",
        department: "Sales",
    },
];

impl Postgres {
    /// Seed permissions, roles, the approval setting and demo users.
    ///
    /// # Errors
    ///
    /// Returns an error when the transaction fails.
    pub async fn seed(&self, password_hash: &str) -> Result<SeedSummary> {
        let mut transaction = self.pool.begin().await.context("begin seed transaction")?;

        let mut permission_ids = Vec::new();

        for permission in Permission::ALL {
            let id = sqlx::query_scalar::<_, String>(
                r"
                INSERT INTO permissions (id, key, label)
                VALUES ($1, $2, $3)
                ON CONFLICT (key) DO UPDATE SET label = EXCLUDED.label
                RETURNING id
                ",
            )
            .bind(Ulid::new().to_string())
            .bind(permission.as_str())
            .bind(permission.label())
            .fetch_one(&mut *transaction)
            .await
            .context("upsert permission")?;

            permission_ids.push(id);
        }

        let mut role_ids = std::collections::HashMap::new();

        for definition in ROLE_DEFINITIONS {
            let id = upsert_role(&mut transaction, definition).await?;

            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
                .bind(&id)
                .execute(&mut *transaction)
                .await
                .context("clear role permissions")?;

            for permission in definition.permissions {
                sqlx::query(
                    r"
                    INSERT INTO role_permissions (role_id, permission_id)
                    SELECT $1, id FROM permissions WHERE key = $2
                    ON CONFLICT DO NOTHING
                    ",
                )
                .bind(&id)
                .bind(permission.as_str())
                .execute(&mut *transaction)
                .await
                .context("insert role permission")?;
            }

            role_ids.insert(definition.name, id);
        }

        sqlx::query(
            r"
            INSERT INTO settings (id, key, value)
            VALUES ($1, 'registration_approval', 'false')
            ON CONFLICT (key) DO NOTHING
            ",
        )
        .bind(Ulid::new().to_string())
        .execute(&mut *transaction)
        .await
        .context("insert registration approval setting")?;

        let mut users_created = 0;

        for (index, demo) in DEMO_USERS.iter().enumerate() {
            let created_at = Utc::now() - Duration::days(i64::try_from(index).unwrap_or(0) * 2 + 1);
            let role_id = role_ids.get(demo.role);

            let existing = sqlx::query_scalar::<_, String>("SELECT id FROM users WHERE email = $1")
                .bind(demo.email)
                .fetch_optional(&mut *transaction)
                .await
                .context("find demo user")?;

            let user_id = if let Some(id) = existing {
                sqlx::query("UPDATE users SET created_at = $1 WHERE id = $2")
                    .bind(created_at)
                    .bind(&id)
                    .execute(&mut *transaction)
                    .await
                    .context("update demo user timestamp")?;
                id
            } else {
                let id = Ulid::new().to_string();

                sqlx::query(
                    r"
                    INSERT INTO users (
                        id, name, email, email_verified, status, role_id,
                        job_title, department, created_at
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    ",
                )
                .bind(&id)
                .bind(demo.name)
                .bind(demo.email)
                .bind(demo.status == UserStatus::Active)
                .bind(StatusRow::from(demo.status))
                .bind(role_id)
                .bind(demo.job_title)
                .bind(demo.department)
                .bind(created_at)
                .execute(&mut *transaction)
                .await
                .context("insert demo user")?;

                users_created += 1;
                id
            };

            sqlx::query(
                r"
                INSERT INTO accounts (id, provider_id, account_id, user_id, password)
                VALUES ($1, 'credential', $2, $2, $3)
                ON CONFLICT (provider_id, account_id) DO UPDATE SET password = EXCLUDED.password
                ",
            )
            .bind(Ulid::new().to_string())
            .bind(&user_id)
            .bind(password_hash)
            .execute(&mut *transaction)
            .await
            .context("upsert demo credential")?;
        }

        transaction
            .commit()
            .await
            .context("commit seed transaction")?;

        Ok(SeedSummary {
            permissions: permission_ids.len(),
            roles: role_ids.len(),
            users_created,
        })
    }
}

async fn upsert_role(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    definition: RoleDefinition,
) -> Result<String> {
    sqlx::query_scalar::<_, String>(
        r"
        INSERT INTO roles (id, name, label, description, is_system)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (name) DO UPDATE SET
            label = EXCLUDED.label,
            description = EXCLUDED.description,
            is_system = EXCLUDED.is_system
        RETURNING id
        ",
    )
    .bind(Ulid::new().to_string())
    .bind(definition.name)
    .bind(definition.label)
    .bind(definition.description)
    .bind(definition.is_system)
    .fetch_one(&mut **transaction)
    .await
    .context("upsert role")
}
