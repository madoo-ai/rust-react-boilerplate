//! Governor-backed attempt limiter.

use std::num::NonZeroU32;

use governor::{Quota, RateLimiter, clock::DefaultClock, state::keyed::DefaultKeyedStateStore};

use crate::app::port::AttemptLimiter;

type KeyedLimiter = RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>;

/// Twenty attempts per minute per key.
#[derive(Debug)]
pub struct GovernorLimiter {
    limiter: KeyedLimiter,
    enabled: bool,
}

impl GovernorLimiter {
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        let max = NonZeroU32::new(20).expect("20 is non-zero");

        Self {
            limiter: RateLimiter::keyed(Quota::per_minute(max)),
            enabled,
        }
    }
}

impl AttemptLimiter for GovernorLimiter {
    fn allow(&self, key: &str) -> bool {
        !self.enabled || self.limiter.check_key(&key.to_owned()).is_ok()
    }
}
