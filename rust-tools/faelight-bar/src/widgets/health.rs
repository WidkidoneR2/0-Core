#![allow(dead_code)]
//! Health widget - gradient: 100%=green, 90%=accent, 80%=yellow, <80%=red

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct HealthWidget {
    text: String,
    color: u32,
}

impl HealthWidget {
    pub fn new() -> Self {
        Self {
            text: "HP:??".to_string(),
            color: colors::FG,
        }
    }

    fn get_health() -> (String, u32) {
        let output = match Command::new("dot-doctor").output() {
            Ok(o) => o,
            Err(_) => return ("HP:??".to_string(), colors::FG),
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.trim_start().starts_with("Health:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pct_str) = parts.last() {
                    if let Ok(num) = pct_str.trim_end_matches('%').parse::<u8>() {
                        let color = if num == 100 {
                            colors::SUCCESS
                        } else if num >= 90 {
                            colors::ACCENT
                        } else if num >= 80 {
                            colors::WARNING
                        } else {
                            colors::ERROR
                        };
                        return (format!("HP:{}%", num), color);
                    }
                }
            }
        }
        ("HP:??".to_string(), colors::FG)
    }
}

impl Widget for HealthWidget {
    fn name(&self) -> &'static str {
        "health"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        let (text, color) = Self::get_health();
        self.text = text;
        self.color = color;
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        Ok(WidgetOutput {
            text: self.text.clone(),
            color: self.color,
            width: 70,
            clickable: false,
        })
    }
}
