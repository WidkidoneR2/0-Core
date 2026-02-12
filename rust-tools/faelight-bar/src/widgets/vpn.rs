//! VPN widget

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct VpnWidget {
    status: String,
}

impl VpnWidget {
    pub fn new() -> Self {
        Self {
            status: String::from("VPN:??"),
        }
    }

    fn check_vpn() -> bool {
        if let Ok(output) = Command::new("mullvad").arg("status").output() {
            if let Ok(result) = String::from_utf8(output.stdout) {
                return result.to_lowercase().contains("connected");
            }
        }
        false
    }
}

impl Widget for VpnWidget {
    fn name(&self) -> &'static str {
        "vpn"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        let connected = Self::check_vpn();
        self.status = if connected {
            String::from("VPN ON")
        } else {
            String::from("VPN OFF")
        };
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let connected = Self::check_vpn();
        let color = if connected {
            colors::SUCCESS
        } else {
            colors::FG
        };

        Ok(WidgetOutput {
            text: self.status.clone(),
            color,
            width: 80,
            clickable: false, // Disable for now
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        // Do nothing for now
        Ok(())
    }
}
