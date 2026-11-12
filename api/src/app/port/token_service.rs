//! Token generation, hashing and signing.

/// Opaque tokens, their storage hashes and signed cookies.
pub trait TokenService: Send + Sync {
    /// Generate a new opaque token.
    fn generate(&self) -> String;

    /// Hash a token for storage.
    fn hash(&self, value: &str) -> String;

    /// Sign a value with the application secret.
    fn sign(&self, value: &str) -> String;

    /// Verify a signed value and return its payload.
    fn verify_signed<'a>(&self, signed: &'a str) -> Option<&'a str>;
}
