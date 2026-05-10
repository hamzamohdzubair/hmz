use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::{manifest, paths, registry};
use crate::registry::Loc;

pub fn run() -> Result<()> {
    sync_win_appdata()?;

    let repo = paths::repo_dir();
    git(&repo, &["add", "."])?;

    let dirty = !Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(&repo)
        .status()
        .context("git not found")?
        .success();

    if dirty {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        git(&repo, &["commit", "-m", &format!("backup {}", now)])?;
    }

    git(&repo, &["push", "-u", "origin", "HEAD"])?;
    Ok(())
}

pub fn status() -> Result<()> {
    Command::new("git")
        .args(["status", "--short"])
        .current_dir(paths::repo_dir())
        .status()
        .context("git not found")?;
    Ok(())
}

fn sync_win_appdata() -> Result<()> {
    if !paths::is_wsl() {
        return Ok(());
    }
    let mf = manifest::load()?;
    for name in &mf.apps {
        let def = match registry::lookup(name) {
            Ok(d) => d,
            Err(_) => continue,
        };
        for loc in &def.locs {
            if let Loc::WinAppData = loc {
                let src = paths::win_appdata(name)?;
                let dst = paths::repo_app_loc(name, loc.dir_name());
                if src.exists() {
                    copy_dir(&src, &dst)
                        .with_context(|| format!("syncing win_appdata for {}", name))?;
                }
            }
        }
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)
                .with_context(|| format!("copying {}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn git(repo: &std::path::Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo)
        .status()
        .context("git not found")?;
    if !status.success() {
        anyhow::bail!("git {} failed", args.join(" "));
    }
    Ok(())
}
