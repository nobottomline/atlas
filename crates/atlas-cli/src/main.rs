//! # atlas
//!
//! Developer CLI for the Atlas source platform.
//!
//! ## Commands
//!
//! ```
//! atlas new source <name>    Scaffold a new source project
//! atlas build                Compile source to WASM
//! atlas validate             Validate manifest (+ optional WASM)
//! atlas inspect <file>       Display manifest or WASM metadata
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(
    name = "atlas",
    about = "Developer CLI for the Atlas source platform",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new source project.
    New(NewKind),

    /// Compile the current source to a WASM package.
    Build(commands::build::BuildArgs),

    /// Validate a source manifest and optionally its WASM module.
    Validate(commands::validate::ValidateArgs),

    /// Display metadata for a manifest.json or .wasm file.
    Inspect(commands::inspect::InspectArgs),
}

#[derive(clap::Args)]
struct NewKind {
    #[command(subcommand)]
    kind: NewSubcommand,
}

#[derive(Subcommand)]
enum NewSubcommand {
    /// Create a new source module project.
    Source(commands::new::NewArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New(kind) => match kind.kind {
            NewSubcommand::Source(args) => commands::new::run(args),
        },
        Commands::Build(args)    => commands::build::run(args),
        Commands::Validate(args) => commands::validate::run(args),
        Commands::Inspect(args)  => commands::inspect::run(args),
    }
}
