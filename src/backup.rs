use std::process::Command;

use anyhow::{Context, Result};

use crate::paths;

pub fn run() -> Result<()> {
    let repo = paths::repo_dir();

    git(&repo, &["add", "."])?;

    // Only commit if there are staged changes.
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

    // -u sets upstream on first push; no-op on subsequent pushes.
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
