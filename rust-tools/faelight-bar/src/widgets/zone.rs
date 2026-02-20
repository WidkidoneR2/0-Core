#![allow(dead_code)]
//! Zone widget - current zone with icon and color

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct ZoneWidget {
    text: String,
    color: u32,
}

impl ZoneWidget {
    pub fn new() -> Self {
        Self {
            text: "\u{1F3E0} HOME".to_string(),
            color: colors::ACCENT,
        }
    }

    fn get_zone() -> (String, u32) {
        let output = Command::new("faelight-zone").arg("--label").output().ok();
        let label = output
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .trim()
            .to_lowercase();

        match label.as_str() {
            "home" => ("\u{1F3E0} HOME".to_string(), colors::ACCENT),
            "core" => ("\u{1F332} CORE".to_string(), colors::SUCCESS),
            "work" => ("\u{1F4BC} WORK".to_string(), colors::ACCENT_BLUE),
            "gaming" => ("\u{1F3AE} GAME".to_string(), colors::ERROR),
            "focus" => ("\u{1F3AF} FOCUS".to_string(), colors::WARNING),
            "learning" | "learn" => ("\u{1F4DA} LEARN".to_string(), colors::ACCENT_BLUE),
            s if !s.is_empty() => (format!("\u{25B6} {}", s.to_uppercase()), colors::FG),
            _ => ("\u{1F3E0} HOME".to_string(), colors::ACCENT),
        }
    }
}

impl Widget for ZoneWidget {
    fn name(&self) -> &'static str {
        "zone"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        let (text, color) = Self::get_zone();
        self.text = text;
        self.color = color;
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        Ok(WidgetOutput {
            text: self.text.clone(),
            color: self.color,
            width: 130,
            clickable: false,
        })
    }
}
