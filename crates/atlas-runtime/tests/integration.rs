//! End-to-end integration test: compile example-source → load WASM → invoke all contract methods.
//!
//! Requires `example-source` to have been built first:
//!   cargo build --target wasm32-unknown-unknown -p example-source

use std::sync::Arc;
use atlas_runtime::{AtlasEngine, SourceInstance};
use atlas_runtime::network::UreqNetworkProvider;
use atlas_spec::manifest::SourceManifest;
use atlas_spec::types::search::SearchQuery;

fn wasm_bytes() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/wasm32-unknown-unknown/debug/example_source.wasm"
    );
    std::fs::read(path).expect("example-source WASM not found — run: cargo build --target wasm32-unknown-unknown -p example-source")
}

fn manifest() -> SourceManifest {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../registry/sources/example-source/manifest.json"
    );
    let json = std::fs::read_to_string(path).expect("manifest.json not found");
    serde_json::from_str(&json).expect("invalid manifest.json")
}

fn load_source() -> SourceInstance {
    let engine = AtlasEngine::new();
    let network = Arc::new(UreqNetworkProvider::new());
    SourceInstance::load(&engine, &wasm_bytes(), manifest(), network)
        .expect("failed to load example-source WASM")
}

#[test]
fn get_info() {
    let mut source = load_source();
    let info = source.get_info().expect("get_info failed");

    assert_eq!(info.id, "example-source");
    assert_eq!(info.name, "Example Source");
    assert_eq!(info.version, "0.1.0");
    assert_eq!(info.lang, "en");
    assert!(!info.supports_nsfw);
    assert!(!info.base_urls.is_empty());
}

#[test]
fn search() {
    let mut source = load_source();
    let query = SearchQuery {
        title: Some("test".into()),
        filters: vec![],
        page: 1,
    };
    let response = source.search(&query).expect("search failed");

    assert!(!response.entries.is_empty());
    assert_eq!(response.entries[0].id, "manga-1");
    assert!(response.entries[0].title.contains("test"));
    assert!(!response.has_next_page);
}

#[test]
fn get_manga_details() {
    let mut source = load_source();
    let manga = source.get_manga_details("manga-1").expect("get_manga_details failed");

    assert_eq!(manga.id, "manga-1");
    assert_eq!(manga.title, "Example Manga");
    assert!(manga.author.is_some());
    assert!(manga.description.is_some());
    assert!(!manga.tags.is_empty());
}

#[test]
fn get_manga_details_not_found() {
    let mut source = load_source();
    let result = source.get_manga_details("nonexistent");
    assert!(result.is_err(), "should return error for unknown id");
}

#[test]
fn get_chapters() {
    let mut source = load_source();
    let chapters = source.get_chapters("manga-1").expect("get_chapters failed");

    assert_eq!(chapters.len(), 2);
    assert_eq!(chapters[0].id, "chapter-2");
    assert_eq!(chapters[1].id, "chapter-1");
    assert!(chapters[0].number.unwrap() > chapters[1].number.unwrap());
}

#[test]
fn get_pages() {
    let mut source = load_source();
    let pages = source.get_pages("chapter-1").expect("get_pages failed");

    assert_eq!(pages.len(), 5);
    assert_eq!(pages[0].index, 0);
    assert_eq!(pages[4].index, 4);
}

#[test]
fn get_latest() {
    let mut source = load_source();
    let response = source.get_latest(1).expect("get_latest failed");
    assert!(!response.entries.is_empty());
}

#[test]
fn get_popular() {
    let mut source = load_source();
    let response = source.get_popular(1).expect("get_popular failed");
    assert!(!response.entries.is_empty());
}

#[test]
fn get_filters() {
    let mut source = load_source();
    let filters = source.get_filters().expect("get_filters failed");
    // Example source returns empty filters
    assert!(filters.is_empty());
}

#[test]
fn get_preferences_schema() {
    let mut source = load_source();
    let schema = source.get_preferences_schema().expect("get_preferences_schema failed");
    // Example source returns empty schema
    assert!(schema.fields.is_empty());
}

#[test]
fn drain_logs() {
    let mut source = load_source();
    // Trigger a call that logs
    let _ = source.search(&SearchQuery { title: Some("log-test".into()), filters: vec![], page: 1 });
    let logs = source.drain_logs();
    assert!(!logs.is_empty(), "should have captured log messages");
    assert!(logs.iter().any(|l| l.contains("log-test")));
}

#[test]
fn integrity_check() {
    let wasm = wasm_bytes();
    let hash = atlas_runtime::compute_sha256_hex(&wasm);
    assert!(!hash.is_empty());
    assert!(atlas_runtime::verify_sha256(&wasm, &hash).is_ok());
    assert!(atlas_runtime::verify_sha256(&wasm, "0000000000000000000000000000000000000000000000000000000000000000").is_err());
}
