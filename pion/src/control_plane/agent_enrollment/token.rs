//! Wire enrollment token format (`ghe.<enrollment_id>.<secret>`), hashing, and comparison.

use sha2::{Digest, Sha256};

pub(super) const TOKEN_PREFIX: &str = "ghe";

pub(super) fn hex_sha256(input: &str) -> String {
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    let b = h.finalize();
    b.iter()
        .fold(String::with_capacity(b.len() * 2), |mut s, x| {
            use std::fmt::Write;
            let _ = write!(s, "{x:02x}");
            s
        })
}

pub(super) fn constant_time_eq(a: &str, b: &str) -> bool {
    use subtle::ConstantTimeEq;
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Parse `ghe.<enrollment_id>.<secret>` token wire form.
pub(super) fn parse_enrollment_wire_token(raw: &str) -> Option<(String, String)> {
    let s = raw.trim();
    let rest = s.strip_prefix(TOKEN_PREFIX)?.strip_prefix('.')?;
    let (id, secret) = rest.split_once('.')?;
    if id.is_empty() || secret.is_empty() {
        return None;
    }
    Some((id.to_string(), secret.to_string()))
}

pub(super) fn build_full_token(enrollment_id: &str, secret: &str) -> String {
    format!("{TOKEN_PREFIX}.{enrollment_id}.{secret}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roundtrip_token_format() {
        let id = "abc";
        let sec = "secret";
        let w = build_full_token(id, sec);
        let (i, s) = parse_enrollment_wire_token(&w).unwrap();
        assert_eq!(i, id);
        assert_eq!(s, sec);
    }
}
