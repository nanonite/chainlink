use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainlinkConfig {
    #[serde(default)]
    pub logseq: LogseqConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogseqConfig {
    pub graph_dir: Option<PathBuf>,
}

impl ChainlinkConfig {
    pub fn load(chainlink_dir: &Path) -> Result<Self> {
        let path = config_path(chainlink_dir);
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))
    }

    pub fn save(&self, chainlink_dir: &Path) -> Result<()> {
        fs::create_dir_all(chainlink_dir)
            .with_context(|| format!("Failed to create {}", chainlink_dir.display()))?;
        let path = config_path(chainlink_dir);
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))
    }
}

pub fn config_path(chainlink_dir: &Path) -> PathBuf {
    chainlink_dir.join("config.toml")
}
