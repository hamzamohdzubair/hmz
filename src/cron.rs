use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

const ENTRY: &str = "0 * * * * hmz backup";
const MARKER: &str = "hmz backup";

pub fn install() -> Result<()> {
    let current = read_crontab();
    if current.contains(MARKER) {
        return Ok(());
    }
    let new = format!("{}\n{}\n", current.trim_end(), ENTRY);
    write_crontab(&new)?;
    println!("Hourly backup cron job registered.");
    Ok(())
}

pub fn remove() -> Result<()> {
    let current = read_crontab();
    if !current.contains(MARKER) {
        return Ok(());
    }
    let new: String = current
        .lines()
        .filter(|l| !l.contains(MARKER))
        .map(|l| format!("{}\n", l))
        .collect();
    write_crontab(&new)?;
    println!("Hourly backup cron job removed.");
    Ok(())
}

fn read_crontab() -> String {
    Command::new("crontab")
        .arg("-l")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

fn write_crontab(content: &str) -> Result<()> {
    let mut child = Command::new("crontab")
        .arg("-")
        .stdin(Stdio::piped())
        .spawn()
        .context("crontab not found")?;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(content.as_bytes())?;
    child.wait()?;
    Ok(())
}
