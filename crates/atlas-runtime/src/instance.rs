//! A loaded, instantiated source module ready for invocation.

use std::sync::Arc;
use wasmi::{Instance, Linker, Module, Store};
use atlas_spec::{
    manifest::SourceManifest,
    types::{Chapter, Manga, Page, SearchQuery, SearchResponse, SourceInfo},
    preferences::PreferenceSchema,

};
use crate::{
    engine::{AtlasEngine, HostState},
    error::RuntimeError,
    host_funcs::register_host_functions,
    invocation::{call_no_args, call_with_arg},
    network::NetworkProvider,
    policy::CapabilityPolicy,
};

/// A fully instantiated source module.
///
/// Create via `SourceInstance::load`. Each instance owns its own `Store`
/// and can be invoked independently. Thread safety depends on the caller —
/// do not share instances across threads without external synchronization.
pub struct SourceInstance {
    store: Store<HostState>,
    instance: Instance,
    pub manifest: SourceManifest,
}

impl SourceInstance {
    /// Load a WASM module from bytes and instantiate it.
    ///
    /// - `wasm_bytes`: the raw `.wasm` binary
    /// - `manifest`: the associated source manifest (used for capability policy)
    /// - `network`: injected network provider
    pub fn load(
        engine: &AtlasEngine,
        wasm_bytes: &[u8],
        manifest: SourceManifest,
        network: Arc<dyn NetworkProvider>,
    ) -> Result<Self, RuntimeError> {
        let policy = CapabilityPolicy::from_capabilities(&manifest.capabilities);
        let state = HostState::new(policy, network);

        let module = Module::new(&engine.engine, wasm_bytes)
            .map_err(|e| RuntimeError::instantiation(e.to_string()))?;

        let mut store = engine.new_store(state);
        let mut linker = Linker::<HostState>::new(&engine.engine);

        register_host_functions(&mut linker)?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| RuntimeError::instantiation(e.to_string()))?
            .start(&mut store)
            .map_err(|e| RuntimeError::instantiation(e.to_string()))?;

        Ok(Self { store, instance, manifest })
    }

    // ── Required contract ────────────────────────────────────────────────────

    pub fn get_info(&mut self) -> Result<SourceInfo, RuntimeError> {
        call_no_args(&mut self.store, &self.instance, "atlas_get_info")
    }

    pub fn search(&mut self, query: &SearchQuery) -> Result<SearchResponse, RuntimeError> {
        call_with_arg(&mut self.store, &self.instance, "atlas_search", query)
    }

    pub fn get_manga_details(&mut self, id: &str) -> Result<Manga, RuntimeError> {
        call_with_arg(&mut self.store, &self.instance, "atlas_get_manga_details", &id.to_string())
    }

    pub fn get_chapters(&mut self, manga_id: &str) -> Result<Vec<Chapter>, RuntimeError> {
        call_with_arg(&mut self.store, &self.instance, "atlas_get_chapters", &manga_id.to_string())
    }

    pub fn get_pages(&mut self, chapter_id: &str) -> Result<Vec<Page>, RuntimeError> {
        call_with_arg(&mut self.store, &self.instance, "atlas_get_pages", &chapter_id.to_string())
    }

    // ── Optional contract ────────────────────────────────────────────────────

    pub fn get_filters(&mut self) -> Result<Vec<atlas_spec::types::search::Filter>, RuntimeError> {
        call_no_args(&mut self.store, &self.instance, "atlas_get_filters")
    }

    pub fn get_latest(&mut self, page: u32) -> Result<SearchResponse, RuntimeError> {
        call_with_arg(&mut self.store, &self.instance, "atlas_get_latest", &page)
    }

    pub fn get_popular(&mut self, page: u32) -> Result<SearchResponse, RuntimeError> {
        call_with_arg(&mut self.store, &self.instance, "atlas_get_popular", &page)
    }

    pub fn get_preferences_schema(&mut self) -> Result<PreferenceSchema, RuntimeError> {
        call_no_args(&mut self.store, &self.instance, "atlas_get_preferences_schema")
    }

    // ── Diagnostics ──────────────────────────────────────────────────────────

    /// Drain and return all log messages emitted by the source since the last call.
    pub fn drain_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.store.data_mut().log_buffer)
    }
}
