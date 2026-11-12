//! HTTP cookie helpers.

use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use cookie::time::Duration;

use crate::{
    app::usecase::auth::{CHALLENGE_MINUTES, CONFIRM_MINUTES, SESSION_DAYS, TRUSTED_DEVICE_DAYS},
    handler::httpapi::state::AppState,
};

/// Set the HTTP-only session cookie.
pub fn set_session(jar: CookieJar, state: &AppState, token: &str, remember: bool) -> CookieJar {
    let mut cookie = Cookie::build((state.cookies.session_name.clone(), token.to_owned()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.cookies.secure)
        .build();

    if remember {
        cookie.set_max_age(Duration::days(SESSION_DAYS));
    }

    jar.add(cookie)
}

/// Clear the session cookie.
pub fn clear_session(jar: CookieJar, state: &AppState) -> CookieJar {
    jar.remove(Cookie::build(state.cookies.session_name.clone()).path("/"))
}

/// Set the short-lived two-factor challenge cookie.
pub fn set_two_factor(jar: CookieJar, state: &AppState, token: &str) -> CookieJar {
    let cookie = Cookie::build((state.cookies.two_factor_name.clone(), token.to_owned()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.cookies.secure)
        .max_age(Duration::minutes(CHALLENGE_MINUTES))
        .build();

    jar.add(cookie)
}

/// Clear the two-factor challenge cookie.
pub fn clear_two_factor(jar: CookieJar, state: &AppState) -> CookieJar {
    jar.remove(Cookie::build(state.cookies.two_factor_name.clone()).path("/"))
}

/// Set a trusted-device cookie.
pub fn set_trusted_device(jar: CookieJar, state: &AppState, token: &str) -> CookieJar {
    let cookie = Cookie::build((state.cookies.trusted_device_name.clone(), token.to_owned()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.cookies.secure)
        .max_age(Duration::days(TRUSTED_DEVICE_DAYS))
        .build();

    jar.add(cookie)
}

/// Set the signed confirm-password cookie.
pub fn set_confirm_password(jar: CookieJar, state: &AppState, signed: &str) -> CookieJar {
    let cookie = Cookie::build((
        state.cookies.confirm_password_name.clone(),
        signed.to_owned(),
    ))
    .path("/")
    .http_only(true)
    .same_site(SameSite::Lax)
    .secure(state.cookies.secure)
    .max_age(Duration::minutes(CONFIRM_MINUTES))
    .build();

    jar.add(cookie)
}

/// Clear the confirm-password cookie.
pub fn clear_confirm_password(jar: CookieJar, state: &AppState) -> CookieJar {
    jar.remove(Cookie::build(state.cookies.confirm_password_name.clone()).path("/"))
}
