//! Environment configuration.
//!
//! Configuration is read once at startup. Missing or malformed values fail
//! fast with a message that names the variable.

use std::{env, path::PathBuf};

use anyhow::{Context, Result, bail};
use chrono_tz::Tz;
use url::Url;

/// Locale used for server-rendered values such as dashboard date labels and
/// default mail copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
    Id,
}

impl Locale {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "en" => Some(Self::En),
            "id" => Some(Self::Id),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Id => "id",
        }
    }

    #[must_use]
    pub const fn is_indonesian(self) -> bool {
        matches!(self, Self::Id)
    }
}

use crate::adapter::smtp::SmtpConfig;

/// Validated application configuration.
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub app_url: Url,
    pub web_url: Url,
    pub app_name: String,
    pub app_locale: Locale,
    pub app_secret: String,
    pub app_time_zone: Tz,
    pub port: u16,
    pub web_dist: PathBuf,
    pub secure_cookies: bool,
    pub rate_limit: bool,
    pub run_migrations: bool,
    pub allowed_origins: Vec<String>,
    pub smtp: Option<SmtpConfig>,
    pub mail_from: String,
    pub session_cookie: String,
    pub two_factor_cookie: String,
    pub trusted_device_cookie: String,
    pub confirm_password_cookie: String,
}

impl Config {
    /// Read configuration from the process environment.
    ///
    /// # Errors
    ///
    /// Returns an error when a required variable is missing, a URL is
    /// malformed, or `APP_SECRET` is shorter than 32 bytes.
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let app_url_raw = env_or("APP_URL", "http://localhost:3000");
        let app_url = Url::parse(&app_url_raw)
            .with_context(|| format!("APP_URL is not a valid URL: {app_url_raw}"))?;

        let web_url_raw = env_or("WEB_URL", &app_url_raw);
        let web_url = Url::parse(&web_url_raw)
            .with_context(|| format!("WEB_URL is not a valid URL: {web_url_raw}"))?;

        let app_secret = required("APP_SECRET")?;
        if app_secret.len() < 32 {
            bail!("APP_SECRET must be at least 32 characters long");
        }

        let app_locale = Locale::parse(&env_or("APP_LOCALE", "id"))
            .context("APP_LOCALE must be 'en' or 'id'")?;

        let app_time_zone = env_or("APP_TIME_ZONE", "Asia/Jakarta")
            .parse::<Tz>()
            .context("APP_TIME_ZONE must be a valid IANA time zone")?;

        let port = env_or("PORT", "3000")
            .parse::<u16>()
            .context("PORT must be a valid port number")?;

        let secure_cookies = env_bool("COOKIE_SECURE", app_url.scheme() == "https");

        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.trim_end_matches('/').to_owned())
            .collect::<Vec<_>>();

        let smtp = env::var("SMTP_HOST")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|host| {
                Ok::<_, anyhow::Error>(SmtpConfig {
                    host,
                    port: env_or("SMTP_PORT", "1025")
                        .parse::<u16>()
                        .context("SMTP_PORT must be a valid port number")?,
                    user: env::var("SMTP_USER").ok().filter(|value| !value.is_empty()),
                    password: env::var("SMTP_PASSWORD")
                        .ok()
                        .filter(|value| !value.is_empty()),
                    secure: env_bool("SMTP_SECURE", false),
                })
            })
            .transpose()?;

        let cookie_prefix = env_or("COOKIE_PREFIX", "rust-boilerplate");

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            app_url,
            web_url,
            app_name: env_or("APP_NAME", "Rust React Boilerplate"),
            app_locale,
            app_secret,
            app_time_zone,
            port,
            web_dist: PathBuf::from(env_or("WEB_DIST", "web/dist")),
            secure_cookies,
            rate_limit: env_bool("RATE_LIMIT", true),
            run_migrations: env_bool("RUN_MIGRATIONS", true),
            allowed_origins,
            smtp,
            mail_from: env_or("MAIL_FROM", "hello@example.com"),
            session_cookie: format!("{cookie_prefix}.session_token"),
            two_factor_cookie: format!("{cookie_prefix}.two_factor"),
            trusted_device_cookie: format!("{cookie_prefix}.trusted_device"),
            confirm_password_cookie: "pwd_confirmed".to_owned(),
        })
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Config")
            .field("database_url", &"<redacted>")
            .field("app_url", &self.app_url)
            .field("web_url", &self.web_url)
            .field("app_name", &self.app_name)
            .field("app_locale", &self.app_locale)
            .field("app_secret", &"<redacted>")
            .field("app_time_zone", &self.app_time_zone)
            .field("port", &self.port)
            .field("web_dist", &self.web_dist)
            .field("secure_cookies", &self.secure_cookies)
            .field("rate_limit", &self.rate_limit)
            .field("run_migrations", &self.run_migrations)
            .field("allowed_origins", &self.allowed_origins)
            .field("smtp_configured", &self.smtp.is_some())
            .field("mail_from", &self.mail_from)
            .finish_non_exhaustive()
    }
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}

fn env_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name).ok().map_or(default, |value| {
        matches!(value.as_str(), "1" | "true" | "TRUE" | "yes")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_round_trips() {
        assert_eq!(Locale::parse("en"), Some(Locale::En));
        assert_eq!(Locale::parse("id"), Some(Locale::Id));
        assert_eq!(Locale::parse("fr"), None);
        assert_eq!(Locale::Id.as_str(), "id");
    }
}
