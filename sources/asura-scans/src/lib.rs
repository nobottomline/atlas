//! Asura Scans — real Atlas source for asuracomic.net
//!
//! Parses HTML from the site to extract manga listings, details,
//! chapters, and page images.

mod parser;

use atlas_sdk::prelude::*;
use atlas_sdk::{atlas_log, export_source};

const BASE_URL: &str = "https://asuracomic.net";
const MEDIA_URL: &str = "https://gg.asuracomic.net/storage/media";

pub struct AsuraScans;

impl Default for AsuraScans {
    fn default() -> Self { Self }
}

impl Source for AsuraScans {
    fn get_info(&self) -> Result<SourceInfo, SourceError> {
        Ok(SourceInfo {
            id: "asura-scans".into(),
            name: "Asura Scans".into(),
            version: "0.1.0".into(),
            lang: "en".into(),
            base_urls: vec![BASE_URL.into()],
            content_type: ContentType::Manhwa,
            supports_nsfw: false,
            capabilities: vec![Capability::NetworkFetch, Capability::LogDebug],
            icon_url: Some(format!("{BASE_URL}/images/logo.png")),
            description: Some("Asura Scans — popular manhwa/manga reader".into()),
        })
    }

    fn search(&self, query: SearchQuery) -> Result<SearchResponse, SourceError> {
        let page = query.page;

        let url = if let Some(ref title) = query.title {
            let encoded = urlencod(title);
            format!(
                "{BASE_URL}/series?page={page}&name={encoded}&genres=&status=-1&types=-1&order=rating"
            )
        } else {
            format!(
                "{BASE_URL}/series?page={page}&genres=&status=-1&types=-1&order=update"
            )
        };

        atlas_log!("[asura] search url={url}");

        let html_text = get_text(&url)?;
        let entries = parser::parse_manga_list(&html_text);
        let has_next = parser::has_next_page(&html_text);

        Ok(SearchResponse {
            entries,
            has_next_page: has_next,
            total: None,
        })
    }

    fn get_manga_details(&self, id: &str) -> Result<Manga, SourceError> {
        let url = format!("{BASE_URL}/series/{id}");
        atlas_log!("[asura] get_manga_details url={url}");

        let html_text = get_text(&url)?;
        parser::parse_manga_details(&html_text, id)
    }

    fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>, SourceError> {
        let url = format!("{BASE_URL}/series/{manga_id}");
        atlas_log!("[asura] get_chapters url={url}");

        let html_text = get_text(&url)?;
        Ok(parser::parse_chapter_list(&html_text, manga_id))
    }

    fn get_pages(&self, chapter_id: &str) -> Result<Vec<Page>, SourceError> {
        // chapter_id format: "manga-slug-/42" (manga_id + "/" + chapter_number)
        let (manga_id, chapter_num) = chapter_id
            .rsplit_once('/')
            .ok_or(SourceError::Parse {
                message: format!("invalid chapter_id format: {chapter_id}"),
            })?;

        let url = format!("{BASE_URL}/series/{manga_id}/chapter/{chapter_num}");
        atlas_log!("[asura] get_pages url={url}");

        let html_text = get_text(&url)?;
        Ok(parser::parse_pages(&html_text))
    }

    fn get_latest(&self, page: u32) -> Result<SearchResponse, SourceError> {
        let url = format!(
            "{BASE_URL}/series?page={page}&genres=&status=-1&types=-1&order=update"
        );
        atlas_log!("[asura] get_latest url={url}");

        let html_text = get_text(&url)?;
        let entries = parser::parse_manga_list(&html_text);
        let has_next = parser::has_next_page(&html_text);

        Ok(SearchResponse {
            entries,
            has_next_page: has_next,
            total: None,
        })
    }

    fn get_popular(&self, page: u32) -> Result<SearchResponse, SourceError> {
        let url = format!(
            "{BASE_URL}/series?page={page}&genres=&status=-1&types=-1&order=rating"
        );
        atlas_log!("[asura] get_popular url={url}");

        let html_text = get_text(&url)?;
        let entries = parser::parse_manga_list(&html_text);
        let has_next = parser::has_next_page(&html_text);

        Ok(SearchResponse {
            entries,
            has_next_page: has_next,
            total: None,
        })
    }
}

/// Simple percent-encoding for URL query parameters.
fn urlencod(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => {
                out.push('%');
                out.push(char::from(b"0123456789ABCDEF"[(b >> 4) as usize]));
                out.push(char::from(b"0123456789ABCDEF"[(b & 0x0F) as usize]));
            }
        }
    }
    out
}

export_source!(AsuraScans);
