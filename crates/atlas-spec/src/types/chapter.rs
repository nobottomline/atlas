use serde::{Deserialize, Serialize};

/// A single chapter entry returned by `get_chapters`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    /// Source-local chapter identifier (used as key for `get_pages`).
    pub id: String,
    /// Parent manga identifier.
    pub manga_id: String,
    pub title: Option<String>,
    /// Chapter number, e.g. 42.5.
    pub number: Option<f64>,
    /// Volume number, e.g. 3.0.
    pub volume: Option<f64>,
    /// BCP-47 language tag.
    pub lang: String,
    /// Unix timestamp (seconds) of when the chapter was published or last updated.
    pub date_updated: Option<i64>,
    pub scanlator: Option<String>,
    /// Canonical URL on the source site.
    pub url: String,
}
