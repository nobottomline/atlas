//! # Atlas Runtime
//!
//! Host-side WASM runtime bridge for the Atlas source platform.
//!
//! ## Architecture
//!
//! ```text
//! iOS App (Swift)
//!   └── SourceRuntime module
//!         └── atlas-runtime (this crate, via C FFI / XCFramework)
//!               └── wasmi  ← pure-Rust WASM interpreter, no JIT
//!                     └── Source module (.wasm)
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use atlas_runtime::{AtlasEngine, SourceInstance};
//! use std::sync::Arc;
//!
//! let engine = AtlasEngine::new();
//! let wasm = std::fs::read("example-source.wasm")?;
//! let manifest = serde_json::from_str(&std::fs::read_to_string("manifest.json")?)?;
//!
//! #[cfg(not(target_os = "ios"))]
//! let network = Arc::new(atlas_runtime::network::UreqNetworkProvider::new());
//!
//! let mut source = SourceInstance::load(&engine, &wasm, manifest, network)?;
//! let info = source.get_info()?;
//! println!("{} v{}", info.name, info.version);
//! ```

pub mod engine;
pub mod error;
pub mod host_funcs;
pub mod instance;
pub mod integrity;
pub mod invocation;
pub mod memory;
pub mod network;
pub mod policy;

pub use engine::AtlasEngine;
pub use error::RuntimeError;
pub use instance::SourceInstance;
pub use integrity::{compute_sha256_hex, verify_sha256};
pub use network::NetworkProvider;
pub use policy::CapabilityPolicy;

/// Current Atlas runtime version, used for compatibility checks.
pub const RUNTIME_VERSION: &str = env!("CARGO_PKG_VERSION");
