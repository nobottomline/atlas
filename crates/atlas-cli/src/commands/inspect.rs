//! `atlas inspect` — display manifest details and WASM metadata for a package.

use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::Args;
use atlas_spec::manifest::SourceManifest;
use atlas_runtime::integrity::compute_sha256_hex;

#[derive(Args)]
pub struct InspectArgs {
    /// Path to a manifest.json or a .wasm file.
    pub target: PathBuf,
}

pub fn run(args: InspectArgs) -> Result<()> {
    let ext = args.target.extension().and_then(|s| s.to_str()).unwrap_or("");

    match ext {
        "json" => inspect_manifest(&args.target),
        "wasm" => inspect_wasm(&args.target),
        _ => {
            // Try manifest first, then wasm.
            if args.target.exists() {
                let bytes = std::fs::read(&args.target)?;
                if bytes.starts_with(b"\0asm") {
                    inspect_wasm(&args.target)
                } else {
                    inspect_manifest(&args.target)
                }
            } else {
                anyhow::bail!("file not found: {}", args.target.display())
            }
        }
    }
}

fn inspect_manifest(path: &PathBuf) -> Result<()> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read {}", path.display()))?;
    let manifest: SourceManifest = serde_json::from_str(&text)
        .with_context(|| "not a valid manifest JSON")?;

    println!("── Source Manifest ──────────────────────────────");
    println!("  ID:              {}", manifest.id);
    println!("  Name:            {}", manifest.name);
    println!("  Version:         {}", manifest.version);
    println!("  Language:        {}", manifest.lang);
    println!("  Content type:    {:?}", manifest.content_type);
    println!("  NSFW:            {}", manifest.supports_nsfw);
    println!("  Module:          {}", manifest.module_filename);
    println!("  SHA-256:         {}", manifest.module_sha256);
    println!("  Min runtime:     {}", manifest.min_runtime_version);

    if let Some(author) = &manifest.author {
        println!("  Author:          {author}");
    }
    if let Some(license) = &manifest.license {
        println!("  License:         {license}");
    }
    if !manifest.tags.is_empty() {
        println!("  Tags:            {}", manifest.tags.join(", "));
    }

    println!("  Base URLs:");
    for url in &manifest.base_urls {
        println!("    - {url}");
    }

    println!("  Capabilities:");
    if manifest.capabilities.is_empty() {
        println!("    (none)");
    }
    for cap in &manifest.capabilities {
        println!("    - {}", cap.identifier());
    }

    if manifest.deprecated {
        println!("  ⚠ DEPRECATED");
        if let Some(replacement) = &manifest.replaced_by {
            println!("    Replaced by: {replacement}");
        }
    }

    if manifest.signature.is_some() {
        println!("  ✓ Signed");
    } else {
        println!("  ⚠ Unsigned (not accepted by registry)");
    }

    Ok(())
}

fn inspect_wasm(path: &PathBuf) -> Result<()> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("cannot read {}", path.display()))?;

    if !bytes.starts_with(b"\0asm") {
        anyhow::bail!("not a valid WASM binary (bad magic bytes)");
    }

    let hash = compute_sha256_hex(&bytes);
    let size_kb = bytes.len() as f64 / 1024.0;

    println!("── WASM Module ──────────────────────────────────");
    println!("  File:    {}", path.display());
    println!("  Size:    {size_kb:.1} KB ({} bytes)", bytes.len());
    println!("  SHA-256: {hash}");

    Ok(())
}
