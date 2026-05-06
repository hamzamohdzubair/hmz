use std::path::PathBuf;

pub fn repo_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hmz")
}

pub fn manifest_path() -> PathBuf {
    repo_dir().join("manifest.toml")
}

pub fn repo_app_data(app: &str) -> PathBuf {
    repo_dir().join(app).join("data")
}

pub fn repo_app_config(app: &str) -> PathBuf {
    repo_dir().join(app).join("config")
}

pub fn repo_dotfiles() -> PathBuf {
    repo_dir().join("dotfiles")
}

pub fn xdg_data(app: &str) -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(app)
}

pub fn xdg_config(app: &str) -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(app)
}

pub fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}
