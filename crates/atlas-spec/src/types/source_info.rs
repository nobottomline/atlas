use serde::{Deserialize, Serialize};
use super::manga::ContentType;
use crate::capability::Capability;

/// Static metadata about a source, returned by `get_info`.
/// This is the source's identity card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    /// Globally unique identifier, e.g. "mangadex".
    pub id: String,
    /// Display name shown in the UI.
    pub name: String,
    /// SemVer string, e.g. "1.2.0".
    pub version: String,
    /// BCP-47 language tag for the source's primary content language.
    pub lang: String,
    /// Base URLs this source fetches from (used for domain allowlist enforcement).
    pub base_urls: Vec<String>,
    pub content_type: ContentType,
    pub supports_nsfw: bool,
    /// Capabilities this source uses (must match manifest declaration).
    pub capabilities: Vec<Capability>,
    pub icon_url: Option<String>,
    pub description: Option<String>,
}
