//! MessagePack encoding/decoding and ABI value packing for WASM boundary calls.
//!
//! ## Return value convention
//!
//! Every exported source function returns a single `i64`. The upper 32 bits
//! hold the pointer into guest linear memory; the lower 32 bits hold the byte
//! length. The host reads that region, decodes it, then frees it via
//! `atlas_dealloc`.
//!
//! ```text
//! i64 = (ptr as u32 as i64) << 32 | (len as u32 as i64)
//! ```
//!
//! The bytes in that region are a MessagePack-encoded `AtlasResult<T>`.

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
///
/// Upper 32 bits = pointer, lower 32 bits = byte length.
#[inline(always)]
pub fn pack_ptr_len(ptr: *const u8, len: usize) -> i64 {
    ((ptr as u32 as i64) << 32) | (len as u32 as i64)
}

/// Encode a `Result<T, SourceError>` to MessagePack bytes, transfer ownership
/// to a heap allocation, and return the packed `i64` representation.
///
/// The host is responsible for freeing the allocation via `atlas_dealloc`.
pub fn encode_result<T: Serialize>(result: Result<T, SourceError>) -> i64 {
    let envelope = match result {
        Ok(v)  => AtlasResult::Ok(v),
        Err(e) => AtlasResult::Err(e),
    };

    let bytes = rmp_serde::to_vec_named(&envelope)
        .unwrap_or_else(|e| {
            // Encoding failure: encode a RuntimeFailure error instead.
            let fallback = AtlasResult::<()>::Err(SourceError::RuntimeFailure {
                message: format!("encode error: {e}"),
            });
            rmp_serde::to_vec_named(&fallback).expect("fallback encode must succeed")
        });

    let len = bytes.len();
    let ptr = bytes.as_ptr();
    core::mem::forget(bytes);
    pack_ptr_len(ptr, len)
}

/// Decode a MessagePack-encoded value from a raw guest-memory slice.
///
/// # Safety
/// `ptr` must point to `len` valid bytes in guest linear memory.
pub fn decode_input<T: for<'de> Deserialize<'de>>(
    ptr: *const u8,
    len: u32,
) -> Result<T, SourceError> {
    let slice = unsafe { core::slice::from_raw_parts(ptr, len as usize) };
    rmp_serde::from_slice(slice).map_err(|e| SourceError::InvalidResponse {
        message: format!("failed to decode input: {e}"),
    })
}

/// Decode a packed `i64` host-return value into a `Result<T, SourceError>`.
///
/// Used by SDK host wrappers (`crate::host`) to decode host-import return values.
///
/// # Safety
/// The pointer embedded in `packed` must point to valid MessagePack bytes in
/// the guest's linear memory. The region is freed after decoding.
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
    // Host functions use JSON wire format for cross-language compatibility.
    let result: AtlasResult<T> = serde_json::from_slice(slice).map_err(|e| {
        SourceError::RuntimeFailure {
            message: format!("failed to decode host result: {e}"),
        }
    })?;

    // Free the host-allocated buffer now that we've decoded it.
    super::alloc::atlas_dealloc_impl(ptr, len);

    match result {
        AtlasResult::Ok(v)  => Ok(v),
        AtlasResult::Err(e) => Err(e),
    }
}
