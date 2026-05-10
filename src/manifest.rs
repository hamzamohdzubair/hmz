use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Manifest {
    #[serde(default)]
    pub crates: Vec<String>,
    #[serde(default)]
    pub apps: Vec<String>,
}

pub fn load() -> Result<Manifest> {
    let path = crate::paths::manifest_path();
    if !path.exists() {
        return Ok(Manifest::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(toml::from_str(&raw)?)
}

pub fn save(mf: &Manifest) -> Result<()> {
    let raw = toml::to_string_pretty(mf)?;
    std::fs::write(crate::paths::manifest_path(), raw)?;
    Ok(())
}
