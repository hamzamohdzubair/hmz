use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::manifest::{self, Manifest};
use crate::paths;

pub fn run(remote: &str) -> Result<()> {
    let repo = paths::repo_dir();

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

    if mf.crates.is_empty() && mf.dotfiles.is_empty() {
        println!("Repo initialised. Run `hmz add <app>` to start adding apps, then `hmz backup`.");
        return Ok(());
    }

    // Restore mode: repo already has data, recreate symlinks and install apps.
    println!("Manifest found — restoring {} app(s)...", mf.crates.len());

    for name in &mf.crates {
        println!("  Installing {}...", name);
        cargo_install(name)?;
        restore_app_symlinks(name)?;
    }

    for dotfile in &mf.dotfiles {
        restore_dotfile_symlink(dotfile)?;
    }

    crate::cron::install()?;

    println!("Done.");
    Ok(())
}

pub fn add(target: &str) -> Result<()> {
    let repo = paths::repo_dir();
    if !repo.exists() {
        anyhow::bail!("Not initialised. Run `hmz setup <remote>` first.");
    }

    let mut mf = manifest::load()?;

    if is_path(target) {
        add_dotfile(target, &mut mf)?;
    } else {
        add_app(target, &mut mf)?;
    }

    manifest::save(&mf)?;
    Ok(())
}

fn is_path(s: &str) -> bool {
    s.starts_with('/') || s.starts_with("~/") || s.starts_with('.')
}

fn add_app(name: &str, mf: &mut Manifest) -> Result<()> {
    if mf.crates.contains(&name.to_string()) {
        anyhow::bail!("{} is already managed", name);
    }

    let src_data = paths::xdg_data(name);
    let dst_data = paths::repo_app_data(name);
    if src_data.exists() && !src_data.is_symlink() {
        move_and_link(&src_data, &dst_data)
            .with_context(|| format!("linking data dir for {}", name))?;
        println!("  data:   {} -> {}", src_data.display(), dst_data.display());
    }

    let src_cfg = paths::xdg_config(name);
    let dst_cfg = paths::repo_app_config(name);
    if src_cfg.exists() && !src_cfg.is_symlink() {
        move_and_link(&src_cfg, &dst_cfg)
            .with_context(|| format!("linking config dir for {}", name))?;
        println!("  config: {} -> {}", src_cfg.display(), dst_cfg.display());
    }

    mf.crates.push(name.to_string());
    println!("Added '{}' to manifest.", name);
    Ok(())
}

fn add_dotfile(input: &str, mf: &mut Manifest) -> Result<()> {
    let src = expand(input);

    if !src.exists() {
        anyhow::bail!("{} does not exist", src.display());
    }

    let rel = src
        .strip_prefix(paths::home())
        .with_context(|| format!("{} is not under home directory", src.display()))?
        .to_owned();

    let dst = paths::repo_dotfiles().join(&rel);

    if dst.exists() {
        anyhow::bail!("{} is already managed", input);
    }

    move_and_link(&src, &dst).with_context(|| format!("linking dotfile {}", input))?;

    let canonical = format!("~/{}", rel.display());
    if !mf.dotfiles.contains(&canonical) {
        mf.dotfiles.push(canonical.clone());
    }
    println!("Added dotfile '{}'.", canonical);
    Ok(())
}

fn restore_app_symlinks(name: &str) -> Result<()> {
    let dst_data = paths::repo_app_data(name);
    let src_data = paths::xdg_data(name);
    if dst_data.exists() && !src_data.exists() {
        ensure_parent(&src_data)?;
        symlink(&dst_data, &src_data)?;
    }

    let dst_cfg = paths::repo_app_config(name);
    let src_cfg = paths::xdg_config(name);
    if dst_cfg.exists() && !src_cfg.exists() {
        ensure_parent(&src_cfg)?;
        symlink(&dst_cfg, &src_cfg)?;
    }

    Ok(())
}

fn restore_dotfile_symlink(dotfile: &str) -> Result<()> {
    let src = expand(dotfile);
    let rel = match src.strip_prefix(paths::home()) {
        Ok(r) => r.to_owned(),
        Err(_) => return Ok(()),
    };
    let dst = paths::repo_dotfiles().join(&rel);
    if dst.exists() && !src.exists() {
        ensure_parent(&src)?;
        symlink(&dst, &src)?;
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

fn expand(input: &str) -> PathBuf {
    if let Some(rest) = input.strip_prefix("~/") {
        paths::home().join(rest)
    } else {
        PathBuf::from(input)
    }
}
