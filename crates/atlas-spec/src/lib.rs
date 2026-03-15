//! # Atlas Spec
//!
//! Core types, contracts, and manifest definitions for the Atlas source platform.
//!
//! This crate has no OS-level dependencies and compiles for both native targets
//! (CLI, runtime) and `wasm32-unknown-unknown` (source modules).

pub mod capability;
pub mod error;
pub mod manifest;
pub mod network;
pub mod preferences;
pub mod types;

// Flat re-exports of the most commonly used items.
pub use capability::Capability;
pub use error::SourceError;
pub use manifest::SourceManifest;
pub use network::{FetchRequest, FetchResponse, HttpMethod};
pub use preferences::{PreferenceField, PreferenceFieldKind, PreferenceSchema, PreferenceValue};
pub use types::{
    Chapter, ContentRating, ContentType, Filter, FilterKind, FilterOption, Manga, MangaEntry,
    MangaStatus, Page, PageData, SearchQuery, SearchResponse, SourceInfo,
};
