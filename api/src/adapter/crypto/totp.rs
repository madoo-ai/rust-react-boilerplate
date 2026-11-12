//! TOTP adapter built on `totp-rs`.

use totp_rs::{Algorithm, Builder, Secret, Totp};

use crate::app::port::TotpProvider;

/// Six-digit, thirty-second TOTP with one period of skew.
#[derive(Debug, Clone)]
pub struct TotpAdapter {
    issuer: String,
}

impl TotpAdapter {
    #[must_use]
    pub fn new(issuer: String) -> Self {
        Self { issuer }
    }

    fn build(&self, secret: &str, account_name: &str) -> Option<Totp> {
        let secret = Secret::try_from_base32(secret).ok()?;

        Builder::new()
            .with_algorithm(Algorithm::SHA1)
            .with_digits(6)
            .with_step_duration(30)
            .with_skew(1)
            .with_secret(secret)
            .with_account_name(account_name)
            .with_issuer(Some(self.issuer.clone()))
            .build()
            .ok()
    }
}

impl TotpProvider for TotpAdapter {
    fn generate_secret(&self) -> String {
        Secret::generate().to_base32()
    }

    fn provisioning_uri(&self, secret: &str, account_name: &str) -> String {
        self.build(secret, account_name)
            .and_then(|totp| totp.to_url().ok())
            .unwrap_or_default()
    }

    fn verify(&self, secret: &str, code: &str) -> bool {
        self.build(secret, "user")
            .is_some_and(|totp| totp.check_current(code).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_secret_works_with_a_current_code() {
        let adapter = TotpAdapter::new("Test App".to_owned());
        let secret = adapter.generate_secret();
        let totp = adapter.build(&secret, "user").expect("totp");
        let code = totp.generate_current().to_string();

        assert!(adapter.verify(&secret, &code));
        assert!(!adapter.verify(&secret, "000000"));
    }

    #[test]
    fn provisioning_uri_is_otpauth() {
        let adapter = TotpAdapter::new("Test App".to_owned());
        let secret = adapter.generate_secret();
        let uri = adapter.provisioning_uri(&secret, "ada@example.com");
        assert!(uri.starts_with("otpauth://totp/"));
    }
}
