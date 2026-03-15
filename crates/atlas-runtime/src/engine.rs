//! wasmi engine and store setup.

use std::sync::Arc;
use wasmi::{Engine, Store};
use crate::network::NetworkProvider;
use crate::policy::CapabilityPolicy;

/// Per-instance state accessible inside host-import callbacks.
pub struct HostState {
    pub policy: CapabilityPolicy,
    pub network: Arc<dyn NetworkProvider>,
    /// Simple in-memory cache for this instance's session.
    pub cache: std::collections::HashMap<String, (Vec<u8>, Option<u64>)>,
    /// Accumulated log messages (flushed by the caller).
    pub log_buffer: Vec<String>,
}

impl HostState {
    pub fn new(policy: CapabilityPolicy, network: Arc<dyn NetworkProvider>) -> Self {
        Self {
            policy,
            network,
            cache: Default::default(),
            log_buffer: Vec::new(),
        }
    }
}

/// Shared wasmi engine. Create once and reuse across all source instances.
pub struct AtlasEngine {
    pub(crate) engine: Engine,
}

impl AtlasEngine {
    pub fn new() -> Self {
        let mut config = wasmi::Config::default();
        // Disable features that have no use in a constrained source sandbox.
        config.wasm_tail_call(false);
        config.wasm_extended_const(false);

        Self {
            engine: Engine::new(&config),
        }
    }

    pub fn new_store(&self, state: HostState) -> Store<HostState> {
        Store::new(&self.engine, state)
    }
}

impl Default for AtlasEngine {
    fn default() -> Self { Self::new() }
}
