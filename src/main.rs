mod backup;
mod clean;
mod cron;
mod manifest;
mod paths;
mod setup;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hmz", about = "Personal app and dotfile backup manager")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Clone remote repo, install apps, create symlinks, register hourly backup
    Setup {
        /// Git remote URL of your private backup repo
        remote: String,
    },
    /// Add an app (by crate name) or dotfile (by path) to be managed
    Add {
        /// Crate name (e.g. zen) or path (e.g. ~/.config/nvim)
        target: String,
    },
    /// Commit and push all changes to remote
    Backup,
    /// Show what has changed since last backup
    Status,
    /// Remove all symlinks, uninstall apps, remove cron job, delete local repo
    Clean,
    /// List all managed apps and dotfiles
    Ls,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Setup { remote } => setup::run(&remote),
        Command::Add { target } => setup::add(&target),
        Command::Backup => backup::run(),
        Command::Status => backup::status(),
        Command::Clean => clean::run(),
        Command::Ls => ls(),
    }
}

fn ls() -> anyhow::Result<()> {
    let mf = manifest::load()?;
    if mf.crates.is_empty() && mf.dotfiles.is_empty() {
        println!("Nothing managed yet. Run `hmz add <app>` to start.");
        return Ok(());
    }
    if !mf.crates.is_empty() {
        println!("Apps:");
        for c in &mf.crates {
            println!("  {}", c);
        }
    }
    if !mf.dotfiles.is_empty() {
        println!("Dotfiles:");
        for d in &mf.dotfiles {
            println!("  {}", d);
        }
    }
    Ok(())
}
