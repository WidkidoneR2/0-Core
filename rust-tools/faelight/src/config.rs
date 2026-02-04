//! Configuration management for faelight CLI
use faelight_core::paths;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FaelightConfig {
    pub theme: String,
    pub profile: String,
    pub features: Features,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    pub auto_update: bool,
    pub doctor_notifications: bool,
    pub git_hooks: bool,
}

impl Default for FaelightConfig {
    fn default() -> Self {
        Self {
            theme: "faelight-forest".to_string(),
            profile: "default".to_string(),
            features: Features {
                auto_update: true,
                doctor_notifications: true,
                git_hooks: true,
            },
        }
    }
}

impl FaelightConfig {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        
        if let Ok(contents) = fs::read_to_string(&config_path) {
            toml::from_str(&contents).unwrap_or_default()
        } else {
            Self::default()
        }
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;
        
        Ok(())
    }
    
    pub fn config_path() -> PathBuf {
        paths::faelight_config_dir().join("cli.toml")
    }
}
