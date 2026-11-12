//! Attempt rate limiting.

/// A keyed limiter for authentication attempts.
pub trait AttemptLimiter: Send + Sync {
    /// True when the key may make another attempt.
    fn allow(&self, key: &str) -> bool;
}
