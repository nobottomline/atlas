use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("WASM instantiation failed: {0}")]
    Instantiation(String),

    #[error("WASM execution trapped: {0}")]
    Trap(String),

    #[error("export not found: {name}")]
    ExportNotFound { name: String },

    #[error("memory access error: {0}")]
    Memory(String),

    #[error("encode/decode error: {0}")]
    Codec(String),

    #[error("capability denied: {capability}")]
    CapabilityDenied { capability: String },

    #[error("network error: {0}")]
    Network(String),

    #[error("integrity check failed: expected {expected}, got {actual}")]
    IntegrityFailed { expected: String, actual: String },

    #[error("source has been revoked")]
    Revoked,

    #[error("incompatible runtime version: source requires {required}, runtime is {current}")]
    IncompatibleVersion { required: String, current: String },

    #[error("source error: {0}")]
    Source(#[from] atlas_spec::error::SourceError),
}

impl RuntimeError {
    pub fn instantiation(msg: impl Into<String>) -> Self {
        Self::Instantiation(msg.into())
    }

    pub fn trap(msg: impl Into<String>) -> Self {
        Self::Trap(msg.into())
    }

    pub fn memory(msg: impl Into<String>) -> Self {
        Self::Memory(msg.into())
    }

    pub fn codec(msg: impl Into<String>) -> Self {
        Self::Codec(msg.into())
    }
}
