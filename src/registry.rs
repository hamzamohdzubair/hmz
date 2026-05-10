use std::collections::HashMap;

use anyhow::{bail, Result};
use serde::Deserialize;

const SRC: &str = include_str!("registry.toml");

#[derive(Deserialize)]
struct RawEntry {
    locations: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Loc {
    XdgConfig,
    XdgData,
    WinAppData,
    Home(String),
}

impl Loc {
    pub fn dir_name(&self) -> &str {
        match self {
            Loc::XdgConfig => "xdg_config",
            Loc::XdgData => "xdg_data",
            Loc::WinAppData => "win_appdata",
            Loc::Home(_) => "home",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppDef {
    pub name: String,
    pub locs: Vec<Loc>,
}

pub fn lookup(name: &str) -> Result<AppDef> {
    let raw: HashMap<String, RawEntry> = toml::from_str(SRC)
        .map_err(|e| anyhow::anyhow!("registry.toml parse error: {}", e))?;

    let entry = raw.get(name).ok_or_else(|| {
        anyhow::anyhow!(
            "'{}' is not in the registry — add it to src/registry.toml and rebuild",
            name
        )
    })?;

    let mut locs = Vec::new();
    for s in &entry.locations {
        let loc = match s.as_str() {
            "xdg_config" => Loc::XdgConfig,
            "xdg_data" => Loc::XdgData,
            "win_appdata" => Loc::WinAppData,
            s if s.starts_with("home:") => Loc::Home(s[5..].to_string()),
            _ => bail!("unknown location '{}' for '{}' in registry.toml", s, name),
        };
        locs.push(loc);
    }

    Ok(AppDef { name: name.to_string(), locs })
}
