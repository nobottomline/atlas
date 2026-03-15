//! Helpers for reading and writing guest linear memory.
//!
//! These functions work on raw `&[u8]` / `&mut [u8]` slices extracted from
//! `wasmi::Memory`. Callers are responsible for obtaining the slice via
//! `memory.data(ctx)` / `memory.data_mut(ctx)`.

use crate::error::RuntimeError;

/// Read `len` bytes starting at `ptr` from a guest memory slice.
pub fn read_slice(data: &[u8], ptr: u32, len: u32) -> Result<Vec<u8>, RuntimeError> {
    let start = ptr as usize;
    let end = start
        .checked_add(len as usize)
        .ok_or_else(|| RuntimeError::memory("pointer overflow"))?;

    if end > data.len() {
        return Err(RuntimeError::memory(format!(
            "read out of bounds: ptr={ptr} len={len} memory_size={}",
            data.len()
        )));
    }

    Ok(data[start..end].to_vec())
}

/// Write `bytes` into a guest memory slice starting at `ptr`.
pub fn write_slice(data: &mut [u8], ptr: u32, bytes: &[u8]) -> Result<(), RuntimeError> {
    let start = ptr as usize;
    let end = start
        .checked_add(bytes.len())
        .ok_or_else(|| RuntimeError::memory("pointer overflow"))?;

    if end > data.len() {
        return Err(RuntimeError::memory(format!(
            "write out of bounds: ptr={ptr} len={} memory_size={}",
            bytes.len(),
            data.len()
        )));
    }

    data[start..end].copy_from_slice(bytes);
    Ok(())
}

/// Unpack a packed `i64` return value into `(ptr, len)`.
pub fn unpack_ptr_len(packed: i64) -> (u32, u32) {
    let ptr = ((packed as u64) >> 32) as u32;
    let len = (packed & 0xFFFF_FFFF) as u32;
    (ptr, len)
}
