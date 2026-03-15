//! End-to-end test: load the real Asura Scans WASM and call the live site.
//!
//! Requires:
//!   cargo build --target wasm32-unknown-unknown --release -p asura-scans

use std::sync::Arc;
use atlas_runtime::{AtlasEngine, SourceInstance};
use atlas_runtime::network::UreqNetworkProvider;
use atlas_spec::manifest::SourceManifest;
use atlas_spec::types::search::SearchQuery;

fn wasm_bytes() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/wasm32-unknown-unknown/release/asura_scans.wasm"
    );
    std::fs::read(path).expect(
        "asura-scans WASM not found — run: cargo build --target wasm32-unknown-unknown --release -p asura-scans"
    )
}

fn manifest() -> SourceManifest {
    serde_json::from_str(r#"{
        "id": "asura-scans",
        "name": "Asura Scans",
        "version": "0.1.0",
        "lang": "en",
        "base_urls": ["https://asuracomic.net"],
        "content_type": "manhwa",
        "supports_nsfw": false,
        "module_filename": "asura_scans.wasm",
        "module_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        "min_runtime_version": "0.1.0",
        "capabilities": ["network_fetch", "log_debug"],
        "allowed_domains": ["asuracomic.net", "gg.asuracomic.net"],
        "author": "Atlas Contributors",
        "license": "MIT OR Apache-2.0",
        "tags": ["manhwa", "popular"],
        "deprecated": false,
        "description": "Asura Scans — popular manhwa/manga reader"
    }"#).expect("bad manifest")
}

fn load_source() -> SourceInstance {
    let engine = AtlasEngine::new();
    let network = Arc::new(UreqNetworkProvider::new());
    SourceInstance::load(&engine, &wasm_bytes(), manifest(), network)
        .expect("failed to load asura-scans WASM")
}

#[test]
fn get_info() {
    let mut source = load_source();
    let info = source.get_info().expect("get_info failed");
    assert_eq!(info.id, "asura-scans");
    assert_eq!(info.name, "Asura Scans");
    assert_eq!(info.lang, "en");
    println!("Source: {} v{}", info.name, info.version);
}

#[test]
fn search_popular() {
    let mut source = load_source();
    let response = source.get_popular(1).expect("get_popular failed");

    println!("Popular results: {} entries", response.entries.len());
    assert!(!response.entries.is_empty(), "should have popular manga");

    for entry in response.entries.iter().take(5) {
        println!("  - {} (id: {})", entry.title, entry.id);
    }
}

#[test]
fn search_by_title() {
    let mut source = load_source();
    let query = SearchQuery {
        title: Some("solo".into()),
        filters: vec![],
        page: 1,
    };
    let response = source.search(&query).expect("search failed");

    println!("Search 'solo': {} results", response.entries.len());
    assert!(!response.entries.is_empty(), "search for 'solo' should return results");

    for entry in &response.entries {
        println!("  - {} (id: {})", entry.title, entry.id);
    }
}

#[test]
fn manga_details() {
    let mut source = load_source();

    // First get a manga ID from the popular list
    let popular = source.get_popular(1).expect("get_popular failed");
    let first = &popular.entries[0];
    println!("Getting details for: {} (id: {})", first.title, first.id);

    let manga = source.get_manga_details(&first.id).expect("get_manga_details failed");

    println!("Title: {}", manga.title);
    println!("Author: {:?}", manga.author);
    println!("Status: {:?}", manga.status);
    println!("Tags: {:?}", manga.tags);
    println!("Description: {:.100}...", manga.description.as_deref().unwrap_or("none"));

    assert!(!manga.title.is_empty());
}

#[test]
fn chapters_and_pages() {
    let mut source = load_source();

    // Get a manga
    let popular = source.get_popular(1).expect("get_popular failed");
    let first = &popular.entries[0];

    // Get chapters
    let chapters = source.get_chapters(&first.id).expect("get_chapters failed");
    println!("{}: {} chapters", first.title, chapters.len());
    assert!(!chapters.is_empty(), "manga should have chapters");

    // Show first few
    for ch in chapters.iter().take(3) {
        println!("  Ch.{:?} - {:?} (id: {})", ch.number, ch.title, ch.id);
    }

    // Get pages of the latest chapter
    let latest = &chapters[0];
    let pages = source.get_pages(&latest.id).expect("get_pages failed");
    println!("Chapter '{}' has {} pages", latest.id, pages.len());
    assert!(!pages.is_empty(), "chapter should have pages");

    for page in pages.iter().take(3) {
        println!("  Page {}: {:?}", page.index, page.data);
    }
}

#[test]
fn drain_logs() {
    let mut source = load_source();
    let _ = source.get_popular(1);
    let logs = source.drain_logs();
    println!("Logs captured: {}", logs.len());
    for log in &logs {
        println!("  {log}");
    }
    assert!(!logs.is_empty(), "should have log entries from the source");
}
