//! Integration test for the authentication use case against `PostgreSQL`.
//!
//! Run with:
//! `TEST_DATABASE_URL=postgresql://... cargo test -p rust-react-boilerplate-api --test auth_flow`
//!
//! The test skips when `TEST_DATABASE_URL` is not set, so unit runs on a
//! machine without `PostgreSQL` stay green.

use std::sync::Arc;

use api::{
    adapter::{
        crypto::{
            Argon2Hasher, HmacTokenService, SystemClock, TotpAdapter, UlidGenerator, hash_sync,
        },
        postgres::Postgres,
    },
    app::{
        error::AppError,
        port::{Mailer, PortError, mailer::AuthEmail},
        usecase::auth::{
            AuthConfig, AuthService,
            sign_in::{SignInCommand, SignInResult},
        },
    },
    entity::error::DomainError,
};
use async_trait::async_trait;

#[derive(Debug)]
struct NoopMailer;

#[async_trait]
impl Mailer for NoopMailer {
    async fn send_auth_email(
        &self,
        _kind: AuthEmail,
        _to: &str,
        _name: &str,
        _url: &str,
    ) -> Result<(), PortError> {
        Ok(())
    }
}

async fn service() -> Option<AuthService> {
    let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipping integration test: TEST_DATABASE_URL is not set");
        return None;
    };

    let postgres = Arc::new(Postgres::connect(&database_url, 4).await.expect("connect"));
    sqlx::migrate!("./migrations")
        .run(postgres.pool())
        .await
        .expect("migrate");

    let password_hash = hash_sync("password").expect("hash seed password");
    postgres.seed(&password_hash).await.expect("seed");

    let hasher = Arc::new(Argon2Hasher);
    let tokens = Arc::new(HmacTokenService::new(
        "integration-secret-that-is-at-least-32-characters".to_owned(),
    ));
    let totp = Arc::new(TotpAdapter::new("Test App".to_owned()));
    let mailer = Arc::new(NoopMailer);
    let clock = Arc::new(SystemClock);
    let ids = Arc::new(UlidGenerator);

    Some(AuthService::new(
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
        clock,
        ids,
        AuthConfig {
            app_name: "Test App".to_owned(),
            app_url: "http://localhost:3000".to_owned(),
            web_url: "http://localhost:3000".to_owned(),
        },
    ))
}

#[tokio::test]
async fn superadmin_signs_in_and_session_resolves() {
    let Some(auth) = service().await else { return };

    let result = auth
        .sign_in(SignInCommand {
            email: "superadmin@example.com".to_owned(),
            password: "password".to_owned(),
            ip_address: Some("127.0.0.1".to_owned()),
            user_agent: Some("integration-test".to_owned()),
            trusted_device_token: None,
        })
        .await
        .expect("sign in");

    let SignInResult::SignedIn { session, token } = result else {
        panic!("expected a session, got a two-factor challenge");
    };

    assert!(
        session
            .user
            .permissions
            .contains(&api::entity::permission::Permission::ManageRoles)
    );

    let resolved = auth
        .resolve_session(&token)
        .await
        .expect("resolve")
        .expect("session exists");
    assert_eq!(resolved.user.user.email, "superadmin@example.com");

    auth.sign_out(&token).await.expect("sign out");
    assert!(
        auth.resolve_session(&token)
            .await
            .expect("resolve")
            .is_none()
    );
}

#[tokio::test]
async fn pending_account_is_blocked() {
    let Some(auth) = service().await else { return };

    let error = auth
        .sign_in(SignInCommand {
            email: "nabila@example.com".to_owned(),
            password: "password".to_owned(),
            ip_address: None,
            user_agent: None,
            trusted_device_token: None,
        })
        .await
        .expect_err("pending account must be blocked");

    assert!(matches!(
        error,
        AppError::Domain(DomainError::AccountPending)
    ));
}

#[tokio::test]
async fn wrong_password_is_rejected() {
    let Some(auth) = service().await else { return };

    let error = auth
        .sign_in(SignInCommand {
            email: "superadmin@example.com".to_owned(),
            password: "definitely-wrong".to_owned(),
            ip_address: None,
            user_agent: None,
            trusted_device_token: None,
        })
        .await
        .expect_err("wrong password must fail");

    assert!(matches!(
        error,
        AppError::Domain(DomainError::InvalidCredentials)
    ));
}
