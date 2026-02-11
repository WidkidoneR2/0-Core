//! VPN widget - Shows Mullvad connection status

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct VpnWidget {
    connected: bool,
    last_check: std::time::Instant,
}

impl VpnWidget {
    pub fn new() -> Self {
        Self {
            connected: false,
            last_check: std::time::Instant::now(),
        }
    }

    fn check_vpn_status() -> bool {
        if let Ok(output) = Command::new("mullvad").arg("status").output() {
            let result = String::from_utf8_lossy(&output.stdout);
            result.contains("Connected")
        } else {
            false
        }
    }
}

impl Widget for VpnWidget {
    fn name(&self) -> &'static str {
        "vpn"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        // Check every 5 seconds
        if self.last_check.elapsed().as_secs() >= 5 {
            self.connected = Self::check_vpn_status();
            self.last_check = std::time::Instant::now();
        }
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let (text, color) = if self.connected {
            ("VPN ON", colors::SUCCESS)
        } else {
            ("VPN OFF", colors::FG)
        };

        Ok(WidgetOutput {
            text: text.to_string(),
            color,
            width: 60,
            clickable: true,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        // Toggle VPN
        if self.connected {
            let _ = Command::new("mullvad").arg("disconnect").spawn();
        } else {
            let _ = Command::new("mullvad").arg("connect").spawn();
        }
        Ok(())
    }
}
