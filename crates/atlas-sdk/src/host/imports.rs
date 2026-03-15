//! Raw WASM import declarations — the host functions that the runtime
//! exposes to source modules.
//!
//! These are `unsafe extern "C"` functions imported from the `"atlas"` module.
//! Source authors should never call these directly; use the safe wrappers in
//! `crate::host::http` and `crate::host::log` instead.

#[link(wasm_import_module = "atlas")]
extern "C" {
    /// Execute an HTTP/HTTPS fetch request.
    ///
    /// Input:  MessagePack-encoded `FetchRequest` at `(ptr, len)`.
    /// Output: packed `(ptr, len)` i64 pointing to MessagePack-encoded
    ///         `AtlasResult<FetchResponse>` in guest memory.
    ///
    /// Requires capability: `network.fetch`
    pub fn host_network_fetch(ptr: *const u8, len: u32) -> i64;

    /// Emit a debug log message.
    ///
    /// Input: UTF-8 string bytes at `(ptr, len)`.
    ///
    /// Requires capability: `log.debug`
    pub fn host_log_debug(ptr: *const u8, len: u32);

    /// Read the current Unix timestamp in seconds.
    ///
    /// Requires capability: `time.now`
    pub fn host_time_now() -> i64;

    /// Read a value from the host-managed cache.
    ///
    /// Input:  MessagePack-encoded cache key (`String`) at `(ptr, len)`.
    /// Output: packed `(ptr, len)` i64 pointing to MessagePack-encoded
    ///         `AtlasResult<Option<Vec<u8>>>`.
    ///
    /// Requires capability: `cache.read`
    pub fn host_cache_get(ptr: *const u8, len: u32) -> i64;

    /// Write a value to the host-managed cache.
    ///
    /// Input:  MessagePack-encoded `CacheEntry { key, value, ttl_seconds }`.
    /// Returns 0 on success, non-zero on failure.
    ///
    /// Requires capability: `cache.write`
    pub fn host_cache_set(ptr: *const u8, len: u32) -> i32;

    /// Read a user preference value.
    ///
    /// Input:  MessagePack-encoded key (`String`) at `(ptr, len)`.
    /// Output: packed `(ptr, len)` i64 pointing to MessagePack-encoded
    ///         `AtlasResult<Option<PreferenceValue>>`.
    ///
    /// Requires capability: `preferences.read`
    pub fn host_preferences_get(ptr: *const u8, len: u32) -> i64;
}
