//! Network provider abstraction.
//!
//! The runtime does not perform HTTP directly. Instead it delegates to a
//! `NetworkProvider` implementation. On desktop/CLI the built-in `ureq`
//! provider is used. On iOS, a Swift-implemented provider is injected via
//! FFI callback.

use atlas_spec::network::{FetchRequest, FetchResponse};
use crate::error::RuntimeError;

/// Trait that the runtime uses to execute HTTP requests on behalf of sources.
pub trait NetworkProvider: Send + Sync {
    fn fetch(&self, request: FetchRequest) -> Result<FetchResponse, RuntimeError>;
}

// ── Desktop / CLI provider (ureq) ────────────────────────────────────────────

#[cfg(not(target_os = "ios"))]
pub struct UreqNetworkProvider;

#[cfg(not(target_os = "ios"))]
impl NetworkProvider for UreqNetworkProvider {
    fn fetch(&self, request: FetchRequest) -> Result<FetchResponse, RuntimeError> {
        use std::collections::HashMap;
        use atlas_spec::network::HttpMethod;

        let method = match request.method {
            HttpMethod::Get    => "GET",
            HttpMethod::Post   => "POST",
            HttpMethod::Put    => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head   => "HEAD",
            HttpMethod::Patch  => "PATCH",
        };

        let mut req = ureq::request(method, &request.url);
        for (k, v) in &request.headers {
            req = req.set(k, v);
        }

        let response = if let Some(body) = request.body {
            req.send_bytes(&body)
        } else {
            req.call()
        }
        .map_err(|e| RuntimeError::Network(e.to_string()))?;

        let status = response.status();
        let mut headers = HashMap::new();
        // ureq doesn't expose all headers easily; collect what we can.
        for name in &["content-type", "content-length", "set-cookie"] {
            if let Some(val) = response.header(name) {
                headers.insert((*name).to_string(), val.to_string());
            }
        }

        let mut body_bytes = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut body_bytes)
            .map_err(|e| RuntimeError::Network(e.to_string()))?;

        Ok(FetchResponse {
            status,
            headers,
            body: body_bytes,
        })
    }
}

#[cfg(not(target_os = "ios"))]
impl UreqNetworkProvider {
    pub fn new() -> Self { Self }
}

#[cfg(not(target_os = "ios"))]
impl Default for UreqNetworkProvider {
    fn default() -> Self { Self::new() }
}

// ── Stub for iOS (replaced by FFI-injected provider) ─────────────────────────

#[cfg(target_os = "ios")]
pub struct NoopNetworkProvider;

#[cfg(target_os = "ios")]
impl NetworkProvider for NoopNetworkProvider {
    fn fetch(&self, _request: FetchRequest) -> Result<FetchResponse, RuntimeError> {
        Err(RuntimeError::Network(
            "no network provider configured for this target".into(),
        ))
    }
}
