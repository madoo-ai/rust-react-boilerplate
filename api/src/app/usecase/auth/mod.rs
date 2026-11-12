//! Authentication use case.
//!
//! The service owns the auth flow and depends only on entities and driven
//! ports. HTTP cookies, status codes and JSON live in the handler adapter.

pub mod password;
pub mod sign_in;
pub mod sign_up;
pub mod two_factor;
pub mod verification;

pub use two_factor::generate_backup_code;

use std::sync::Arc;

use chrono::{Duration, Utc};

use crate::{
    app::{
        error::AppError,
        port::{
            Clock, IdGenerator, Mailer, PasswordHasher, RoleRepository, SessionRepository,
            SettingsRepository, TokenService, TotpProvider, TwoFactorRepository, UserRepository,
            VerificationRepository,
        },
    },
    entity::{
        error::DomainError,
        role::USER,
        session::{AuthenticatedSession, AuthenticatedUser, Session},
        user::UserWithRole,
    },
};

/// Session lifetime in days.
pub const SESSION_DAYS: i64 = 7;
/// Email verification lifetime in hours.
pub const VERIFICATION_HOURS: i64 = 1;
/// Password reset lifetime in hours.
pub const RESET_HOURS: i64 = 1;
/// Two-factor challenge lifetime in minutes.
pub const CHALLENGE_MINUTES: i64 = 10;
/// Trusted-device lifetime in days.
pub const TRUSTED_DEVICE_DAYS: i64 = 30;
/// Recent password confirmation lifetime in minutes.
pub const CONFIRM_MINUTES: i64 = 15;
/// Failed two-factor attempts before a temporary lock.
pub const TWO_FACTOR_MAX_FAILURES: i32 = 5;
/// Two-factor lock duration in minutes.
pub const TWO_FACTOR_LOCK_MINUTES: i64 = 15;

/// Public auth configuration used by the service.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub app_name: String,
    pub app_url: String,
    pub web_url: String,
}

/// The authentication use case.
pub struct AuthService {
    pub(crate) users: Arc<dyn UserRepository>,
    pub(crate) roles: Arc<dyn RoleRepository>,
    pub(crate) sessions: Arc<dyn SessionRepository>,
    pub(crate) verifications: Arc<dyn VerificationRepository>,
    pub(crate) two_factors: Arc<dyn TwoFactorRepository>,
    pub(crate) settings: Arc<dyn SettingsRepository>,
    pub(crate) hasher: Arc<dyn PasswordHasher>,
    pub(crate) tokens: Arc<dyn TokenService>,
    pub(crate) totp: Arc<dyn TotpProvider>,
    pub(crate) mailer: Arc<dyn Mailer>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) ids: Arc<dyn IdGenerator>,
    pub(crate) config: AuthConfig,
}

impl std::fmt::Debug for AuthService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AuthService")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl AuthService {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        users: Arc<dyn UserRepository>,
        roles: Arc<dyn RoleRepository>,
        sessions: Arc<dyn SessionRepository>,
        verifications: Arc<dyn VerificationRepository>,
        two_factors: Arc<dyn TwoFactorRepository>,
        settings: Arc<dyn SettingsRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
        totp: Arc<dyn TotpProvider>,
        mailer: Arc<dyn Mailer>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
        config: AuthConfig,
    ) -> Self {
        Self {
            users,
            roles,
            sessions,
            verifications,
            two_factors,
            settings,
            hasher,
            tokens,
            totp,
            mailer,
            clock,
            ids,
            config,
        }
    }

    /// Resolve a raw session cookie token.
    ///
    /// # Errors
    ///
    /// Returns an error when a backing query fails.
    pub async fn resolve_session(
        &self,
        token: &str,
    ) -> Result<Option<AuthenticatedSession>, AppError> {
        let token_hash = self.tokens.hash(token);

        let Some(session) = self.sessions.find_by_token_hash(&token_hash).await? else {
            return Ok(None);
        };

        let now = self.clock.now();

        if session.is_expired(now) {
            self.sessions.revoke(&session.id).await?;
            return Ok(None);
        }

        let Some(user) = self.load_authenticated_user(&session.user_id).await? else {
            self.sessions.revoke(&session.id).await?;
            return Ok(None);
        };

        if !user.user.status.is_active() {
            self.sessions.revoke(&session.id).await?;
            return Ok(None);
        }

        if session.needs_touch(now) {
            self.sessions
                .touch(&session.id, now + Duration::days(SESSION_DAYS))
                .await?;
        }

        Ok(Some(AuthenticatedSession { session, user }))
    }

    /// Load the user, role and permissions for a session.
    ///
    /// # Errors
    ///
    /// Returns an error when a backing query fails.
    pub async fn load_authenticated_user(
        &self,
        user_id: &str,
    ) -> Result<Option<AuthenticatedUser>, AppError> {
        let Some(user) = self.users.find_by_id(user_id).await? else {
            return Ok(None);
        };

        let permissions = if let Some(role) = &user.role {
            self.roles.permissions_for_role(&role.id).await?
        } else {
            Vec::new()
        };

        Ok(Some(AuthenticatedUser {
            user: user.user,
            role: user.role,
            permissions,
        }))
    }

    /// Number of users with the superadmin role.
    ///
    /// # Errors
    ///
    /// Returns an error when the query fails.
    pub async fn superadmin_count(&self) -> Result<i64, AppError> {
        Ok(self
            .users
            .count_by_role(crate::entity::role::SUPERADMIN)
            .await?)
    }

    /// Whether the account may be deleted: the last superadmin may not.
    ///
    /// # Errors
    ///
    /// Returns an error when the count query fails.
    pub async fn can_delete_account(
        &self,
        session: &AuthenticatedSession,
    ) -> Result<bool, AppError> {
        if session.actor().is_superadmin() && self.superadmin_count().await? <= 1 {
            return Ok(false);
        }

        Ok(true)
    }

    pub(crate) async fn issue_session(
        &self,
        user_id: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(Session, String), AppError> {
        let now = self.clock.now();
        let session = Session {
            id: self.ids.new_id(),
            user_id: user_id.to_owned(),
            created_at: now,
            updated_at: now,
            expires_at: now + Duration::days(SESSION_DAYS),
            ip_address,
            user_agent,
        };
        let token = self.tokens.generate();
        let token_hash = self.tokens.hash(&token);

        self.sessions.create(&session, &token_hash).await?;

        Ok((session, token))
    }

    pub(crate) async fn issue_session_for_user(
        &self,
        user: &UserWithRole,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(AuthenticatedSession, String), AppError> {
        let (session, token) = self
            .issue_session(&user.user.id, ip_address, user_agent)
            .await?;

        let permissions = if let Some(role) = &user.role {
            self.roles.permissions_for_role(&role.id).await?
        } else {
            Vec::new()
        };

        let authenticated = AuthenticatedSession {
            session,
            user: AuthenticatedUser {
                user: user.user.clone(),
                role: user.role.clone(),
                permissions,
            },
        };

        Ok((authenticated, token))
    }

    pub(crate) async fn default_role(&self) -> Result<crate::entity::role::Role, AppError> {
        self.roles
            .find_by_name(USER)
            .await?
            .ok_or_else(|| AppError::unexpected(anyhow::anyhow!("default user role is missing")))
    }

    pub(crate) async fn registration_approval_required(&self) -> Result<bool, AppError> {
        Ok(self.settings.get("registration_approval").await?.as_deref() == Some("true"))
    }

    /// Current time from the clock port.
    #[must_use]
    pub fn now(&self) -> chrono::DateTime<Utc> {
        self.clock.now()
    }

    /// Build the signed value for the confirm-password cookie.
    #[must_use]
    pub fn confirmation_cookie_value(&self) -> String {
        self.tokens.sign(&self.clock.now().timestamp().to_string())
    }

    /// True when a signed confirmation is less than 15 minutes old.
    #[must_use]
    pub fn has_recent_confirmation(&self, signed: &str) -> bool {
        let Some(value) = self.tokens.verify_signed(signed) else {
            return false;
        };
        let Ok(timestamp) = value.parse::<i64>() else {
            return false;
        };

        self.clock.now().timestamp().saturating_sub(timestamp) < CONFIRM_MINUTES * 60
    }

    /// Revoke the session for a raw token. Idempotent.
    ///
    /// # Errors
    ///
    /// Returns an error when the delete fails.
    pub async fn sign_out(&self, token: &str) -> Result<(), AppError> {
        let token_hash = self.tokens.hash(token);

        if let Some(session) = self.sessions.find_by_token_hash(&token_hash).await? {
            self.sessions.revoke(&session.id).await?;
        }

        Ok(())
    }

    /// List the current user's sessions.
    ///
    /// # Errors
    ///
    /// Returns an error when the query fails.
    pub async fn list_sessions(
        &self,
        session: &AuthenticatedSession,
    ) -> Result<Vec<Session>, AppError> {
        Ok(self.sessions.list(&session.user.user.id).await?)
    }

    /// Revoke one of the current user's sessions.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` when the session does not belong to the user, or an
    /// infrastructure error.
    pub async fn revoke_session(
        &self,
        session: &AuthenticatedSession,
        session_id: &str,
    ) -> Result<(), AppError> {
        let owned = self.sessions.list(&session.user.user.id).await?;

        if !owned.iter().any(|item| item.id == session_id) {
            return Err(DomainError::NotFound.into());
        }

        self.sessions.revoke(session_id).await?;
        Ok(())
    }

    /// Revoke every other session for the current user.
    ///
    /// # Errors
    ///
    /// Returns an error when the delete fails.
    pub async fn revoke_other_sessions(
        &self,
        session: &AuthenticatedSession,
    ) -> Result<(), AppError> {
        self.sessions
            .revoke_others(&session.user.user.id, &session.session.id)
            .await?;
        Ok(())
    }
}
