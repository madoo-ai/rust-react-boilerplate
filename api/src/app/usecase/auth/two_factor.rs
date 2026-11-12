//! TOTP enrollment, sign-in challenges and backup codes.

use chrono::Duration;

use crate::{
    app::{
        error::AppError,
        usecase::auth::{AuthService, TWO_FACTOR_LOCK_MINUTES, TWO_FACTOR_MAX_FAILURES},
    },
    entity::{
        error::DomainError,
        session::{AuthenticatedSession, TwoFactor},
        user::{UserStatus, UserWithRole},
    },
};

/// A new TOTP enrollment. Backup codes are shown once.
#[derive(Debug, Clone)]
pub struct TotpSetup {
    pub secret: String,
    pub totp_uri: String,
    pub backup_codes: Vec<String>,
}

/// Result of a completed two-factor challenge.
#[derive(Debug, Clone)]
pub struct TwoFactorVerified {
    pub session: AuthenticatedSession,
    pub token: String,
    pub trusted_device_token: Option<String>,
}

/// A TOTP challenge response.
#[derive(Debug, Clone)]
pub struct TwoFactorChallengeCommand {
    pub challenge_token: String,
    pub code: String,
    pub trust_device: bool,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuthService {
    /// Start TOTP enrollment after confirming the password.
    ///
    /// # Errors
    ///
    /// Returns `IncorrectPassword`, `TwoFactorAlreadyEnabled`, or an
    /// infrastructure error.
    pub async fn start_two_factor_enable(
        &self,
        session: &AuthenticatedSession,
        password: &str,
    ) -> Result<TotpSetup, AppError> {
        self.confirm_password(session, password).await?;

        if session.user.user.two_factor_enabled {
            return Err(DomainError::TwoFactorAlreadyEnabled.into());
        }

        let secret = self.totp.generate_secret();
        let backup_codes = (0..10)
            .map(|_| crate::app::usecase::auth::generate_backup_code())
            .collect::<Vec<_>>();
        let hashed_codes = backup_codes
            .iter()
            .map(|code| self.tokens.hash(code))
            .collect();

        let enrollment = TwoFactor {
            id: self.ids.new_id(),
            user_id: session.user.user.id.clone(),
            secret: secret.clone(),
            backup_codes: hashed_codes,
            verified: false,
            failed_verification_count: 0,
            locked_until: None,
        };

        self.two_factors.save(&enrollment).await?;

        let totp_uri = self
            .totp
            .provisioning_uri(&secret, &session.user.user.email);

        Ok(TotpSetup {
            secret,
            totp_uri,
            backup_codes,
        })
    }

    /// Finish TOTP enrollment by verifying the first code.
    ///
    /// # Errors
    ///
    /// Returns `TwoFactorNotEnabled`, `InvalidCode`, or an infrastructure
    /// error.
    pub async fn verify_two_factor_enable(
        &self,
        session: &AuthenticatedSession,
        code: &str,
    ) -> Result<(), AppError> {
        let Some(mut enrollment) = self.two_factors.find_by_user(&session.user.user.id).await?
        else {
            return Err(DomainError::TwoFactorNotEnabled.into());
        };

        if enrollment.verified {
            return Err(DomainError::TwoFactorAlreadyEnabled.into());
        }

        if !self.totp.verify(&enrollment.secret, code) {
            self.record_failure(&mut enrollment).await?;
            return Err(DomainError::InvalidCode.into());
        }

        enrollment.failed_verification_count = 0;
        enrollment.locked_until = None;
        enrollment.verified = true;
        self.two_factors.save(&enrollment).await?;
        self.users
            .set_two_factor_enabled(&session.user.user.id, true)
            .await?;

        Ok(())
    }

    /// Disable two-factor authentication after confirming the password.
    ///
    /// # Errors
    ///
    /// Returns `IncorrectPassword`, `TwoFactorNotEnabled`, or an
    /// infrastructure error.
    pub async fn disable_two_factor(
        &self,
        session: &AuthenticatedSession,
        password: &str,
    ) -> Result<(), AppError> {
        self.confirm_password(session, password).await?;

        let Some(_) = self.two_factors.find_by_user(&session.user.user.id).await? else {
            return Err(DomainError::TwoFactorNotEnabled.into());
        };

        self.two_factors.delete(&session.user.user.id).await?;
        self.users
            .set_two_factor_enabled(&session.user.user.id, false)
            .await?;
        self.verifications
            .delete_two_factor_artifacts(&session.user.user.id)
            .await?;

        Ok(())
    }

    /// Verify a TOTP code for a pending sign-in challenge.
    ///
    /// # Errors
    ///
    /// Returns `InvalidToken`, `InvalidCode` or an infrastructure error.
    pub async fn verify_two_factor_totp(
        &self,
        command: TwoFactorChallengeCommand,
    ) -> Result<TwoFactorVerified, AppError> {
        let user_id = self.challenge_user(&command.challenge_token).await?;
        let user = self.active_user_with_role(&user_id).await?;

        let Some(mut enrollment) = self.two_factors.find_by_user(&user_id).await? else {
            return Err(DomainError::TwoFactorNotEnabled.into());
        };

        let now = self.now();

        if enrollment.is_locked(now) || !enrollment.verified {
            return Err(DomainError::InvalidCode.into());
        }

        if !self.totp.verify(&enrollment.secret, command.code.as_str()) {
            self.record_failure(&mut enrollment).await?;
            return Err(DomainError::InvalidCode.into());
        }

        enrollment.failed_verification_count = 0;
        enrollment.locked_until = None;
        self.two_factors.save(&enrollment).await?;

        if !self.consume_challenge(&command.challenge_token).await? {
            return Err(DomainError::InvalidToken.into());
        }

        self.complete_challenge(
            &user,
            command.trust_device,
            command.ip_address,
            command.user_agent,
        )
        .await
    }

    /// Verify a single-use backup code for a pending sign-in challenge.
    ///
    /// # Errors
    ///
    /// Returns `InvalidToken`, `InvalidCode` or an infrastructure error.
    pub async fn verify_two_factor_backup_code(
        &self,
        command: TwoFactorChallengeCommand,
    ) -> Result<TwoFactorVerified, AppError> {
        let user_id = self.challenge_user(&command.challenge_token).await?;
        let user = self.active_user_with_role(&user_id).await?;

        let Some(mut enrollment) = self.two_factors.find_by_user(&user_id).await? else {
            return Err(DomainError::TwoFactorNotEnabled.into());
        };

        let now = self.now();

        if enrollment.is_locked(now) || !enrollment.verified {
            return Err(DomainError::InvalidCode.into());
        }

        let normalized = command.code.trim().to_lowercase();
        let hashed = self.tokens.hash(&normalized);

        let Some(index) = enrollment
            .backup_codes
            .iter()
            .position(|code| code == &hashed)
        else {
            self.record_failure(&mut enrollment).await?;
            return Err(DomainError::InvalidCode.into());
        };

        enrollment.backup_codes.remove(index);
        enrollment.failed_verification_count = 0;
        enrollment.locked_until = None;
        self.two_factors.save(&enrollment).await?;

        if !self.consume_challenge(&command.challenge_token).await? {
            return Err(DomainError::InvalidToken.into());
        }

        self.complete_challenge(
            &user,
            command.trust_device,
            command.ip_address,
            command.user_agent,
        )
        .await
    }

    /// Replace the backup code set after confirming the password.
    ///
    /// # Errors
    ///
    /// Returns `IncorrectPassword`, `TwoFactorNotEnabled`, or an
    /// infrastructure error.
    pub async fn regenerate_backup_codes(
        &self,
        session: &AuthenticatedSession,
        password: &str,
    ) -> Result<Vec<String>, AppError> {
        self.confirm_password(session, password).await?;

        let Some(_) = self.two_factors.find_by_user(&session.user.user.id).await? else {
            return Err(DomainError::TwoFactorNotEnabled.into());
        };

        let codes = (0..10)
            .map(|_| crate::app::usecase::auth::generate_backup_code())
            .collect::<Vec<_>>();
        let hashed = codes
            .iter()
            .map(|code| self.tokens.hash(code))
            .collect::<Vec<_>>();

        self.two_factors
            .update_backup_codes(&session.user.user.id, &hashed)
            .await?;

        Ok(codes)
    }

    async fn challenge_user(&self, token: &str) -> Result<String, AppError> {
        self.verifications
            .find("2fa-challenge:", &self.tokens.hash(token))
            .await?
            .ok_or_else(|| DomainError::InvalidToken.into())
    }

    async fn consume_challenge(&self, token: &str) -> Result<bool, AppError> {
        Ok(self
            .verifications
            .consume("2fa-challenge:", &self.tokens.hash(token))
            .await?
            .is_some())
    }

    async fn active_user_with_role(&self, user_id: &str) -> Result<UserWithRole, AppError> {
        let Some(user) = self.users.find_by_id(user_id).await? else {
            return Err(DomainError::UserNotFound.into());
        };

        if user.user.status != UserStatus::Active {
            return Err(match user.user.status {
                UserStatus::Pending => DomainError::AccountPending.into(),
                UserStatus::Suspended => DomainError::AccountSuspended.into(),
                UserStatus::Active => DomainError::AccountInactive.into(),
            });
        }

        Ok(user)
    }

    async fn complete_challenge(
        &self,
        user: &UserWithRole,
        trust_device: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<TwoFactorVerified, AppError> {
        let (session, token) = self
            .issue_session_for_user(user, ip_address, user_agent)
            .await?;

        let trusted_device_token = if trust_device {
            Some(self.create_trusted_device(&user.user.id).await?)
        } else {
            None
        };

        Ok(TwoFactorVerified {
            session,
            token,
            trusted_device_token,
        })
    }

    async fn record_failure(&self, enrollment: &mut TwoFactor) -> Result<(), AppError> {
        let now = self.now();

        if enrollment
            .locked_until
            .is_some_and(|locked_until| locked_until <= now)
        {
            enrollment.failed_verification_count = 0;
            enrollment.locked_until = None;
        }

        enrollment.failed_verification_count += 1;

        if enrollment.failed_verification_count >= TWO_FACTOR_MAX_FAILURES {
            enrollment.locked_until = Some(now + Duration::minutes(TWO_FACTOR_LOCK_MINUTES));
        }

        self.two_factors.save(enrollment).await?;

        Ok(())
    }
}

/// Generate a ten-character backup code from an unambiguous alphabet.
#[must_use]
pub fn generate_backup_code() -> String {
    use rand::Rng;

    const ALPHABET: &[u8] = b"23456789abcdefghjkmnpqrstuvwxyz";
    let mut rng = rand::rng();
    let mut code = String::with_capacity(10);

    for _ in 0..10 {
        let index = usize::try_from(rng.next_u32()).unwrap_or(0) % ALPHABET.len();
        code.push(char::from(ALPHABET[index]));
    }

    code
}
