use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::manifest;
use crate::paths;

pub fn run() -> Result<()> {
    let repo = paths::repo_dir();
    if !repo.exists() {
        println!("Nothing to clean.");
        return Ok(());
    }

    let mf = manifest::load().unwrap_or_default();

    for name in &mf.crates {
        remove_symlink(&paths::xdg_data(name));
        remove_symlink(&paths::xdg_config(name));
        cargo_uninstall(name);
    }

    for dotfile in &mf.dotfiles {
        let src = expand(dotfile);
        remove_symlink(&src);
    }

    // Windows dotfiles are plain copies — nothing to remove on the Windows side.

    crate::cron::remove()?;

    fs::remove_dir_all(&repo)
        .with_context(|| format!("removing {}", repo.display()))?;

    println!("Done. All symlinks removed, apps uninstalled, local repo deleted.");
    println!("Your data remains safely in the remote GitHub repo.");
    Ok(())
}

fn remove_symlink(path: &Path) {
    if path.is_symlink() {
        let _ = fs::remove_file(path);
    }
}

fn cargo_uninstall(name: &str) {
    let _ = Command::new("cargo").args(["uninstall", name]).status();
}

fn expand(input: &str) -> PathBuf {
    if let Some(rest) = input.strip_prefix("~/") {
        paths::home().join(rest)
    } else {
        PathBuf::from(input)
    }
}
