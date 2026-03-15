use serde::{Deserialize, Serialize};
use crate::capability::Capability;
use crate::types::manga::ContentType;

/// Full source package manifest, stored as `manifest.json` in the registry
/// and embedded alongside the WASM module in every source package.
///
/// The manifest is the authoritative descriptor for a source: its identity,
/// compatibility constraints, declared capabilities, and integrity hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceManifest {
    /// Globally unique source identifier. Lowercase, hyphen-separated.
    /// e.g. "mangadex", "manga-plus"
    pub id: String,

    /// Human-readable display name.
    pub name: String,

    /// SemVer string, e.g. "1.0.0".
    pub version: String,

    /// BCP-47 language tag for the source's primary content, e.g. "en", "ja".
    pub lang: String,

    /// Base URLs this source fetches from (enforced as domain allowlist).
    pub base_urls: Vec<String>,

    pub content_type: ContentType,

    pub supports_nsfw: bool,

    /// Filename of the compiled WASM module inside the package.
    pub module_filename: String,

    /// Lowercase hex SHA-256 digest of the WASM module bytes.
    pub module_sha256: String,

    /// Detached ed25519 signature over the SHA-256 digest, hex-encoded.
    /// None for unsigned packages (dev/local only — rejected by registry).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// Minimum Atlas runtime version required (SemVer range).
    pub min_runtime_version: String,

    /// Minimum host app version required (SemVer range), if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_app_version: Option<String>,

    /// All capabilities this source declares. The runtime enforces this list.
    pub capabilities: Vec<Capability>,

    /// Exact domains the source is allowed to contact via `network.fetch`.
    pub allowed_domains: Vec<String>,

    /// Version of the preferences schema, if the source exposes preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences_schema_version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    pub tags: Vec<String>,

    /// True if this source is deprecated and should not be newly installed.
    #[serde(default)]
    pub deprecated: bool,

    /// Source ID that supersedes this one, if deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl SourceManifest {
    /// Validate that the manifest is internally consistent.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.id.is_empty() {
            errors.push("id must not be empty".into());
        }
        if self.id.contains(|c: char| c.is_uppercase() || c == '_') {
            errors.push("id must be lowercase and use hyphens, not underscores".into());
        }
        if self.module_sha256.len() != 64 {
            errors.push("module_sha256 must be a 64-character lowercase hex string".into());
        }
        if self.base_urls.is_empty() {
            errors.push("base_urls must contain at least one URL".into());
        }
        if self.min_runtime_version.is_empty() {
            errors.push("min_runtime_version must not be empty".into());
        }
        if self.capabilities.is_empty() && self.base_urls.iter().any(|_| true) {
            // Sources that make network requests must declare network.fetch
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
