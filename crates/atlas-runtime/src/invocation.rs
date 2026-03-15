//! Host-side call sequence for invoking source module exports.
//!
//! ## Call protocol (per exported function)
//!
//! 1. Encode the request argument as MessagePack bytes.
//! 2. Call `atlas_alloc(len)` on the guest to get a guest-side pointer.
//! 3. Write the encoded bytes into guest linear memory at that pointer.
//! 4. Call the export (e.g. `atlas_search(ptr: i32, len: i32) -> i64`).
//! 5. Unpack the `i64` return value into `(result_ptr, result_len)`.
//! 6. Read `result_len` bytes from guest memory at `result_ptr`.
//! 7. Decode the `AtlasResult<T>` envelope from MessagePack.
//! 8. Call `atlas_dealloc(result_ptr, result_len)` to free the result buffer.
//! 9. Call `atlas_dealloc(input_ptr, input_len)` to free the input buffer.

use serde::{Deserialize, Serialize};
use wasmi::{Instance, Store};
use atlas_spec::error::SourceError;
use crate::{
    engine::HostState,
    error::RuntimeError,
    memory::{read_slice, unpack_ptr_len, write_slice},
};

/// Wire envelope shared between host and guest for all call results.
#[derive(Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "snake_case")]
pub enum AtlasResultWire<T> {
    Ok(T),
    Err(SourceError),
}

impl<T: Serialize> AtlasResultWire<T> {
    pub fn ok(value: T) -> Self {
        AtlasResultWire::Ok(value)
    }
}

impl AtlasResultWire<()> {
    pub fn err(error: SourceError) -> Self {
        AtlasResultWire::Err(error)
    }
}

/// Call a parameterless export function (e.g. `atlas_get_info`, `atlas_get_filters`).
pub fn call_no_args<T>(
    store: &mut Store<HostState>,
    instance: &Instance,
    export_name: &str,
) -> Result<T, RuntimeError>
where
    T: for<'de> Deserialize<'de>,
{
    let func = instance
        .get_func(&mut *store, export_name)
        .ok_or_else(|| RuntimeError::ExportNotFound { name: export_name.into() })?;

    let mut results = [wasmi::Val::I64(0)];
    func.call(&mut *store, &[], &mut results)
        .map_err(|e| RuntimeError::trap(e.to_string()))?;

    let packed = match results[0] {
        wasmi::Val::I64(v) => v,
        _ => return Err(RuntimeError::codec("export returned unexpected type")),
    };

    read_result(&mut *store, instance, packed)
}

/// Call an export function with a single MessagePack-encoded argument.
pub fn call_with_arg<A, T>(
    store: &mut Store<HostState>,
    instance: &Instance,
    export_name: &str,
    arg: &A,
) -> Result<T, RuntimeError>
where
    A: Serialize,
    T: for<'de> Deserialize<'de>,
{
    // 1. Encode the argument.
    let arg_bytes = rmp_serde::to_vec_named(arg)
        .map_err(|e| RuntimeError::codec(format!("encode arg: {e}")))?;

    // 2. Allocate guest-side buffer.
    let arg_ptr = guest_alloc(&mut *store, instance, arg_bytes.len() as u32)?;

    // 3. Write encoded bytes into guest memory.
    {
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| RuntimeError::memory("module has no exported 'memory'"))?;
        let data = memory.data_mut(&mut *store);
        write_slice(data, arg_ptr, &arg_bytes)?;
    }

    // 4. Call the export.
    let func = instance
        .get_func(&mut *store, export_name)
        .ok_or_else(|| RuntimeError::ExportNotFound { name: export_name.into() })?;

    let mut results = [wasmi::Val::I64(0)];
    func.call(
        &mut *store,
        &[wasmi::Val::I32(arg_ptr as i32), wasmi::Val::I32(arg_bytes.len() as i32)],
        &mut results,
    )
    .map_err(|e| RuntimeError::trap(e.to_string()))?;

    // 5. Free the input buffer.
    guest_dealloc(&mut *store, instance, arg_ptr, arg_bytes.len() as u32);

    // 6-9. Read, decode, and free the result.
    let packed = match results[0] {
        wasmi::Val::I64(v) => v,
        _ => return Err(RuntimeError::codec("export returned unexpected type")),
    };

    read_result(&mut *store, instance, packed)
}

/// Read and decode the packed `(ptr, len)` result from guest memory.
fn read_result<T>(
    store: &mut Store<HostState>,
    instance: &Instance,
    packed: i64,
) -> Result<T, RuntimeError>
where
    T: for<'de> Deserialize<'de>,
{
    let (ptr, len) = unpack_ptr_len(packed);

    if ptr == 0 || len == 0 {
        return Err(RuntimeError::codec("guest returned null result pointer"));
    }

    // Read result bytes from guest memory.
    let result_bytes = {
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| RuntimeError::memory("module has no exported 'memory'"))?;
        let data = memory.data(&*store);
        read_slice(data, ptr, len)?
    };

    // Free the result buffer in guest memory.
    guest_dealloc(&mut *store, instance, ptr, len);

    // Decode the envelope.
    let envelope: AtlasResultWire<T> = rmp_serde::from_slice(&result_bytes)
        .map_err(|e| RuntimeError::codec(format!("decode result: {e}")))?;

    match envelope {
        AtlasResultWire::Ok(v)  => Ok(v),
        AtlasResultWire::Err(e) => Err(RuntimeError::Source(e)),
    }
}

// ── Guest allocator helpers ───────────────────────────────────────────────────

fn guest_alloc(
    store: &mut Store<HostState>,
    instance: &Instance,
    size: u32,
) -> Result<u32, RuntimeError> {
    let alloc = instance
        .get_func(&mut *store, "atlas_alloc")
        .ok_or_else(|| RuntimeError::ExportNotFound { name: "atlas_alloc".into() })?;

    let mut res = [wasmi::Val::I32(0)];
    alloc
        .call(&mut *store, &[wasmi::Val::I32(size as i32)], &mut res)
        .map_err(|e| RuntimeError::trap(e.to_string()))?;

    match res[0] {
        wasmi::Val::I32(p) => Ok(p as u32),
        _ => Err(RuntimeError::memory("atlas_alloc returned unexpected type")),
    }
}

fn guest_dealloc(
    store: &mut Store<HostState>,
    instance: &Instance,
    ptr: u32,
    size: u32,
) {
    if let Some(dealloc) = instance.get_func(&mut *store, "atlas_dealloc") {
        let _ = dealloc.call(
            &mut *store,
            &[wasmi::Val::I32(ptr as i32), wasmi::Val::I32(size as i32)],
            &mut [],
        );
    }
}
