#![allow(dead_code)]
//! VPN widget - green=connected, red=disconnected, no double-call

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct VpnWidget {
    connected: bool,
}

impl VpnWidget {
    pub fn new() -> Self {
        Self { connected: false }
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
        self.connected = Self::check_vpn();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let (text, color) = if self.connected {
            ("VPN ON".to_string(), colors::SUCCESS)
        } else {
            ("VPN OFF".to_string(), colors::ERROR)
        };
        Ok(WidgetOutput {
            text,
            color,
            width: 80,
            clickable: false,
        })
    }
}
