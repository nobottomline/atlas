use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Structured error type returned by all source operations.
///
/// Errors cross the WASM boundary encoded as MessagePack.
/// The host maps these into user-facing categories for display.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum SourceError {
    /// An HTTP or TCP-level network failure.
    #[error("network error: {message}")]
    Network {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        status_code: Option<u16>,
    },

    /// HTML/JSON/structured-data parsing failure.
    #[error("parse error: {message}")]
    Parse { message: String },

    /// Authentication required or credentials rejected.
    #[error("authentication error: {message}")]
    Auth { message: String },

    /// The source attempted an operation it does not have permission for.
    #[error("capability denied: {capability}")]
    CapabilityDenied { capability: String },

    /// The host or source produced a response that does not match the expected schema.
    #[error("invalid response: {message}")]
    InvalidResponse { message: String },

    /// The source or runtime version is incompatible.
    #[error("compatibility error: {message}")]
    Compatibility { message: String },

    /// The operation exceeded its time limit.
    #[error("request timed out")]
    Timeout,

    /// An unrecoverable runtime failure (WASM trap, panic, etc.).
    #[error("runtime failure: {message}")]
    RuntimeFailure { message: String },

    /// The requested content was not found.
    #[error("not found")]
    NotFound,

    /// The source does not implement the requested optional operation.
    #[error("operation not supported")]
    NotSupported,

    /// The source has been disabled or revoked and cannot be used.
    #[error("source is disabled or revoked")]
    SourceUnavailable,
}

impl SourceError {
    /// True if retrying the same request might succeed.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Network { .. } | Self::Timeout)
    }

    /// True if the error is a user-facing auth prompt.
    pub fn requires_auth(&self) -> bool {
        matches!(self, Self::Auth { .. })
    }
}
