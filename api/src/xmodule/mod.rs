//! Composition root.
//!
//! This is the only module that knows concrete adapters exist. It wires
//! adapters → use cases → handlers.

pub mod config;

use std::sync::Arc;

use anyhow::{Context, Result};

use crate::{
    adapter::{
        crypto::{
            Argon2Hasher, GovernorLimiter, HmacTokenService, SystemClock, TotpAdapter,
            UlidGenerator,
        },
        postgres::Postgres,
        smtp::SmtpMailer,
    },
    app::usecase::{
        auth::{AuthConfig, AuthService},
        dashboard::DashboardService,
        roles::RoleService,
        settings::SettingsService,
        users::UserService,
    },
    handler::httpapi::state::{AppState, CookieConfig, OriginPolicy},
};

pub use config::Config;

/// The wired application.
#[derive(Debug)]
pub struct Dependencies {
    pub config: Config,
    pub postgres: Arc<Postgres>,
    pub state: AppState,
}

/// Read configuration and wire every dependency.
///
/// # Errors
///
/// Returns an error when configuration, the database connection or the mailer
/// cannot be created.
pub async fn build() -> Result<Dependencies> {
    let config = Config::from_env().context("invalid configuration")?;
    let postgres = Arc::new(Postgres::connect(&config.database_url, 10).await?);

    let hasher = Arc::new(Argon2Hasher);
    let tokens = Arc::new(HmacTokenService::new(config.app_secret.clone()));
    let totp = Arc::new(TotpAdapter::new(config.app_name.clone()));
    let mailer = Arc::new(
        SmtpMailer::new(config.smtp.as_ref(), &config.mail_from, &config.app_name)
            .context("invalid mail configuration")?,
    );
    let clock = Arc::new(SystemClock);
    let ids = Arc::new(UlidGenerator);
    let limiter = Arc::new(GovernorLimiter::new(config.rate_limit));

    let auth = Arc::new(AuthService::new(
        postgres.clone(),
        postgres.clone(),
        postgres.clone(),
        postgres.clone(),
        postgres.clone(),
        postgres.clone(),
        hasher,
        tokens,
        totp,
        mailer,
        clock.clone(),
        ids.clone(),
        AuthConfig {
            app_name: config.app_name.clone(),
            app_url: config.app_url.to_string(),
            web_url: config.web_url.to_string(),
        },
    ));

    let users = Arc::new(UserService::new(
        postgres.clone(),
        postgres.clone(),
        postgres.clone(),
        Arc::new(Argon2Hasher),
        ids.clone(),
        auth.clone(),
    ));

    let roles = Arc::new(RoleService::new(postgres.clone(), ids));

    let settings = Arc::new(SettingsService::new(postgres.clone(), auth.clone()));

    let dashboard = Arc::new(DashboardService::new(
        postgres.clone(),
        clock,
        config.app_time_zone,
    ));

    let state = AppState {
        auth,
        users,
        roles,
        settings,
        dashboard,
        limiter,
        health: postgres.clone(),
        cookies: CookieConfig {
            secure: config.secure_cookies,
            session_name: config.session_cookie.clone(),
            two_factor_name: config.two_factor_cookie.clone(),
            trusted_device_name: config.trusted_device_cookie.clone(),
            confirm_password_name: config.confirm_password_cookie.clone(),
        },
        origins: OriginPolicy {
            app_origin: config.app_url.origin().ascii_serialization(),
            extra: config.allowed_origins.clone(),
        },
        web_dist: config.web_dist.clone(),
    };

    Ok(Dependencies {
        config,
        postgres,
        state,
    })
}

/// Apply the embedded migrations.
///
/// # Errors
///
/// Returns an error when a migration fails.
pub async fn migrate(postgres: &Postgres) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(postgres.pool())
        .await
        .context("migration failed")
}
