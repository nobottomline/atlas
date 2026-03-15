//! Example Atlas source module.
//!
//! This is a minimal, working demonstration of the Atlas SDK.
//! It does not connect to a real site — it returns static stub data.
//!
//! To build:
//!   cargo build --target wasm32-unknown-unknown --release -p example-source
//!
//! To run end-to-end:
//!   atlas build && atlas validate --wasm dist/example_source.wasm

use atlas_sdk::prelude::*;
use atlas_sdk::{atlas_log, export_source};

pub struct ExampleSource;

impl Default for ExampleSource {
    fn default() -> Self { Self }
}

impl Source for ExampleSource {
    fn get_info(&self) -> Result<SourceInfo, SourceError> {
        Ok(SourceInfo {
            id: "example-source".into(),
            name: "Example Source".into(),
            version: "0.1.0".into(),
            lang: "en".into(),
            base_urls: vec!["https://example.com".into()],
            content_type: ContentType::Manga,
            supports_nsfw: false,
            capabilities: vec![Capability::NetworkFetch, Capability::LogDebug],
            icon_url: None,
            description: Some("A minimal Atlas source for SDK demonstration.".into()),
        })
    }

    fn search(&self, query: SearchQuery) -> Result<SearchResponse, SourceError> {
        atlas_log!("[example-source] search called: title={:?} page={}", query.title, query.page);

        // Stub: return one entry matching any query.
        let entry = MangaEntry {
            id: "manga-1".into(),
            title: format!(
                "Example Manga (query: {})",
                query.title.as_deref().unwrap_or("*")
            ),
            url: "https://example.com/manga/1".into(),
            cover_url: Some("https://example.com/covers/1.jpg".into()),
            content_rating: ContentRating::Safe,
        };

        Ok(SearchResponse {
            entries: vec![entry],
            has_next_page: false,
            total: Some(1),
        })
    }

    fn get_manga_details(&self, id: &str) -> Result<Manga, SourceError> {
        atlas_log!("[example-source] get_manga_details: id={id}");

        if id != "manga-1" {
            return Err(SourceError::NotFound);
        }

        Ok(Manga {
            id: "manga-1".into(),
            title: "Example Manga".into(),
            url: "https://example.com/manga/1".into(),
            cover_url: Some("https://example.com/covers/1.jpg".into()),
            author: Some("Atlas SDK".into()),
            artist: Some("Atlas SDK".into()),
            description: Some("A placeholder manga for SDK testing.".into()),
            tags: vec!["action".into(), "adventure".into()],
            status: MangaStatus::Ongoing,
            content_rating: ContentRating::Safe,
            content_type: ContentType::Manga,
            lang: "en".into(),
            alt_titles: vec![],
        })
    }

    fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>, SourceError> {
        atlas_log!("[example-source] get_chapters: manga_id={manga_id}");

        if manga_id != "manga-1" {
            return Ok(vec![]);
        }

        Ok(vec![
            Chapter {
                id: "chapter-2".into(),
                manga_id: manga_id.into(),
                title: Some("Chapter 2: The Journey".into()),
                number: Some(2.0),
                volume: Some(1.0),
                lang: "en".into(),
                date_updated: Some(1_700_000_100),
                scanlator: Some("Atlas Scans".into()),
                url: "https://example.com/manga/1/chapter/2".into(),
            },
            Chapter {
                id: "chapter-1".into(),
                manga_id: manga_id.into(),
                title: Some("Chapter 1: The Beginning".into()),
                number: Some(1.0),
                volume: Some(1.0),
                lang: "en".into(),
                date_updated: Some(1_700_000_000),
                scanlator: Some("Atlas Scans".into()),
                url: "https://example.com/manga/1/chapter/1".into(),
            },
        ])
    }

    fn get_pages(&self, chapter_id: &str) -> Result<Vec<Page>, SourceError> {
        atlas_log!("[example-source] get_pages: chapter_id={chapter_id}");

        let base = format!("https://example.com/images/{chapter_id}");
        let pages = (1..=5)
            .map(|i| Page {
                index: i - 1,
                data: PageData::Url(format!("{base}/page-{i:03}.jpg")),
            })
            .collect();

        Ok(pages)
    }

    fn get_latest(&self, page: u32) -> Result<SearchResponse, SourceError> {
        atlas_log!("[example-source] get_latest: page={page}");
        self.search(SearchQuery { title: None, filters: vec![], page })
    }

    fn get_popular(&self, page: u32) -> Result<SearchResponse, SourceError> {
        atlas_log!("[example-source] get_popular: page={page}");
        self.search(SearchQuery { title: None, filters: vec![], page })
    }
}

// Generates all required WASM exports.
export_source!(ExampleSource);
