//! Registration of all host-import functions into the wasmi Linker.
//!
//! These are the Swift/native implementations of everything declared in
//! `atlas-sdk/src/host/imports.rs`. The capability policy is enforced
//! before any operation is executed.

use std::sync::Arc;
use wasmi::{Caller, Linker, Memory};
use atlas_spec::{
    capability::Capability,
    network::FetchRequest,
};
use crate::{
    engine::HostState,
    error::RuntimeError,
    invocation::{AtlasResultWire},
    memory::{read_slice, write_slice},
};

const MODULE: &str = "atlas";

/// Register all Atlas host-import functions into `linker`.
pub fn register_host_functions(
    linker: &mut Linker<HostState>,
) -> Result<(), RuntimeError> {
    register_network_fetch(linker)?;
    register_log_debug(linker)?;
    register_time_now(linker)?;
    register_cache_get(linker)?;
    register_cache_set(linker)?;
    register_preferences_get(linker)?;
    Ok(())
}

// ── network.fetch ─────────────────────────────────────────────────────────────

fn register_network_fetch(linker: &mut Linker<HostState>) -> Result<(), RuntimeError> {
    linker
        .func_wrap(MODULE, "host_network_fetch",
            |mut caller: Caller<HostState>, ptr: i32, len: i32| -> i64 {
                if let Err(e) = caller.data().policy.check(&Capability::NetworkFetch) {
                    return write_error(&mut caller, e.to_string());
                }

                let input = match read_from_guest(&caller, ptr as u32, len as u32) {
                    Ok(b) => b,
                    Err(e) => return write_error(&mut caller, e.to_string()),
                };

                // SDK sends JSON for host function requests (cross-language compat)
                let request: FetchRequest = match serde_json::from_slice(&input) {
                    Ok(r) => r,
                    Err(e) => return write_error(&mut caller, format!("decode FetchRequest: {e}")),
                };

                let network = Arc::clone(&caller.data().network);
                match network.fetch(request) {
                    // Host functions respond in JSON format
                    Ok(response) => write_ok_json(&mut caller, &response),
                    Err(e) => write_error(&mut caller, e.to_string()),
                }
            })
        .map(|_| ())
        .map_err(|e| RuntimeError::instantiation(e.to_string()))
}

// ── log.debug ─────────────────────────────────────────────────────────────────

fn register_log_debug(linker: &mut Linker<HostState>) -> Result<(), RuntimeError> {
    linker
        .func_wrap(MODULE, "host_log_debug",
            |mut caller: Caller<HostState>, ptr: i32, len: i32| {
                if !caller.data().policy.is_granted(&Capability::LogDebug) {
                    return;
                }
                if let Ok(bytes) = read_from_guest(&caller, ptr as u32, len as u32) {
                    let msg = String::from_utf8_lossy(&bytes).into_owned();
                    caller.data_mut().log_buffer.push(msg);
                }
            })
        .map(|_| ())
        .map_err(|e| RuntimeError::instantiation(e.to_string()))
}

// ── time.now ──────────────────────────────────────────────────────────────────

fn register_time_now(linker: &mut Linker<HostState>) -> Result<(), RuntimeError> {
    linker
        .func_wrap(MODULE, "host_time_now",
            |caller: Caller<HostState>| -> i64 {
                if !caller.data().policy.is_granted(&Capability::TimeNow) {
                    return 0;
                }
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
            })
        .map(|_| ())
        .map_err(|e| RuntimeError::instantiation(e.to_string()))
}

// ── cache.read ────────────────────────────────────────────────────────────────

fn register_cache_get(linker: &mut Linker<HostState>) -> Result<(), RuntimeError> {
    linker
        .func_wrap(MODULE, "host_cache_get",
            |mut caller: Caller<HostState>, ptr: i32, len: i32| -> i64 {
                if let Err(e) = caller.data().policy.check(&Capability::CacheRead) {
                    return write_error(&mut caller, e.to_string());
                }

                let key_bytes = match read_from_guest(&caller, ptr as u32, len as u32) {
                    Ok(b) => b,
                    Err(e) => return write_error(&mut caller, e.to_string()),
                };
                let key: String = match serde_json::from_slice(&key_bytes) {
                    Ok(k) => k,
                    Err(e) => return write_error(&mut caller, format!("decode cache key: {e}")),
                };

                let value: Option<Vec<u8>> = caller
                    .data()
                    .cache
                    .get(&key)
                    .map(|(v, _)| v.clone());

                write_ok(&mut caller, &value)
            })
        .map(|_| ())
        .map_err(|e| RuntimeError::instantiation(e.to_string()))
}

// ── cache.write ───────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct CacheEntry {
    key: String,
    value: Vec<u8>,
    ttl_seconds: Option<u32>,
}

fn register_cache_set(linker: &mut Linker<HostState>) -> Result<(), RuntimeError> {
    linker
        .func_wrap(MODULE, "host_cache_set",
            |mut caller: Caller<HostState>, ptr: i32, len: i32| -> i32 {
                if !caller.data().policy.is_granted(&Capability::CacheWrite) {
                    return 1;
                }
                let bytes = match read_from_guest(&caller, ptr as u32, len as u32) {
                    Ok(b) => b,
                    Err(_) => return 1,
                };
                let entry: CacheEntry = match serde_json::from_slice(&bytes) {
                    Ok(e) => e,
                    Err(_) => return 1,
                };
                let expiry = entry.ttl_seconds.map(|ttl| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() + ttl as u64)
                        .unwrap_or(0)
                });
                caller.data_mut().cache.insert(entry.key, (entry.value, expiry));
                0
            })
        .map(|_| ())
        .map_err(|e| RuntimeError::instantiation(e.to_string()))
}

// ── preferences.read ──────────────────────────────────────────────────────────

fn register_preferences_get(linker: &mut Linker<HostState>) -> Result<(), RuntimeError> {
    linker
        .func_wrap(MODULE, "host_preferences_get",
            |mut caller: Caller<HostState>, _ptr: i32, _len: i32| -> i64 {
                if let Err(e) = caller.data().policy.check(&Capability::PreferencesRead) {
                    return write_error(&mut caller, e.to_string());
                }
                // Phase 1: preferences storage not yet implemented. Return None.
                let none: Option<atlas_spec::PreferenceValue> = None;
                write_ok(&mut caller, &none)
            })
        .map(|_| ())
        .map_err(|e| RuntimeError::instantiation(e.to_string()))
}

// ── Memory helpers ────────────────────────────────────────────────────────────

fn get_memory(caller: &Caller<HostState>) -> Option<Memory> {
    caller.get_export("memory").and_then(|e| e.into_memory())
}

/// Read bytes from guest linear memory.
fn read_from_guest(caller: &Caller<HostState>, ptr: u32, len: u32) -> Result<Vec<u8>, RuntimeError> {
    let memory = get_memory(caller)
        .ok_or_else(|| RuntimeError::memory("module has no exported 'memory'"))?;
    let data = memory.data(caller);
    read_slice(data, ptr, len)
}

/// Encode `Ok(value)` as MessagePack and write it into guest memory.
/// Used for non-network host functions (cache, preferences).
fn write_ok<T: serde::Serialize>(caller: &mut Caller<HostState>, value: &T) -> i64 {
    let envelope = AtlasResultWire::ok(value.to_owned());
    let bytes = match serde_json::to_vec(&envelope) {
        Ok(b) => b,
        Err(e) => return write_error(caller, format!("encode response: {e}")),
    };
    write_to_guest(caller, bytes)
}

/// Encode `Ok(value)` as JSON and write it into guest memory.
/// Used for network host function (cross-language compatibility).
fn write_ok_json<T: serde::Serialize>(caller: &mut Caller<HostState>, value: &T) -> i64 {
    let envelope = AtlasResultWire::ok(value.to_owned());
    let bytes = match serde_json::to_vec(&envelope) {
        Ok(b) => b,
        Err(e) => return write_error_json(caller, format!("encode response: {e}")),
    };
    write_to_guest(caller, bytes)
}

/// Encode an error as MessagePack and write it into guest memory.
fn write_error(caller: &mut Caller<HostState>, msg: String) -> i64 {
    use atlas_spec::SourceError;
    let envelope = AtlasResultWire::<()>::err(SourceError::RuntimeFailure { message: msg });
    let bytes = serde_json::to_vec(&envelope)
        .unwrap_or_else(|_| b"\x81\xa6status\xa3err".to_vec());
    write_to_guest(caller, bytes)
}

/// Encode an error as JSON and write it into guest memory.
fn write_error_json(caller: &mut Caller<HostState>, msg: String) -> i64 {
    use atlas_spec::SourceError;
    let envelope = AtlasResultWire::<()>::err(SourceError::RuntimeFailure { message: msg });
    let bytes = serde_json::to_vec(&envelope)
        .unwrap_or_else(|_| br#"{"status":"err","data":{"kind":"runtime_failure","detail":{"message":"encode error"}}}"#.to_vec());
    write_to_guest(caller, bytes)
}

/// Allocate guest memory via `atlas_alloc`, write bytes, return packed i64.
fn write_to_guest(caller: &mut Caller<HostState>, bytes: Vec<u8>) -> i64 {
    let len = bytes.len() as u32;
    if len == 0 { return 0; }

    // Call atlas_alloc in the guest to get a pointer.
    let ptr = {
        let alloc_fn = match caller.get_export("atlas_alloc").and_then(|e| e.into_func()) {
            Some(f) => f,
            None => return 0,
        };
        let mut res = [wasmi::Val::I32(0)];
        if alloc_fn.call(&mut *caller, &[wasmi::Val::I32(len as i32)], &mut res).is_err() {
            return 0;
        }
        match res[0] {
            wasmi::Val::I32(p) => p as u32,
            _ => return 0,
        }
    };

    // Write bytes into guest memory.
    {
        let memory = match get_memory(caller) {
            Some(m) => m,
            None => return 0,
        };
        let data = memory.data_mut(caller);
        if write_slice(data, ptr, &bytes).is_err() {
            return 0;
        }
    }

    // Pack (ptr, len) into i64.
    ((ptr as i64) << 32) | (len as i64)
}
