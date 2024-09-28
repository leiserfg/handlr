mod cli {
    include!("../src/cli.rs");
}
mod common; // Trick the cli module into cooperating

use cli::Cmd;

use clap::CommandFactory;
use std::{
    env,
    error::Error,
    fs::{create_dir_all, remove_file},
    path::{Path, PathBuf},
};

type DynResult = Result<(), Box<dyn Error>>;

fn main() -> DynResult {
    mangen()
}

/// Generate man page for binary and subcommands
fn mangen() -> DynResult {
    eprintln!("Generating man pages");

    let out_dir = release_dir().join("manual/man1");
    let cmd = Cmd::command().name("handlr");

    create_dir_all(&out_dir)?;

    clap_mangen::generate_to(cmd, &out_dir)?;

    // Remove hidden subcommand's manpage
    remove_file(out_dir.join("handlr-autocomplete.1"))?;

    Ok(())
}

// Project root
fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Output directory for generated assets
fn release_dir() -> PathBuf {
    project_root().join("release")
}
