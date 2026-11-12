//! HMAC token service.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use rand::RngExt;
use sha2::{Digest, Sha256};

use crate::app::port::TokenService;

type HmacSha256 = Hmac<Sha256>;

/// Opaque tokens, SHA-256 storage hashes and HMAC-signed cookie values.
#[derive(Debug, Clone)]
pub struct HmacTokenService {
    secret: String,
}

impl HmacTokenService {
    #[must_use]
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl TokenService for HmacTokenService {
    fn generate(&self) -> String {
        let mut bytes = [0_u8; 32];
        rand::rng().fill(&mut bytes);
        URL_SAFE_NO_PAD.encode(bytes)
    }

    fn hash(&self, value: &str) -> String {
        to_hex(&Sha256::digest(value.as_bytes()))
    }

    fn sign(&self, value: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .expect("HMAC accepts keys of any size");
        mac.update(value.as_bytes());
        let signature = mac.finalize().into_bytes();
        format!("{value}.{}", to_hex(&signature))
    }

    fn verify_signed<'a>(&self, signed: &'a str) -> Option<&'a str> {
        let (value, signature) = signed.rsplit_once('.')?;
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes()).ok()?;
        mac.update(value.as_bytes());
        let expected = to_hex(&mac.finalize().into_bytes());

        if constant_time_eq(expected.as_bytes(), signature.as_bytes()) {
            Some(value)
        } else {
            None
        }
    }
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }

    output
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut difference = 0_u8;
    for (left, right) in left.iter().zip(right) {
        difference |= left ^ right;
    }

    difference == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signs_and_verifies() {
        let service = HmacTokenService::new("a-secret-that-is-long-enough-for-tests".to_owned());
        let signed = service.sign("12345");
        assert_eq!(service.verify_signed(&signed), Some("12345"));
        assert_eq!(service.verify_signed("12345.deadbeef"), None);
    }

    #[test]
    fn hashes_are_stable() {
        let service = HmacTokenService::new("secret".to_owned());
        assert_eq!(service.hash("token"), service.hash("token"));
        assert_ne!(service.hash("token"), service.hash("other"));
    }
}
