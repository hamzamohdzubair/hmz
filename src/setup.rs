use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::manifest::{self, Manifest};
use crate::paths;
use crate::registry::{self, AppDef, Loc};

// Update this to your private dotfiles repo URL.
const REMOTE: &str = "git@github.com:hamzamohdzubair/dotfiles.git";

pub fn run(remote: Option<&str>) -> Result<()> {
    let repo = paths::repo_dir();
    let remote = remote.unwrap_or(REMOTE);

    if repo.exists() {
        anyhow::bail!(
            "{} already exists. Run `hmz backup` to back up, or `hmz clean` to start fresh.",
            repo.display()
        );
    }

    println!("Cloning {}...", remote);
    let status = Command::new("git")
        .args(["clone", remote])
        .arg(&repo)
        .status()
        .context("git not found — is git installed?")?;

    if !status.success() {
        anyhow::bail!("git clone failed");
    }

    let mf = manifest::load()?;

    if mf.crates.is_empty() && mf.apps.is_empty() {
        println!("Repo initialised. Run `hmz add <name>` to start.");
        return Ok(());
    }

    println!(
        "Manifest found — restoring {} crate(s), {} app(s)...",
        mf.crates.len(),
        mf.apps.len()
    );

    for name in &mf.crates {
        println!("  Installing {}...", name);
        cargo_install(name)?;
        restore_crate(name)?;
    }

    for name in &mf.apps {
        let def = registry::lookup(name)?;
        restore_app(&def)?;
    }

    crate::cron::install()?;
    println!("Done.");
    Ok(())
}

pub fn add(name: &str, is_crate: bool) -> Result<()> {
    let repo = paths::repo_dir();
    if !repo.exists() {
        anyhow::bail!("Not initialised. Run `hmz setup` first.");
    }

    let mut mf = manifest::load()?;

    if is_crate {
        add_crate(name, &mut mf)?;
    } else {
        let def = registry::lookup(name)?;
        add_app(&def, &mut mf)?;
    }

    manifest::save(&mf)?;
    Ok(())
}

fn add_crate(name: &str, mf: &mut Manifest) -> Result<()> {
    if mf.crates.contains(&name.to_string()) || mf.apps.contains(&name.to_string()) {
        anyhow::bail!("'{}' is already managed", name);
    }

    println!("Installing {}...", name);
    cargo_install(name)?;

    let src_cfg = paths::xdg_config(name);
    let dst_cfg = paths::repo_crate_config(name);
    if src_cfg.exists() && !src_cfg.is_symlink() {
        move_and_link(&src_cfg, &dst_cfg)
            .with_context(|| format!("linking config for {}", name))?;
        println!("  config: {} -> {}", src_cfg.display(), dst_cfg.display());
    }

    let src_data = paths::xdg_data(name);
    let dst_data = paths::repo_crate_data(name);
    if src_data.exists() && !src_data.is_symlink() {
        move_and_link(&src_data, &dst_data)
            .with_context(|| format!("linking data for {}", name))?;
        println!("  data:   {} -> {}", src_data.display(), dst_data.display());
    }

    mf.crates.push(name.to_string());
    println!("Added crate '{}'.", name);
    Ok(())
}

fn add_app(def: &AppDef, mf: &mut Manifest) -> Result<()> {
    let name = &def.name;
    if mf.apps.contains(name) || mf.crates.contains(name) {
        anyhow::bail!("'{}' is already managed", name);
    }

    for loc in &def.locs {
        match loc {
            Loc::XdgConfig => {
                let src = paths::xdg_config(name);
                let dst = paths::repo_app_loc(name, loc.dir_name());
                if src.exists() && !src.is_symlink() {
                    move_and_link(&src, &dst)
                        .with_context(|| format!("linking xdg_config for {}", name))?;
                    println!("  xdg_config: {} -> {}", src.display(), dst.display());
                }
            }
            Loc::XdgData => {
                let src = paths::xdg_data(name);
                let dst = paths::repo_app_loc(name, loc.dir_name());
                if src.exists() && !src.is_symlink() {
                    move_and_link(&src, &dst)
                        .with_context(|| format!("linking xdg_data for {}", name))?;
                    println!("  xdg_data: {} -> {}", src.display(), dst.display());
                }
            }
            Loc::WinAppData => {
                if paths::is_wsl() {
                    let src = paths::win_appdata(name)?;
                    let dst = paths::repo_app_loc(name, loc.dir_name());
                    if src.exists() {
                        copy_dir(&src, &dst)
                            .with_context(|| format!("copying win_appdata for {}", name))?;
                        println!("  win_appdata: {} -> {}", src.display(), dst.display());
                    }
                }
            }
            Loc::Home(rel) => {
                let src = paths::home().join(rel);
                let dst = paths::repo_app_loc(name, loc.dir_name()).join(rel);
                if src.exists() && !src.is_symlink() {
                    move_and_link(&src, &dst)
                        .with_context(|| format!("linking ~/{} for {}", rel, name))?;
                    println!("  ~/{}: {} -> {}", rel, src.display(), dst.display());
                }
            }
        }
    }

    mf.apps.push(name.to_string());
    println!("Added app '{}'.", name);
    Ok(())
}

fn restore_crate(name: &str) -> Result<()> {
    let dst_cfg = paths::repo_crate_config(name);
    let src_cfg = paths::xdg_config(name);
    if dst_cfg.exists() && !src_cfg.exists() {
        ensure_parent(&src_cfg)?;
        symlink(&dst_cfg, &src_cfg)?;
    }

    let dst_data = paths::repo_crate_data(name);
    let src_data = paths::xdg_data(name);
    if dst_data.exists() && !src_data.exists() {
        ensure_parent(&src_data)?;
        symlink(&dst_data, &src_data)?;
    }

    Ok(())
}

fn restore_app(def: &AppDef) -> Result<()> {
    let name = &def.name;
    for loc in &def.locs {
        match loc {
            Loc::XdgConfig => {
                let repo = paths::repo_app_loc(name, loc.dir_name());
                let sys = paths::xdg_config(name);
                if repo.exists() && !sys.exists() {
                    ensure_parent(&sys)?;
                    symlink(&repo, &sys)?;
                }
            }
            Loc::XdgData => {
                let repo = paths::repo_app_loc(name, loc.dir_name());
                let sys = paths::xdg_data(name);
                if repo.exists() && !sys.exists() {
                    ensure_parent(&sys)?;
                    symlink(&repo, &sys)?;
                }
            }
            Loc::WinAppData => {
                if paths::is_wsl() {
                    let repo = paths::repo_app_loc(name, loc.dir_name());
                    let sys = paths::win_appdata(name)?;
                    if repo.exists() && !sys.exists() {
                        copy_dir(&repo, &sys)
                            .with_context(|| format!("restoring win_appdata for {}", name))?;
                    }
                }
            }
            Loc::Home(rel) => {
                let repo = paths::repo_app_loc(name, loc.dir_name()).join(rel);
                let sys = paths::home().join(rel);
                if repo.exists() && !sys.exists() {
                    ensure_parent(&sys)?;
                    symlink(&repo, &sys)?;
                }
            }
        }
    }
    Ok(())
}

fn cargo_install(name: &str) -> Result<()> {
    Command::new("cargo")
        .args(["install", name])
        .status()
        .context("cargo not found")?;
    Ok(())
}

fn move_and_link(src: &Path, dst: &Path) -> Result<()> {
    ensure_parent(dst)?;
    fs::rename(src, dst)
        .with_context(|| format!("moving {} to {}", src.display(), dst.display()))?;
    symlink(dst, src)
}

fn symlink(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link)
        .with_context(|| format!("symlink {} -> {}", link.display(), target.display()))
}

fn ensure_parent(p: &Path) -> Result<()> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), &dst_path)
                .with_context(|| format!("copying {}", entry.path().display()))?;
        }
    }
    Ok(())
}
