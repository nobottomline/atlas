//! # Atlas SDK
//!
//! Developer toolkit for authoring Atlas source modules in Rust.
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use atlas_sdk::prelude::*;
//!
//! pub struct MySource;
//!
//! impl Default for MySource {
//!     fn default() -> Self { Self }
//! }
//!
//! impl Source for MySource {
//!     fn get_info(&self) -> Result<SourceInfo, SourceError> {
//!         Ok(SourceInfo {
//!             id: "my-source".into(),
//!             name: "My Source".into(),
//!             version: "0.1.0".into(),
//!             lang: "en".into(),
//!             base_urls: vec!["https://example.com".into()],
//!             content_type: ContentType::Manga,
//!             supports_nsfw: false,
//!             capabilities: vec![Capability::NetworkFetch, Capability::LogDebug],
//!             icon_url: None,
//!             description: None,
//!         })
//!     }
//!
//!     fn search(&self, query: SearchQuery) -> Result<SearchResponse, SourceError> {
//!         // fetch + parse + return
//!         todo!()
//!     }
//!
//!     // ... other required methods
//! }
//!
//! // Generates all WASM exports.
//! export_source!(MySource);
//! ```

pub mod abi;
pub mod host;
pub mod source;

#[macro_use]
pub mod macros;

/// Convenient re-export of everything a source author needs.
pub mod prelude {
    pub use crate::host::{fetch, get, get_text, log_debug, post, post_json};
    pub use crate::source::Source;
    pub use atlas_spec::{
        Capability, Chapter, ContentRating, ContentType, FetchRequest, FetchResponse,
        Filter, FilterKind, FilterOption, Manga, MangaEntry, MangaStatus, Page, PageData,
        PreferenceSchema, SearchQuery, SearchResponse, SourceError, SourceInfo,
    };
}
