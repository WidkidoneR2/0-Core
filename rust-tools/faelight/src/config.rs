//! Configuration loading and validation

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub system: SystemConfig,
    pub paths: PathsConfig,
    pub health: HealthConfig,
    pub notifications: NotificationConfig,
    pub bar: BarConfig,
    pub lock: LockConfig,
}

#[derive(Debug, Deserialize)]
pub struct SystemConfig {
    pub version: String,
    pub theme: String,
    pub default_profile: String,
}

#[derive(Debug, Deserialize)]
pub struct PathsConfig {
    pub core_dir: String,
    pub scripts_dir: String,
    pub state_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthConfig {
    pub fail_on_warning: bool,
    pub auto_check_on_unlock: bool,
}

#[derive(Debug, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub timeout_ms: u32,
    pub position: String,
}

#[derive(Debug, Deserialize)]
pub struct BarConfig {
    pub refresh_ms: u32,
    pub show_vpn: bool,
    pub show_battery: bool,
    pub show_volume: bool,
}

#[derive(Debug, Deserialize)]
pub struct LockConfig {
    pub timeout_minutes: u32,
    pub show_clock: bool,
    pub blur_background: bool,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub description: String,
    pub icon: String,
    pub vpn: bool,
    pub cpu_governor: String,
    pub notifications: String,
    #[serde(default)]
    pub gpu_mode: Option<String>,
    #[serde(default)]
    pub bar_modules: Vec<String>,
    #[serde(default)]
    pub auto_launch: Vec<String>,
    #[serde(default)]
    pub bar_refresh_ms: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub accent_secondary: String,
    pub highlight: String,
    pub warning: String,
    pub error: String,
    pub dim: String,
    pub border: String,
    pub selected: String,
}

#[derive(Debug, Deserialize)]
pub struct ThemeComponent {
    pub bg: String,
    pub fg: String,
    #[serde(default)]
    pub border: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub separator: Option<String>,
    #[serde(default)]
    pub selected_bg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Theme {
    pub description: String,
    pub colors: ThemeColors,
    #[serde(default)]
    pub bar: Option<ThemeComponent>,
    #[serde(default)]
    pub notify: Option<ThemeComponent>,
    #[serde(default)]
    pub menu: Option<ThemeComponent>,
}

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".config/faelight")
}

pub fn load_config() -> Result<Config, String> {
    let path = config_dir().join("config.toml");
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config.toml: {}", e))?;
    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config.toml: {}", e))
}

pub fn load_profiles() -> Result<HashMap<String, Profile>, String> {
    let path = config_dir().join("profiles.toml");
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read profiles.toml: {}", e))?;
    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse profiles.toml: {}", e))
}

pub fn load_themes() -> Result<HashMap<String, Theme>, String> {
    let path = config_dir().join("themes.toml");
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read themes.toml: {}", e))?;
    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse themes.toml: {}", e))
}

pub fn validate_all() -> Vec<(String, Result<(), String>)> {
    vec![
        ("config.toml".to_string(), load_config().map(|_| ())),
        ("profiles.toml".to_string(), load_profiles().map(|_| ())),
        ("themes.toml".to_string(), load_themes().map(|_| ())),
    ]
}
