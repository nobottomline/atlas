//! Module integrity verification via SHA-256.

use sha2::{Digest, Sha256};
use crate::error::RuntimeError;

/// Verify that `wasm_bytes` matches the expected SHA-256 hex digest.
pub fn verify_sha256(wasm_bytes: &[u8], expected_hex: &str) -> Result<(), RuntimeError> {
    let actual = compute_sha256_hex(wasm_bytes);
    if actual.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(RuntimeError::IntegrityFailed {
            expected: expected_hex.to_string(),
            actual,
        })
    }
}

/// Compute the lowercase hex SHA-256 digest of `bytes`.
pub fn compute_sha256_hex(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    format!("{hash:x}")
}
