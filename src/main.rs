mod backup;
mod clean;
mod cron;
mod manifest;
mod paths;
mod registry;
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
        /// Git remote URL (uses built-in default if omitted)
        remote: Option<String>,
    },
    /// Add an app or crate to be managed
    Add {
        /// App or crate name (e.g. tmux, nvim, zellij)
        name: String,
        /// Install via cargo and manage as a crate
        #[arg(long)]
        cargo: bool,
    },
    /// Commit and push all changes to remote
    Backup,
    /// Show what has changed since last backup
    Status,
    /// Remove all symlinks, uninstall crates, remove cron job, delete local repo
    Clean,
    /// List all managed crates and apps
    Ls,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Setup { remote } => setup::run(remote.as_deref()),
        Command::Add { name, cargo } => setup::add(&name, cargo),
        Command::Backup => backup::run(),
        Command::Status => backup::status(),
        Command::Clean => clean::run(),
        Command::Ls => ls(),
    }
}

fn ls() -> anyhow::Result<()> {
    let mf = manifest::load()?;
    if mf.crates.is_empty() && mf.apps.is_empty() {
        println!("Nothing managed yet. Run `hmz add <name>` to start.");
        return Ok(());
    }
    if !mf.crates.is_empty() {
        println!("Crates:");
        for c in &mf.crates {
            println!("  {}", c);
        }
    }
    if !mf.apps.is_empty() {
        println!("Apps:");
        for a in &mf.apps {
            println!("  {}", a);
        }
    }
    Ok(())
}
