use atlas_spec::{
    error::SourceError,
    preferences::PreferenceSchema,
    types::{
        search::{Filter, SearchQuery, SearchResponse},
        Chapter, Manga, Page, SourceInfo,
    },
};

/// The core contract every Atlas source module must implement.
///
/// All methods are synchronous. WASM has no native async model; I/O is
/// performed via host-imported functions (see `crate::host`), which block
/// the WASM thread until the host completes the operation.
///
/// The `export_source!` macro generates the `extern "C"` WASM exports that
/// delegate to these trait methods. Source authors only need to implement
/// this trait — they never write `extern "C"` themselves.
pub trait Source {
    // ── Required ─────────────────────────────────────────────────────────────

    /// Static metadata: id, name, version, language, base URLs, capabilities.
    fn get_info(&self) -> Result<SourceInfo, SourceError>;

    /// Execute a full-text search. `query.page` is 1-indexed.
    fn search(&self, query: SearchQuery) -> Result<SearchResponse, SourceError>;

    /// Fetch complete metadata for a single manga by its source-local ID.
    fn get_manga_details(&self, id: &str) -> Result<Manga, SourceError>;

    /// Fetch the chapter list for a manga, ordered newest-first.
    fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>, SourceError>;

    /// Resolve page image URLs/data for a chapter.
    fn get_pages(&self, chapter_id: &str) -> Result<Vec<Page>, SourceError>;

    // ── Optional ─────────────────────────────────────────────────────────────

    /// Return the filter definitions this source supports.
    fn get_filters(&self) -> Result<Vec<Filter>, SourceError> {
        Ok(Vec::new())
    }

    /// Return the latest-updated manga listing. `page` is 1-indexed.
    fn get_latest(&self, _page: u32) -> Result<SearchResponse, SourceError> {
        Err(SourceError::NotSupported)
    }

    /// Return the most popular manga listing. `page` is 1-indexed.
    fn get_popular(&self, _page: u32) -> Result<SearchResponse, SourceError> {
        Err(SourceError::NotSupported)
    }

    /// Return the preference schema for this source.
    fn get_preferences_schema(&self) -> Result<PreferenceSchema, SourceError> {
        Ok(PreferenceSchema::default())
    }

    /// Attempt to resolve a deep URL into a canonical content ID.
    /// Returns `None` if the URL is not recognizable.
    fn resolve_url(&self, _url: &str) -> Result<Option<String>, SourceError> {
        Ok(None)
    }
}
