//! State for the HTTP driving adapter.

use std::{path::PathBuf, sync::Arc};

use axum_extra::extract::cookie::CookieJar;
use tower_http::services::{ServeDir, ServeFile};

use crate::{
    app::{
        port::{AttemptLimiter, HealthProbe},
        usecase::{
            auth::AuthService, dashboard::DashboardService, roles::RoleService,
            settings::SettingsService, users::UserService,
        },
    },
    handler::httpapi::error::ApiError,
};

/// Allowed browser origins for state-changing requests.
#[derive(Debug, Clone)]
pub struct OriginPolicy {
    pub app_origin: String,
    pub extra: Vec<String>,
}

impl OriginPolicy {
    #[must_use]
    pub fn is_allowed(&self, origin: &str) -> bool {
        let origin = origin.trim_end_matches('/');
        origin == self.app_origin || self.extra.iter().any(|value| value == origin)
    }

    /// Header values for the CORS layer.
    #[must_use]
    pub fn allowed_origins(&self) -> Vec<axum::http::HeaderValue> {
        std::iter::once(&self.app_origin)
            .chain(self.extra.iter())
            .filter_map(|origin| origin.parse().ok())
            .collect()
    }
}

/// Cookie names and flags.
#[derive(Debug, Clone)]
pub struct CookieConfig {
    pub secure: bool,
    pub session_name: String,
    pub two_factor_name: String,
    pub trusted_device_name: String,
    pub confirm_password_name: String,
}

/// Everything the HTTP handlers need. Concrete adapters are hidden behind
/// use cases and ports.
#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub users: Arc<UserService>,
    pub roles: Arc<RoleService>,
    pub settings: Arc<SettingsService>,
    pub dashboard: Arc<DashboardService>,
    pub limiter: Arc<dyn AttemptLimiter>,
    pub health: Arc<dyn HealthProbe>,
    pub cookies: CookieConfig,
    pub origins: OriginPolicy,
    pub web_dist: PathBuf,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AppState")
            .field("cookies", &self.cookies)
            .field("origins", &self.origins)
            .field("web_dist", &self.web_dist)
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// Serve the built React application, with an SPA fallback.
    #[must_use]
    pub fn static_files(&self) -> ServeDir<ServeFile> {
        let index = self.web_dist.join("index.html");
        ServeDir::new(&self.web_dist).fallback(ServeFile::new(index))
    }

    /// Read the raw session token from a cookie jar.
    #[must_use]
    pub fn session_token(&self, jar: &CookieJar) -> Option<String> {
        jar.get(&self.cookies.session_name)
            .map(|cookie| cookie.value().to_owned())
    }

    /// Read the raw two-factor challenge token.
    #[must_use]
    pub fn two_factor_token(&self, jar: &CookieJar) -> Option<String> {
        jar.get(&self.cookies.two_factor_name)
            .map(|cookie| cookie.value().to_owned())
    }

    /// Read the raw trusted-device token.
    #[must_use]
    pub fn trusted_device_token(&self, jar: &CookieJar) -> Option<String> {
        jar.get(&self.cookies.trusted_device_name)
            .map(|cookie| cookie.value().to_owned())
    }

    /// Read the signed confirm-password cookie payload.
    #[must_use]
    pub fn confirm_value(&self, jar: &CookieJar) -> Option<String> {
        jar.get(&self.cookies.confirm_password_name)
            .map(|cookie| cookie.value().to_owned())
    }
}

/// Convert a missing session into the shared unauthorized error.
#[must_use]
pub fn unauthorized() -> ApiError {
    ApiError::from(crate::entity::error::DomainError::Unauthorized)
}
