use std::path::PathBuf;
use std::process::Command;

pub fn repo_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hmz")
}

pub fn manifest_path() -> PathBuf {
    repo_dir().join("manifest.toml")
}

pub fn repo_crate(name: &str) -> PathBuf {
    repo_dir().join("crates").join(name)
}

pub fn repo_crate_config(name: &str) -> PathBuf {
    repo_crate(name).join("config")
}

pub fn repo_crate_data(name: &str) -> PathBuf {
    repo_crate(name).join("data")
}

pub fn repo_app(name: &str) -> PathBuf {
    repo_dir().join("apps").join(name)
}

pub fn repo_app_loc(name: &str, loc: &str) -> PathBuf {
    repo_app(name).join(loc)
}

/// Always ~/.config/<name> regardless of OS (XDG default).
pub fn xdg_config(name: &str) -> PathBuf {
    home().join(".config").join(name)
}

/// Always ~/.local/share/<name> regardless of OS (XDG default).
pub fn xdg_data(name: &str) -> PathBuf {
    home().join(".local").join("share").join(name)
}

pub fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn is_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .map(|v| v.to_lowercase().contains("microsoft"))
        .unwrap_or(false)
}

/// Returns the WSL path to %APPDATA%/<name>. Only available on WSL.
pub fn win_appdata(name: &str) -> anyhow::Result<PathBuf> {
    let out = Command::new("cmd.exe")
        .args(["/c", "echo %APPDATA%"])
        .output()
        .map_err(|_| anyhow::anyhow!("cmd.exe not found — not running under WSL?"))?;
    let win_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if win_path.contains('%') {
        anyhow::bail!("APPDATA env var did not expand");
    }
    let wsl_out = Command::new("wslpath")
        .arg(&win_path)
        .output()
        .map_err(|_| anyhow::anyhow!("wslpath not found"))?;
    let base = PathBuf::from(String::from_utf8_lossy(&wsl_out.stdout).trim().to_string());
    Ok(base.join(name))
}
