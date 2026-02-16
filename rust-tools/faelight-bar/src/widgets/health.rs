//! Health widget - Shows system health percentage from doctor

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
        // Use actual binary path (doctor is an alias)
        if let Ok(output) = Command::new("dot-doctor").output() {
            if let Ok(result) = String::from_utf8(output.stdout) {
                // Look for "   Health:   94%" line in Statistics section
                for line in result.lines() {
                    if line.trim_start().starts_with("Health:") {
                        // Extract percentage after "Health:"
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(pct) = parts.last() {
                            if pct.ends_with('%') {
                                return format!("HP:{}", pct);
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
            width: 70,
            clickable: false,
        })
    }
}
