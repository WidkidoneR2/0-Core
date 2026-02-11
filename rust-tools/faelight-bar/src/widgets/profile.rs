//! Profile widget - Shows current profile

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::fs;
use std::process::Command;

pub struct ProfileWidget {
    current: String,
}

impl ProfileWidget {
    pub fn new() -> Self {
        Self {
            current: "default".to_string(),
        }
    }

    fn get_current_profile() -> String {
        let path = dirs::home_dir()
            .map(|h| h.join(".local/share/profile/current"))
            .unwrap_or_default();

        fs::read_to_string(&path)
            .unwrap_or_else(|_| "default".to_string())
            .trim()
            .to_string()
    }
}

impl Widget for ProfileWidget {
    fn name(&self) -> &'static str {
        "profile"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.current = Self::get_current_profile();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let text = format!("PROF:{}", self.current.to_uppercase());

        Ok(WidgetOutput {
            text,
            color: colors::ACCENT,
            width: 100,
            clickable: true,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        // Cycle profiles
        let next = match self.current.as_str() {
            "default" => "gaming",
            "gaming" => "work",
            "work" => "low-power",
            _ => "default",
        };

        let _ = Command::new("profile").arg(next).spawn();
        Ok(())
    }
}
