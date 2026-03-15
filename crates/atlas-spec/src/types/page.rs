use serde::{Deserialize, Serialize};

/// A single page within a chapter, returned by `get_pages`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// Zero-based display order index.
    pub index: u32,
    #[serde(flatten)]
    pub data: PageData,
}

/// The actual content of a page, which may be a URL, inline data, or text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum PageData {
    /// Remote image URL. The host app fetches and renders this.
    Url(String),
    /// Base64-encoded image bytes. Used when the source resolves the image itself.
    Base64(String),
    /// Plain text content. Used for novel/light-novel sources.
    Text(String),
}
