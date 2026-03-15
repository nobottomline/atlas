//! `atlas build` — compile a source module to WASM and generate a manifest.

use std::path::PathBuf;
use std::process::Command;
use anyhow::{bail, Context, Result};
use clap::Args;
use atlas_runtime::integrity::compute_sha256_hex;

#[derive(Args)]
pub struct BuildArgs {
    /// Build in release mode (smaller, optimized WASM).
    #[arg(long, default_value = "true")]
    pub release: bool,

    /// Target directory for the output package (default: ./dist).
    #[arg(long, default_value = "dist")]
    pub out_dir: PathBuf,
}

pub fn run(args: BuildArgs) -> Result<()> {
    println!("Building source module...");

    // 1. Run cargo build for wasm32-unknown-unknown.
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--target", "wasm32-unknown-unknown"]);
    if args.release {
        cmd.arg("--release");
    }

    let status = cmd.status().context("failed to run cargo build")?;
    if !status.success() {
        bail!("cargo build failed");
    }

    // 2. Find the output .wasm file.
    let profile = if args.release { "release" } else { "debug" };
    let target_dir = PathBuf::from("target/wasm32-unknown-unknown").join(profile);

    let wasm_files: Vec<_> = std::fs::read_dir(&target_dir)
        .context("cannot read target directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("wasm"))
        .collect();

    if wasm_files.is_empty() {
        bail!("no .wasm files found in {}", target_dir.display());
    }
    if wasm_files.len() > 1 {
        println!("Multiple WASM files found, using the first one.");
    }

    let wasm_path = wasm_files[0].path();
    let wasm_bytes = std::fs::read(&wasm_path).context("cannot read WASM file")?;
    let hash = compute_sha256_hex(&wasm_bytes);

    // 3. Copy to output directory.
    std::fs::create_dir_all(&args.out_dir).context("cannot create output directory")?;
    let out_wasm = args.out_dir.join(wasm_path.file_name().unwrap());
    std::fs::copy(&wasm_path, &out_wasm).context("cannot copy WASM file")?;

    println!("  Built:   {}", out_wasm.display());
    println!("  SHA-256: {hash}");
    println!("  Size:    {:.1} KB", wasm_bytes.len() as f64 / 1024.0);

    // 4. Update SHA-256 in manifest.json if it exists.
    let manifest_path = PathBuf::from("manifest.json");
    if manifest_path.exists() {
        let text = std::fs::read_to_string(&manifest_path)?;
        let mut manifest: serde_json::Value = serde_json::from_str(&text)?;
        manifest["module_sha256"] = serde_json::Value::String(hash.clone());
        let updated = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(&manifest_path, updated)?;
        println!("  Updated manifest.json with new SHA-256");
    }

    println!("\nBuild complete.");
    Ok(())
}
