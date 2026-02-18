use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitUserConfig {
    pub user_name: String,
    pub user_email: String,
    pub ssh_key_path: PathBuf,
    pub default_branch: String,
    pub commit_template: Option<PathBuf>,
    pub signing_key: Option<String>,
}

impl GitUserConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read config file")?;
        let config: GitUserConfig = toml::from_str(&content).context("Failed to parse config")?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(path, content).context("Failed to write config file")?;
        Ok(())
    }
}
