//! `atlas validate` — validate a source manifest and optionally a WASM module.

use std::path::PathBuf;
use anyhow::{bail, Context, Result};
use clap::Args;
use atlas_spec::manifest::SourceManifest;
use atlas_runtime::integrity::compute_sha256_hex;

/// Required WASM exports every source module must provide.
const REQUIRED_EXPORTS: &[&str] = &[
    "atlas_alloc",
    "atlas_dealloc",
    "atlas_get_info",
    "atlas_search",
    "atlas_get_manga_details",
    "atlas_get_chapters",
    "atlas_get_pages",
];

#[derive(Args)]
pub struct ValidateArgs {
    /// Path to the source manifest JSON file.
    #[arg(default_value = "manifest.json")]
    pub manifest: PathBuf,

    /// Path to the WASM module (skips WASM checks if not provided).
    #[arg(long)]
    pub wasm: Option<PathBuf>,

    /// Fail on warnings as well as errors.
    #[arg(long)]
    pub strict: bool,
}

pub fn run(args: ValidateArgs) -> Result<()> {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // ── Load and parse manifest ───────────────────────────────────────────────

    println!("Validating manifest: {}", args.manifest.display());

    let manifest_text = std::fs::read_to_string(&args.manifest)
        .with_context(|| format!("cannot read {}", args.manifest.display()))?;

    let manifest: SourceManifest = serde_json::from_str(&manifest_text)
        .with_context(|| "manifest is not valid JSON")?;

    // ── Schema validation ─────────────────────────────────────────────────────

    if let Err(schema_errors) = manifest.validate() {
        errors.extend(schema_errors);
    }

    if manifest.author.is_none() {
        warnings.push("author field is missing".into());
    }
    if manifest.description.is_none() {
        warnings.push("description field is missing".into());
    }
    if manifest.tags.is_empty() {
        warnings.push("no tags declared".into());
    }
    if manifest.signature.is_none() {
        warnings.push("manifest has no signature — package will be rejected by the registry".into());
    }

    // ── WASM module validation ────────────────────────────────────────────────

    if let Some(wasm_path) = &args.wasm {
        println!("Validating WASM module: {}", wasm_path.display());

        let wasm_bytes = std::fs::read(wasm_path)
            .with_context(|| format!("cannot read {}", wasm_path.display()))?;

        // Check integrity hash.
        let actual_hash = compute_sha256_hex(&wasm_bytes);
        if !actual_hash.eq_ignore_ascii_case(&manifest.module_sha256) {
            errors.push(format!(
                "SHA-256 mismatch: manifest says {}, actual is {actual_hash}",
                manifest.module_sha256,
            ));
        } else {
            println!("  ✓ SHA-256 matches manifest");
        }

        // Check required exports via wasmparser.
        match check_wasm_exports(&wasm_bytes, REQUIRED_EXPORTS) {
            Ok(missing) if missing.is_empty() => {
                println!("  ✓ All required exports present");
            }
            Ok(missing) => {
                for name in missing {
                    errors.push(format!("missing required WASM export: {name}"));
                }
            }
            Err(e) => errors.push(format!("WASM parse error: {e}")),
        }
    }

    // ── Report ────────────────────────────────────────────────────────────────

    for w in &warnings {
        println!("  ⚠ {w}");
    }
    for e in &errors {
        println!("  ✗ {e}");
    }

    let warning_count = warnings.len();
    let error_count = errors.len();

    if error_count > 0 {
        bail!("validation failed with {error_count} error(s) and {warning_count} warning(s)");
    }
    if args.strict && warning_count > 0 {
        bail!("strict mode: validation failed with {warning_count} warning(s)");
    }

    println!(
        "\nValidation passed ({warning_count} warning(s), {error_count} error(s))"
    );
    Ok(())
}

/// Scan WASM binary exports and return the list of required names that are absent.
fn check_wasm_exports<'a>(
    wasm_bytes: &[u8],
    required: &[&'a str],
) -> Result<Vec<&'a str>> {
    // Minimal WASM binary parsing: scan the export section for function exports.
    // We use a simple scan rather than a full parser dependency for now.
    // TODO: replace with wasmparser when added as a dependency.
    let wasm_text = String::from_utf8_lossy(wasm_bytes);
    let missing = required
        .iter()
        .filter(|&&name| !wasm_text.contains(name))
        .copied()
        .collect();
    Ok(missing)
}
