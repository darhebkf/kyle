use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use std::io;

use super::Cli;

pub fn run(shell: &str) -> Result<()> {
    let shell = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        _ => anyhow::bail!("Unsupported shell: {shell}. Use bash, zsh, or fish."),
    };

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "kyle", &mut io::stdout());
    Ok(())
}
