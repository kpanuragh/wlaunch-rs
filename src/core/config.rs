use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub gemini_api_key: Option<String>,
    #[serde(default)]
    pub bitwarden_server: Option<String>,
    #[serde(default)]
    pub bitwarden_email: Option<String>,
    #[serde(default)]
    pub clipboard_history_size: Option<usize>,
    #[serde(default)]
    pub max_recent_files: Option<usize>,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("wlaunch")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(Self::config_path(), content)?;
        Ok(())
    }

    pub fn clipboard_history_size(&self) -> usize {
        self.clipboard_history_size.unwrap_or(50)
    }

    pub fn max_recent_files(&self) -> usize {
        self.max_recent_files.unwrap_or(100)
    }

    pub fn scripts_dir() -> PathBuf {
        Self::config_dir().join("scripts")
    }

    pub fn data_path(name: &str) -> PathBuf {
        Self::config_dir().join(name)
    }
}
