#![allow(dead_code)]
//! Profile widget - icon per profile, cycle on click (default/gaming/work)

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
        let (text, color) = match self.current.as_str() {
            "gaming" => ("\u{1F3AE} GAME".to_string(), colors::ERROR),
            "work" => ("\u{1F4BC} WORK".to_string(), colors::ACCENT_BLUE),
            _ => ("\u{1F3E0} DEF".to_string(), colors::ACCENT),
        };
        Ok(WidgetOutput {
            text,
            color,
            width: 100,
            clickable: true,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        let next = match self.current.as_str() {
            "default" => "gaming",
            "gaming" => "work",
            _ => "default",
        };
        let _ = Command::new("profile").arg(next).spawn();
        Ok(())
    }
}
