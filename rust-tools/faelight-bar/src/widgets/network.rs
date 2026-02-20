#![allow(dead_code)]
//! Network widget - wifi on/off with SSID, green/red

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct NetworkWidget {
    text: String,
    connected: bool,
}

impl NetworkWidget {
    pub fn new() -> Self {
        Self {
            text: "WIFI:??".to_string(),
            connected: false,
        }
    }

    fn check_network() -> (String, bool) {
        if let Ok(output) = Command::new("nmcli")
            .args(["-t", "-f", "active,ssid", "dev", "wifi"])
            .output()
        {
            if let Ok(result) = String::from_utf8(output.stdout) {
                for line in result.lines() {
                    if line.starts_with("yes:") {
                        let ssid = line.trim_start_matches("yes:").trim();
                        let label = if ssid.is_empty() {
                            "ON".to_string()
                        } else {
                            ssid.to_string()
                        };
                        return (format!("WIFI:{}", label), true);
                    }
                }
                return ("WIFI:OFF".to_string(), false);
            }
        }
        ("WIFI:??".to_string(), false)
    }
}

impl Widget for NetworkWidget {
    fn name(&self) -> &'static str {
        "network"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        let (text, connected) = Self::check_network();
        self.text = text;
        self.connected = connected;
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let color = if self.connected {
            colors::SUCCESS
        } else {
            colors::ERROR
        };
        Ok(WidgetOutput {
            text: self.text.clone(),
            color,
            width: 100,
            clickable: false,
        })
    }
}
