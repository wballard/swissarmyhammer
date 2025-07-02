use crate::cli::Cli;
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use std::io;
use std::path::Path;

/// Generate shell completion scripts
#[allow(dead_code)]
pub fn generate_completions<P: AsRef<Path>>(outdir: P) -> Result<()> {
    let outdir = outdir.as_ref();

    let mut cmd = Cli::command();

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        generate_to(shell, &mut cmd, "swissarmyhammer", outdir)?;
    }

    println!("Generated shell completions in: {}", outdir.display());

    Ok(())
}

/// Print shell completion script to stdout
pub fn print_completion(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();

    clap_complete::generate(shell, &mut cmd, "swissarmyhammer", &mut io::stdout());

    Ok(())
}
