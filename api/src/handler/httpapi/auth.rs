//! Authentication HTTP handlers.

use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::{
    app::usecase::auth::{
        password::{ChangePasswordCommand, ResetPasswordCommand},
        sign_in::{SignInCommand, SignInResult},
        sign_up::SignUpCommand,
        two_factor::TwoFactorChallengeCommand,
    },
    entity::error::DomainError,
    handler::httpapi::{
        cookies,
        dto::{
            SessionDto, SessionEnvelope, SessionResponseDto, SignInResponse, SignUpResponse,
            SignUpUserDto, TwoFactorRequiredResponse, UserDto,
        },
        error::ApiError,
        extractors::{ClientInfo, CurrentSession, JsonBody, OkResponse, OptionalSession},
        state::AppState,
    },
};

#[derive(Debug, Deserialize)]
pub struct SignUpRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInRequest {
    email: String,
    password: String,
    #[serde(default = "default_true")]
    remember_me: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailRequest {
    email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    token: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordRequest {
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct CodeRequest {
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoFactorCodeRequest {
    code: String,
    #[serde(default)]
    trust_device: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordCodeRequest {
    password: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentResponse {
    sent: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsResponse {
    sessions: Vec<SessionDto>,
    current_session_id: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpSetupResponse {
    method: &'static str,
    #[serde(rename = "totpURI")]
    totp_uri: String,
    secret: String,
    backup_codes: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupCodesResponse {
    backup_codes: Vec<String>,
}

/// `GET /api/auth/session`
pub async fn session(
    State(state): State<AppState>,
    OptionalSession(session): OptionalSession,
    jar: CookieJar,
) -> Result<Json<SessionEnvelope>, ApiError> {
    let Some(session) = session else {
        return Ok(Json(SessionEnvelope { session: None }));
    };

    let password_confirmed = state
        .confirm_value(&jar)
        .is_some_and(|value| state.auth.has_recent_confirmation(&value));
    let can_delete_account = state.auth.can_delete_account(&session).await?;
    let superadmin_count = state.auth.superadmin_count().await?;

    Ok(Json(SessionEnvelope {
        session: Some(SessionResponseDto {
            session: SessionDto::from(&session.session),
            user: UserDto::from(&session),
            password_confirmed,
            can_delete_account,
            superadmin_count,
        }),
    }))
}

/// `POST /api/auth/sign-up/email`
pub async fn sign_up(
    State(state): State<AppState>,
    JsonBody(request): JsonBody<SignUpRequest>,
) -> Result<Json<SignUpResponse>, ApiError> {
    let result = state
        .auth
        .sign_up(SignUpCommand {
            name: request.name,
            email: request.email,
            password: request.password,
        })
        .await?;

    Ok(Json(SignUpResponse {
        user: SignUpUserDto {
            id: result.user.user.id,
            name: result.user.user.name,
            email: result.user.user.email,
            status: result.user.user.status,
        },
    }))
}

/// `POST /api/auth/sign-in/email`
pub async fn sign_in(
    State(state): State<AppState>,
    jar: CookieJar,
    client: ClientInfo,
    JsonBody(request): JsonBody<SignInRequest>,
) -> Result<Response, ApiError> {
    let result = state
        .auth
        .sign_in(SignInCommand {
            email: request.email,
            password: request.password,
            ip_address: client.ip,
            user_agent: client.user_agent,
            trusted_device_token: state.trusted_device_token(&jar),
        })
        .await?;

    match result {
        SignInResult::SignedIn { session, token } => {
            let jar = cookies::set_session(jar, &state, &token, request.remember_me);
            Ok((
                jar,
                Json(SignInResponse {
                    user: UserDto::from(session.as_ref()),
                }),
            )
                .into_response())
        }
        SignInResult::TwoFactorRequired { challenge_token } => {
            let jar = cookies::set_two_factor(jar, &state, &challenge_token);
            Ok((
                jar,
                Json(TwoFactorRequiredResponse {
                    two_factor_redirect: true,
                }),
            )
                .into_response())
        }
    }
}

/// `POST /api/auth/sign-out`
pub async fn sign_out(State(state): State<AppState>, jar: CookieJar) -> Result<Response, ApiError> {
    if let Some(token) = state.session_token(&jar) {
        state.auth.sign_out(&token).await?;
    }

    let jar = cookies::clear_session(jar, &state);
    Ok((jar, Json(OkResponse::default())).into_response())
}

/// `POST /api/auth/verify-email`
pub async fn verify_email(
    State(state): State<AppState>,
    JsonBody(request): JsonBody<TokenRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state.auth.verify_email(&request.token).await?;
    Ok(Json(OkResponse::default()))
}

/// `POST /api/auth/send-verification-email`
pub async fn send_verification_email(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
) -> Result<Json<SentResponse>, ApiError> {
    let sent = state.auth.resend_verification(&session).await?;
    Ok(Json(SentResponse { sent }))
}

/// `POST /api/auth/forgot-password`
pub async fn forgot_password(
    State(state): State<AppState>,
    JsonBody(request): JsonBody<EmailRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state.auth.request_password_reset(&request.email).await?;
    Ok(Json(OkResponse::default()))
}

/// `POST /api/auth/reset-password`
pub async fn reset_password(
    State(state): State<AppState>,
    JsonBody(request): JsonBody<ResetPasswordRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state
        .auth
        .reset_password(ResetPasswordCommand {
            token: request.token,
            new_password: request.new_password,
        })
        .await?;
    Ok(Json(OkResponse::default()))
}

/// `POST /api/auth/change-password`
pub async fn change_password(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<ChangePasswordRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state
        .auth
        .change_password(
            &session,
            ChangePasswordCommand {
                current_password: request.current_password,
                new_password: request.new_password,
            },
        )
        .await?;
    Ok(Json(OkResponse::default()))
}

/// `POST /api/auth/confirm-password`
pub async fn confirm_password(
    State(state): State<AppState>,
    jar: CookieJar,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<PasswordRequest>,
) -> Result<Response, ApiError> {
    state
        .auth
        .confirm_password(&session, &request.password)
        .await?;

    let jar = cookies::set_confirm_password(jar, &state, &state.auth.confirmation_cookie_value());
    Ok((jar, Json(OkResponse::default())).into_response())
}

/// `GET /api/auth/sessions`
pub async fn list_sessions(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
) -> Result<Json<SessionsResponse>, ApiError> {
    let sessions = state.auth.list_sessions(&session).await?;

    Ok(Json(SessionsResponse {
        sessions: sessions.iter().map(SessionDto::from).collect(),
        current_session_id: session.session.id,
    }))
}

/// `DELETE /api/auth/sessions/{id}`
pub async fn revoke_session(
    State(state): State<AppState>,
    jar: CookieJar,
    CurrentSession(session): CurrentSession,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    state.auth.revoke_session(&session, &id).await?;

    let jar = if id == session.session.id {
        cookies::clear_session(jar, &state)
    } else {
        jar
    };

    Ok((jar, Json(OkResponse::default())).into_response())
}

/// `POST /api/auth/sessions/revoke-others`
pub async fn revoke_other_sessions(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
) -> Result<Json<OkResponse>, ApiError> {
    state.auth.revoke_other_sessions(&session).await?;
    Ok(Json(OkResponse::default()))
}

/// `POST /api/auth/two-factor/enable`
pub async fn enable_two_factor(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<PasswordRequest>,
) -> Result<Json<TotpSetupResponse>, ApiError> {
    let setup = state
        .auth
        .start_two_factor_enable(&session, &request.password)
        .await?;

    Ok(Json(TotpSetupResponse {
        method: "totp",
        totp_uri: setup.totp_uri,
        secret: setup.secret,
        backup_codes: setup.backup_codes,
    }))
}

/// `POST /api/auth/two-factor/verify`
pub async fn verify_two_factor_enable(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<CodeRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state
        .auth
        .verify_two_factor_enable(&session, &request.code)
        .await?;
    Ok(Json(OkResponse::default()))
}

/// `POST /api/auth/two-factor/disable`
pub async fn disable_two_factor(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<PasswordRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state
        .auth
        .disable_two_factor(&session, &request.password)
        .await?;
    Ok(Json(OkResponse::default()))
}

/// `POST /api/auth/two-factor/backup-codes`
pub async fn regenerate_backup_codes(
    State(state): State<AppState>,
    CurrentSession(session): CurrentSession,
    JsonBody(request): JsonBody<PasswordCodeRequest>,
) -> Result<Json<BackupCodesResponse>, ApiError> {
    let codes = state
        .auth
        .regenerate_backup_codes(&session, &request.password)
        .await?;
    Ok(Json(BackupCodesResponse {
        backup_codes: codes,
    }))
}

/// `POST /api/auth/two-factor/verify-totp`
pub async fn verify_two_factor_totp(
    State(state): State<AppState>,
    jar: CookieJar,
    client: ClientInfo,
    JsonBody(request): JsonBody<TwoFactorCodeRequest>,
) -> Result<Response, ApiError> {
    let challenge_token = state
        .two_factor_token(&jar)
        .ok_or_else(|| ApiError::from(DomainError::InvalidToken))?;

    let verified = state
        .auth
        .verify_two_factor_totp(TwoFactorChallengeCommand {
            challenge_token,
            code: request.code,
            trust_device: request.trust_device,
            ip_address: client.ip,
            user_agent: client.user_agent,
        })
        .await?;

    Ok(finish_two_factor(&state, jar, verified))
}

/// `POST /api/auth/two-factor/verify-backup-code`
pub async fn verify_two_factor_backup_code(
    State(state): State<AppState>,
    jar: CookieJar,
    client: ClientInfo,
    JsonBody(request): JsonBody<TwoFactorCodeRequest>,
) -> Result<Response, ApiError> {
    let challenge_token = state
        .two_factor_token(&jar)
        .ok_or_else(|| ApiError::from(DomainError::InvalidToken))?;

    let verified = state
        .auth
        .verify_two_factor_backup_code(TwoFactorChallengeCommand {
            challenge_token,
            code: request.code,
            trust_device: request.trust_device,
            ip_address: client.ip,
            user_agent: client.user_agent,
        })
        .await?;

    Ok(finish_two_factor(&state, jar, verified))
}

fn finish_two_factor(
    state: &AppState,
    jar: CookieJar,
    verified: crate::app::usecase::auth::two_factor::TwoFactorVerified,
) -> Response {
    let jar = cookies::set_session(jar, state, &verified.token, true);
    let jar = cookies::clear_two_factor(jar, state);
    let jar = if let Some(token) = verified.trusted_device_token {
        cookies::set_trusted_device(jar, state, &token)
    } else {
        jar
    };

    (
        jar,
        Json(SignInResponse {
            user: UserDto::from(&verified.session),
        }),
    )
        .into_response()
}
