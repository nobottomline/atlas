use serde::{Deserialize, Serialize};
use super::manga::MangaEntry;

/// Input to `search`. Page is 1-indexed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub title: Option<String>,
    pub filters: Vec<AppliedFilter>,
    /// 1-indexed page number.
    pub page: u32,
}

impl SearchQuery {
    pub fn title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            filters: Vec::new(),
            page: 1,
        }
    }
}

/// Response from `search`, `get_latest`, and `get_popular`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub entries: Vec<MangaEntry>,
    pub has_next_page: bool,
    /// Total result count, if known by the source.
    pub total: Option<u32>,
}

/// A filter definition that a source can advertise via `get_filters`.
/// The host app renders filter UI from this schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub id: String,
    pub label: String,
    pub kind: FilterKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "options", rename_all = "snake_case")]
pub enum FilterKind {
    Select(Vec<FilterOption>),
    MultiSelect(Vec<FilterOption>),
    Toggle,
    Sort(Vec<FilterOption>),
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOption {
    pub id: String,
    pub label: String,
}

/// A filter value chosen by the user, included in a `SearchQuery`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedFilter {
    pub filter_id: String,
    pub value: FilterValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    Str(String),
    StrList(Vec<String>),
    Bool(bool),
}
