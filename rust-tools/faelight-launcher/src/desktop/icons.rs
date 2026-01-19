//! Icon mapping configuration
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct IconConfig {
    #[serde(default)]
    pub icons: HashMap<String, String>,
}

impl IconConfig {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }

        // Create default config
        let default = Self::default_config();
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&config_path, toml::to_string_pretty(&default).unwrap());
        default
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home)
            .join(".config/faelight-launcher/icons.toml")
    }

    fn default_config() -> Self {
        let mut icons = HashMap::new();
        
        // Your curated apps
        icons.insert("brave".to_string(), "󰖟".to_string());
        icons.insert("brave-browser".to_string(), "󰖟".to_string());
        icons.insert("discord".to_string(), "󰙯".to_string());
        icons.insert("keepassxc".to_string(), "󰌋".to_string());
        icons.insert("filen-desktop".to_string(), "󰅟".to_string());
        icons.insert("thunar".to_string(), "󰉋".to_string());
        icons.insert("notesnook".to_string(), "󱞁".to_string());
        icons.insert("tutanota-desktop".to_string(), "󰇮".to_string());
        icons.insert("foot".to_string(), "".to_string());
        icons.insert("nvim".to_string(), "".to_string());
        icons.insert("yazi".to_string(), "󰉖".to_string());
        icons.insert("btop".to_string(), "󰄪".to_string());
        icons.insert("lazygit".to_string(), "󰊢".to_string());
        
        IconConfig { icons }
    }

    pub fn get_icon(&self, exec: &str, categories: &[String]) -> String {
        // Try to match by exec command (first word)
        let exec_cmd = exec.split_whitespace().next().unwrap_or(exec);
        let exec_name = exec_cmd.split('/').last().unwrap_or(exec_cmd);
        
        if let Some(icon) = self.icons.get(exec_name) {
            return icon.clone();
        }

        // Fall back to category-based icon
        super::get_category_icon(categories).to_string()
    }
}

    // Add fallback matching for terminal emulators and editors
