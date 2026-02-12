//! Health widget - Shows system health percentage

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct HealthWidget {
    health: String,
}

impl HealthWidget {
    pub fn new() -> Self {
        Self {
            health: String::from("HP:??"),
        }
    }

    fn get_health() -> String {
        // Use the actual binary, not the alias
        let doctor_path = "/home/christian/.local/bin/dot-doctor";

        if let Ok(output) = Command::new(doctor_path).output() {
            if let Ok(result) = String::from_utf8(output.stdout) {
                for line in result.lines() {
                    if line.contains("Health:") && line.contains("%") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for part in parts {
                            if part.ends_with('%') {
                                return format!("HP:{}", part);
                            }
                        }
                    }
                }
            }
        }
        String::from("HP:??")
    }
}

impl Widget for HealthWidget {
    fn name(&self) -> &'static str {
        "health"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.health = Self::get_health();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let color = if self.health == "HP:100%" {
            colors::SUCCESS
        } else if self.health.contains("??") {
            colors::FG
        } else {
            colors::WARNING
        };

        Ok(WidgetOutput {
            text: self.health.clone(),
            color,
            width: 60,
            clickable: false,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}
