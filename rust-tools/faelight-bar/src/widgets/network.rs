//! Network widget - Shows wifi/ethernet connection status

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct NetworkWidget {
    status: String,
}

impl NetworkWidget {
    pub fn new() -> Self {
        Self {
            status: String::from("≈ ??"),
        }
    }

    fn check_network() -> String {
        // Check wifi - just show ON/OFF
        if let Ok(output) = Command::new("nmcli")
            .args(["-t", "-f", "active,ssid", "dev", "wifi"])
            .output()
        {
            if let Ok(result) = String::from_utf8(output.stdout) {
                for line in result.lines() {
                    if line.starts_with("yes:") {
                        return String::from("≈ ON");
                    }
                }
            }
        }

        // Check ethernet
        if let Ok(output) = Command::new("nmcli")
            .args(["-t", "-f", "device,state", "dev", "status"])
            .output()
        {
            if let Ok(result) = String::from_utf8(output.stdout) {
                for line in result.lines() {
                    if line.contains("ethernet") && line.contains("connected") {
                        return String::from("≡ ON");
                    }
                }
            }
        }

        String::from("≈ OFF")
    }
}

impl Widget for NetworkWidget {
    fn name(&self) -> &'static str {
        "network"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.status = Self::check_network();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let color = if self.status.contains(":ON") {
            colors::SUCCESS
        } else {
            colors::FG
        };

        Ok(WidgetOutput {
            text: self.status.clone(),
            color,
            width: 80,
            clickable: false,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}
