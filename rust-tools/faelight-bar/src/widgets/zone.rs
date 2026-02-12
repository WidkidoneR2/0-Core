//! Zone widget with symbols

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use std::fs;

pub struct ZoneWidget {
    zone: String,
}

impl ZoneWidget {
    pub fn new() -> Self {
        Self {
            zone: String::from("▶ ??"),
        }
    }

    fn get_current_zone() -> String {
        if let Some(home) = dirs::home_dir() {
            let zone_file = home.join(".local/share/zone/current");
            if let Ok(zone) = fs::read_to_string(zone_file) {
                let zone_name = zone.trim().to_lowercase();
                let symbol = match zone_name.as_str() {
                    "work" => "◆",     // Diamond for work
                    "focus" => "●",    // Filled circle for focus
                    "creative" => "◇", // Hollow diamond for creative
                    "rest" => "○",     // Hollow circle for rest
                    "learn" => "▲",    // Triangle for learn
                    _ => "▶",          // Arrow default
                };
                return format!("{} {}", symbol, zone.trim().to_uppercase());
            }
        }
        String::from("▶ ??")
    }
}

impl Widget for ZoneWidget {
    fn name(&self) -> &'static str {
        "zone"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.zone = Self::get_current_zone();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        Ok(WidgetOutput {
            text: self.zone.clone(),
            color: 0xFFFF6B6B,
            width: 120,
            clickable: false,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}
