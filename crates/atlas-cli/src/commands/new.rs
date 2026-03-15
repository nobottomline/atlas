//! `atlas new source <name>` — scaffold a new source project.

use std::path::PathBuf;
use anyhow::{bail, Context, Result};
use clap::Args;

#[derive(Args)]
pub struct NewArgs {
    /// Source identifier (lowercase, hyphen-separated). e.g. "manga-plus"
    pub name: String,

    /// Directory to create the project in (default: ./<name>).
    #[arg(long)]
    pub out: Option<PathBuf>,
}

pub fn run(args: NewArgs) -> Result<()> {
    let id = args.name.to_lowercase().replace('_', "-");
    if id.is_empty() {
        bail!("source name must not be empty");
    }

    let dir = args.out.unwrap_or_else(|| PathBuf::from(&id));
    if dir.exists() {
        bail!("directory {} already exists", dir.display());
    }

    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir).context("failed to create source directory")?;

    // ── Cargo.toml ────────────────────────────────────────────────────────────

    write_file(
        &dir.join("Cargo.toml"),
        &cargo_toml(&id),
    )?;

    // ── src/lib.rs ────────────────────────────────────────────────────────────

    write_file(
        &src_dir.join("lib.rs"),
        &lib_rs(&id),
    )?;

    // ── manifest.json ─────────────────────────────────────────────────────────

    write_file(
        &dir.join("manifest.json"),
        &manifest_json(&id),
    )?;

    println!("Created new source: {id}");
    println!("  {}", dir.display());
    println!("\nNext steps:");
    println!("  cd {id}");
    println!("  # Implement the Source trait in src/lib.rs");
    println!("  atlas build");
    println!("  atlas validate --wasm dist/{id}.wasm");

    Ok(())
}

fn write_file(path: &PathBuf, content: &str) -> Result<()> {
    std::fs::write(path, content)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn cargo_toml(id: &str) -> String {
    format!(
        r#"[package]
name    = "{id}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
atlas-sdk  = {{ version = "0.1" }}
atlas-spec = {{ version = "0.1" }}
serde      = {{ version = "1", features = ["derive"] }}

[profile.release]
opt-level     = "z"
lto           = true
strip         = true
codegen-units = 1
"#
    )
}

fn lib_rs(id: &str) -> String {
    let struct_name: String = id
        .split('-')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect();

    format!(
        r#"use atlas_sdk::prelude::*;

pub struct {struct_name};

impl Default for {struct_name} {{
    fn default() -> Self {{ Self }}
}}

impl Source for {struct_name} {{
    fn get_info(&self) -> Result<SourceInfo, SourceError> {{
        Ok(SourceInfo {{
            id: "{id}".into(),
            name: "{struct_name}".into(),
            version: "0.1.0".into(),
            lang: "en".into(),
            base_urls: vec!["https://example.com".into()],
            content_type: ContentType::Manga,
            supports_nsfw: false,
            capabilities: vec![Capability::NetworkFetch, Capability::LogDebug],
            icon_url: None,
            description: Some("{struct_name} source for Atlas".into()),
        }})
    }}

    fn search(&self, query: SearchQuery) -> Result<SearchResponse, SourceError> {{
        atlas_log!("search: {{:?}}", query.title);
        // TODO: implement search
        Ok(SearchResponse {{
            entries: vec![],
            has_next_page: false,
            total: Some(0),
        }})
    }}

    fn get_manga_details(&self, id: &str) -> Result<Manga, SourceError> {{
        // TODO: fetch and parse manga details
        Err(SourceError::NotFound)
    }}

    fn get_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>, SourceError> {{
        // TODO: fetch and parse chapter list
        Ok(vec![])
    }}

    fn get_pages(&self, chapter_id: &str) -> Result<Vec<Page>, SourceError> {{
        // TODO: resolve page URLs
        Ok(vec![])
    }}
}}

// Generates all required WASM exports.
export_source!({struct_name});
"#
    )
}

fn manifest_json(id: &str) -> String {
    format!(
        r#"{{
  "id": "{id}",
  "name": "{id}",
  "version": "0.1.0",
  "lang": "en",
  "base_urls": ["https://example.com"],
  "content_type": "manga",
  "supports_nsfw": false,
  "module_filename": "{id}.wasm",
  "module_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
  "min_runtime_version": "0.1.0",
  "capabilities": ["network_fetch", "log_debug"],
  "allowed_domains": ["example.com"],
  "tags": [],
  "deprecated": false
}}
"#
    )
}
