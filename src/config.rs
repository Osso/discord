use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub webhooks: HashMap<String, String>,
    pub default_webhook: Option<String>,
    pub bot_token: Option<String>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Could not find config directory")?;
    Ok(config_dir.join("discord-cli").join("config.json"))
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn load() -> Config {
    let path = match config_path() {
        Ok(p) => p,
        Err(_) => return Config::default(),
    };

    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }
    let contents = serde_json::to_string_pretty(config)?;
    fs::write(&path, contents).context("Failed to write config")?;
    Ok(())
}
