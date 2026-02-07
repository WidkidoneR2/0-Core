use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub colors: Colors,
    #[serde(default)]
    pub font: Font,
    #[serde(default)]
    pub window: Window,
    #[serde(default)]
    pub behavior: Behavior,
    #[serde(default)]
    pub keybindings: KeyBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Colors {
    #[serde(default = "default_foreground")]
    pub foreground: String,
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default = "default_cursor")]
    pub cursor: String,
    #[serde(default = "default_selection")]
    pub selection: String,
    
    // ANSI colors
    #[serde(default = "default_black")]
    pub black: String,
    #[serde(default = "default_red")]
    pub red: String,
    #[serde(default = "default_green")]
    pub green: String,
    #[serde(default = "default_yellow")]
    pub yellow: String,
    #[serde(default = "default_blue")]
    pub blue: String,
    #[serde(default = "default_magenta")]
    pub magenta: String,
    #[serde(default = "default_cyan")]
    pub cyan: String,
    #[serde(default = "default_white")]
    pub white: String,
    
    // Bright colors
    #[serde(default = "default_bright_black")]
    pub bright_black: String,
    #[serde(default = "default_bright_red")]
    pub bright_red: String,
    #[serde(default = "default_bright_green")]
    pub bright_green: String,
    #[serde(default = "default_bright_yellow")]
    pub bright_yellow: String,
    #[serde(default = "default_bright_blue")]
    pub bright_blue: String,
    #[serde(default = "default_bright_magenta")]
    pub bright_magenta: String,
    #[serde(default = "default_bright_cyan")]
    pub bright_cyan: String,
    #[serde(default = "default_bright_white")]
    pub bright_white: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Font {
    #[serde(default = "default_font_family")]
    pub family: String,
    #[serde(default = "default_font_size")]
    pub size: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    #[serde(default = "default_padding")]
    pub padding: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Behavior {
    #[serde(default = "default_scrollback_lines")]
    pub scrollback_lines: usize,
    #[serde(default = "default_cursor_blink_ms")]
    pub cursor_blink_ms: u64,
    #[serde(default = "default_cursor_style")]
    pub cursor_style: String,
    #[serde(default = "default_selection_style")]
    pub selection_style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(default = "default_copy")]
    pub copy: String,
    #[serde(default = "default_paste")]
    pub paste: String,
    #[serde(default = "default_zoom_in")]
    pub zoom_in: String,
    #[serde(default = "default_zoom_out")]
    pub zoom_out: String,
    #[serde(default = "default_zoom_reset")]
    pub zoom_reset: String,
    #[serde(default = "default_scroll_up")]
    pub scroll_up: String,
    #[serde(default = "default_scroll_down")]
    pub scroll_down: String,
}

// Default value functions
fn default_foreground() -> String { "#E6E6E6".to_string() }
fn default_background() -> String { "#0F140E".to_string() }
fn default_cursor() -> String { "#6BE3A3".to_string() }
fn default_selection() -> String { "#6BE3A3".to_string() }
fn default_black() -> String { "#0F140E".to_string() }
fn default_red() -> String { "#E36B6B".to_string() }
fn default_green() -> String { "#6BE3A3".to_string() }
fn default_yellow() -> String { "#E3C66B".to_string() }
fn default_blue() -> String { "#6BA3E3".to_string() }
fn default_magenta() -> String { "#C66BE3".to_string() }
fn default_cyan() -> String { "#6BE3C6".to_string() }
fn default_white() -> String { "#E6E6E6".to_string() }
fn default_bright_black() -> String { "#4A4A4A".to_string() }
fn default_bright_red() -> String { "#FF7A7A".to_string() }
fn default_bright_green() -> String { "#7AFFB2".to_string() }
fn default_bright_yellow() -> String { "#FFD67A".to_string() }
fn default_bright_blue() -> String { "#7AB2FF".to_string() }
fn default_bright_magenta() -> String { "#D67AFF".to_string() }
fn default_bright_cyan() -> String { "#7AFFD6".to_string() }
fn default_bright_white() -> String { "#FFFFFF".to_string() }

fn default_font_family() -> String { "JetBrainsMono Nerd Font".to_string() }
fn default_font_size() -> f32 { 14.0 }
fn default_padding() -> u32 { 15 }
fn default_scrollback_lines() -> usize { 10000 }
fn default_cursor_blink_ms() -> u64 { 500 }
fn default_cursor_style() -> String { "line".to_string() }
fn default_selection_style() -> String { "underline-border".to_string() }
fn default_copy() -> String { "Ctrl+Shift+C".to_string() }
fn default_paste() -> String { "Ctrl+Shift+V".to_string() }
fn default_zoom_in() -> String { "Ctrl+Plus".to_string() }
fn default_zoom_out() -> String { "Ctrl+Minus".to_string() }
fn default_zoom_reset() -> String { "Ctrl+0".to_string() }
fn default_scroll_up() -> String { "Shift+PageUp".to_string() }
fn default_scroll_down() -> String { "Shift+PageDown".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            colors: Colors::default(),
            font: Font::default(),
            window: Window::default(),
            behavior: Behavior::default(),
            keybindings: KeyBindings::default(),
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            foreground: default_foreground(),
            background: default_background(),
            cursor: default_cursor(),
            selection: default_selection(),
            black: default_black(),
            red: default_red(),
            green: default_green(),
            yellow: default_yellow(),
            blue: default_blue(),
            magenta: default_magenta(),
            cyan: default_cyan(),
            white: default_white(),
            bright_black: default_bright_black(),
            bright_red: default_bright_red(),
            bright_green: default_bright_green(),
            bright_yellow: default_bright_yellow(),
            bright_blue: default_bright_blue(),
            bright_magenta: default_bright_magenta(),
            bright_cyan: default_bright_cyan(),
            bright_white: default_bright_white(),
        }
    }
}

impl Default for Font {
    fn default() -> Self {
        Self {
            family: default_font_family(),
            size: default_font_size(),
        }
    }
}

impl Default for Window {
    fn default() -> Self {
        Self { padding: default_padding() }
    }
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            scrollback_lines: default_scrollback_lines(),
            cursor_blink_ms: default_cursor_blink_ms(),
            cursor_style: default_cursor_style(),
            selection_style: default_selection_style(),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            copy: default_copy(),
            paste: default_paste(),
            zoom_in: default_zoom_in(),
            zoom_out: default_zoom_out(),
            zoom_reset: default_zoom_reset(),
            scroll_up: default_scroll_up(),
            scroll_down: default_scroll_down(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(contents) => {
                    match toml::from_str(&contents) {
                        Ok(config) => {
                            println!("✅ Loaded config from: {}", config_path.display());
                            config
                        }
                        Err(e) => {
                            eprintln!("⚠️  Failed to parse config: {}", e);
                            eprintln!("   Using defaults");
                            Config::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to read config: {}", e);
                    eprintln!("   Using defaults");
                    Config::default()
                }
            }
        } else {
            println!("ℹ️  No config found, using defaults");
            println!("   Create one at: {}", config_path.display());
            Config::default()
        }
    }
    
    pub fn config_path() -> PathBuf {
        faelight_core::paths::faelight_config_dir().join("term.toml")
    }
    
    
    /// Parse hex color to RGB bytes [r, g, b]
    pub fn parse_color(hex: &str) -> [u8; 3] {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return [r, g, b];
            }
        }
        // Fallback
        [255, 255, 255]
    }
}
