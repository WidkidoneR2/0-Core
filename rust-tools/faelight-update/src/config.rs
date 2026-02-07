use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub skip_packages: Vec<String>,
    
    #[serde(default)]
    pub skip_categories: Vec<String>,
    
    #[serde(default)]
    pub auto_yes_categories: Vec<String>,
    
    #[serde(default)]
    pub update_order: Vec<String>,
    
    #[serde(default)]
    pub parallel_updates: bool,
    
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u32,
    
    #[serde(default)]
    pub notify_on_updates: bool,
}

fn default_check_interval() -> u32 {
    24
}

impl Default for Config {
    fn default() -> Self {
        Self {
            skip_packages: Vec::new(),
            skip_categories: Vec::new(),
            auto_yes_categories: Vec::new(),
            update_order: vec![
                "System Packages".to_string(),
                "AUR Packages".to_string(),
                "Rust Toolchain".to_string(),
                "Cargo Tools".to_string(),
                "0-Core Workspace".to_string(),
            ],
            parallel_updates: false,
            check_interval_hours: 24,
            notify_on_updates: false,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            // Create default config
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        
        let content = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        std::fs::write(&config_path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        Ok(PathBuf::from(home)
            .join(".config")
            .join("faelight-update")
            .join("config.toml"))
    }
    
    pub fn should_skip_package(&self, package: &str) -> bool {
        self.skip_packages.iter().any(|p| package.contains(p))
    }
    
    pub fn should_skip_category(&self, category: &str) -> bool {
        self.skip_categories.iter().any(|c| category.contains(c))
    }
    
    pub fn should_auto_yes(&self, category: &str) -> bool {
        self.auto_yes_categories.iter().any(|c| category.contains(c))
    }
}
