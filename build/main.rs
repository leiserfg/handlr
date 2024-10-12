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
    println!("cargo:rerun-if-changed=build/");
    let out_dir = Path::new(&env::var("OUT_DIR")?).to_path_buf();
    mangen(out_dir)
}

/// Generate man page for binary and subcommands
fn mangen(out_dir: PathBuf) -> DynResult {
    println!("cargo:rerun-if-env-changed=PROJECT_NAME");
    println!("cargo:rerun-if-env-changed=PROJECT_EXECUTABLE");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

    eprintln!("Generating man pages");

    let dest_dir = out_dir.join("manual/man1");
    let cmd = Cmd::command().name("handlr");

    create_dir_all(&dest_dir)?;

    clap_mangen::generate_to(cmd, &dest_dir)?;

    // Remove hidden subcommand's manpage
    remove_file(dest_dir.join("handlr-autocomplete.1"))?;

    Ok(())
}
