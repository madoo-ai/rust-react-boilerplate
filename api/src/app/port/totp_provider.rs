//! TOTP generation and verification.

/// Issues and checks RFC 6238 codes. The domain owns the six-digit,
/// thirty-second policy; the adapter performs the cryptography.
pub trait TotpProvider: Send + Sync {
    /// Generate a new base32 secret.
    fn generate_secret(&self) -> String;

    /// Build the `otpauth://` URI used by authenticator apps.
    fn provisioning_uri(&self, secret: &str, account_name: &str) -> String;

    /// Verify a six-digit code, allowing one period of clock skew.
    fn verify(&self, secret: &str, code: &str) -> bool;
}
