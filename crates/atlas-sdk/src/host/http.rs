//! Safe HTTP request wrapper over the `host_network_fetch` import.

use atlas_spec::{
    error::SourceError,
    network::{FetchRequest, FetchResponse},
};
use crate::abi::codec::decode_host_result;

/// Execute an HTTP request through the host capability layer.
///
/// The host enforces domain allowlists, rate limits, and timeout rules
/// before forwarding the request. Capability `network.fetch` must be declared.
pub fn fetch(request: FetchRequest) -> Result<FetchResponse, SourceError> {
    let bytes = rmp_serde::to_vec_named(&request).map_err(|e| SourceError::RuntimeFailure {
        message: format!("failed to encode FetchRequest: {e}"),
    })?;

    let packed = unsafe {
        super::imports::host_network_fetch(bytes.as_ptr(), bytes.len() as u32)
    };

    unsafe { decode_host_result::<FetchResponse>(packed) }
}

/// Convenience: GET request. Returns the full response.
pub fn get(url: &str) -> Result<FetchResponse, SourceError> {
    fetch(FetchRequest::get(url))
}

/// Convenience: GET request, returning the response body as UTF-8 text.
pub fn get_text(url: &str) -> Result<String, SourceError> {
    get(url)?.text()
}

/// Convenience: POST request with a raw body.
pub fn post(url: &str, body: Vec<u8>) -> Result<FetchResponse, SourceError> {
    fetch(FetchRequest::post(url, body))
}

/// Convenience: POST request with a JSON body.
pub fn post_json<T: serde::Serialize>(url: &str, body: &T) -> Result<FetchResponse, SourceError> {
    let json_bytes = serde_json::to_vec(body).map_err(|e| SourceError::RuntimeFailure {
        message: format!("failed to serialize JSON body: {e}"),
    })?;
    let mut req = FetchRequest::post(url, json_bytes);
    req.headers.insert("Content-Type".into(), "application/json".into());
    fetch(req)
}
