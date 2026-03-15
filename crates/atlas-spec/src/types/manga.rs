use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MangaStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContentRating {
    #[default]
    Safe,
    Suggestive,
    Nsfw,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    #[default]
    Manga,
    Manhwa,
    Manhua,
    Comic,
    Novel,
    Webtoon,
    Unknown,
}

/// Full manga metadata, returned by `get_manga_details`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manga {
    /// Source-local identifier (used as key for chapters/pages).
    pub id: String,
    pub title: String,
    /// Canonical URL on the source site.
    pub url: String,
    pub cover_url: Option<String>,
    pub author: Option<String>,
    pub artist: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub status: MangaStatus,
    pub content_rating: ContentRating,
    pub content_type: ContentType,
    /// BCP-47 language tag, e.g. "en", "ja".
    pub lang: String,
    pub alt_titles: Vec<String>,
}

impl Default for Manga {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            url: String::new(),
            cover_url: None,
            author: None,
            artist: None,
            description: None,
            tags: Vec::new(),
            status: MangaStatus::Unknown,
            content_rating: ContentRating::Safe,
            content_type: ContentType::Manga,
            lang: String::new(),
            alt_titles: Vec::new(),
        }
    }
}

/// Lightweight manga entry used in search and listing responses.
/// Does not include chapters, description, or full metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaEntry {
    pub id: String,
    pub title: String,
    pub url: String,
    pub cover_url: Option<String>,
    pub content_rating: ContentRating,
}
