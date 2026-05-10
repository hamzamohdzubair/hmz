use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::{manifest, paths, registry};
use crate::registry::Loc;

pub fn run() -> Result<()> {
    let repo = paths::repo_dir();
    if !repo.exists() {
        println!("Nothing to clean.");
        return Ok(());
    }

    let mf = manifest::load().unwrap_or_default();

    for name in &mf.crates {
        remove_symlink(&paths::xdg_config(name));
        remove_symlink(&paths::xdg_data(name));
        cargo_uninstall(name);
    }

    for name in &mf.apps {
        if let Ok(def) = registry::lookup(name) {
            for loc in &def.locs {
                match loc {
                    Loc::XdgConfig => remove_symlink(&paths::xdg_config(name)),
                    Loc::XdgData => remove_symlink(&paths::xdg_data(name)),
                    Loc::WinAppData => {} // leave Windows files intact
                    Loc::Home(rel) => remove_symlink(&paths::home().join(rel)),
                }
            }
        }
    }

    crate::cron::remove()?;

    fs::remove_dir_all(&repo)
        .with_context(|| format!("removing {}", repo.display()))?;

    println!("Done. All symlinks removed, crates uninstalled, local repo deleted.");
    println!("Your data remains safely in the remote repo.");
    Ok(())
}

fn remove_symlink(path: &Path) {
    if path.is_symlink() {
        let _ = fs::remove_file(path);
    }
}

fn cargo_uninstall(name: &str) {
    match Command::new("cargo").args(["uninstall", name]).status() {
        Ok(s) if !s.success() => eprintln!("warning: cargo uninstall {} failed", name),
        Err(e) => eprintln!("warning: could not run cargo uninstall {}: {}", name, e),
        _ => {}
    }
}
