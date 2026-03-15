//! JSON encoding/decoding and ABI value packing for WASM boundary calls.
//!
//! All host-guest communication uses JSON (serde_json) for cross-language
//! compatibility between Rust WASM modules and the Swift iOS host.
//!
//! ## Return value convention
//!
//! Every exported source function returns a single `i64`. The upper 32 bits
//! hold the pointer into guest linear memory; the lower 32 bits hold the byte
//! length. The bytes in that region are a JSON-encoded `AtlasResult<T>`.

use atlas_spec::error::SourceError;
use serde::{Deserialize, Serialize};

/// Wire envelope for all host-guest responses.
#[derive(Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum AtlasResult<T> {
    Ok(T),
    Err(SourceError),
}

/// Pack a (pointer, length) pair into a single `i64` return value.
#[inline(always)]
pub fn pack_ptr_len(ptr: *const u8, len: usize) -> i64 {
    ((ptr as u32 as i64) << 32) | (len as u32 as i64)
}

/// Encode a `Result<T, SourceError>` to JSON bytes, transfer ownership
/// to a heap allocation, and return the packed `i64` representation.
pub fn encode_result<T: Serialize>(result: Result<T, SourceError>) -> i64 {
    let envelope = match result {
        Ok(v)  => AtlasResult::Ok(v),
        Err(e) => AtlasResult::Err(e),
    };

    let bytes = serde_json::to_vec(&envelope)
        .unwrap_or_else(|e| {
            let fallback = AtlasResult::<()>::Err(SourceError::RuntimeFailure {
                message: format!("encode error: {e}"),
            });
            serde_json::to_vec(&fallback).expect("fallback encode must succeed")
        });

    let len = bytes.len();
    let ptr = bytes.as_ptr();
    core::mem::forget(bytes);
    pack_ptr_len(ptr, len)
}

/// Decode a JSON-encoded value from a raw guest-memory slice.
pub fn decode_input<T: for<'de> Deserialize<'de>>(
    ptr: *const u8,
    len: u32,
) -> Result<T, SourceError> {
    let slice = unsafe { core::slice::from_raw_parts(ptr, len as usize) };
    serde_json::from_slice(slice).map_err(|e| SourceError::InvalidResponse {
        message: format!("failed to decode input: {e}"),
    })
}

/// Decode a packed `i64` host-return value into a `Result<T, SourceError>`.
pub unsafe fn decode_host_result<T: for<'de> Deserialize<'de>>(
    packed: i64,
) -> Result<T, SourceError> {
    let ptr = ((packed as u64) >> 32) as u32 as *mut u8;
    let len = (packed & 0xFFFF_FFFF) as u32;

    if ptr.is_null() || len == 0 {
        return Err(SourceError::RuntimeFailure {
            message: "host returned a null or empty result buffer".into(),
        });
    }

    let slice = core::slice::from_raw_parts(ptr, len as usize);
    let result: AtlasResult<T> = serde_json::from_slice(slice).map_err(|e| {
        SourceError::RuntimeFailure {
            message: format!("failed to decode host result: {e}"),
        }
    })?;

    super::alloc::atlas_dealloc_impl(ptr, len);

    match result {
        AtlasResult::Ok(v)  => Ok(v),
        AtlasResult::Err(e) => Err(e),
    }
}
