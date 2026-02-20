#![allow(dead_code)]
//! Battery widget - % with gradient color and charging indicator

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::fs;

pub struct BatteryWidget {
    text: String,
    color: u32,
}

impl BatteryWidget {
    pub fn new() -> Self {
        Self {
            text: "BAT:??".to_string(),
            color: colors::FG,
        }
    }

    fn get_battery() -> (String, u32) {
        let capacity = fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
            .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/capacity"))
            .unwrap_or_default();
        let status = fs::read_to_string("/sys/class/power_supply/BAT0/status")
            .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/status"))
            .unwrap_or_default();

        let level: u8 = capacity.trim().parse().unwrap_or(0);
        let charging = status.trim() == "Charging";

        let text = if charging {
            format!("\u{26A1}{}%", level)
        } else {
            format!("\u{1F50B}{}%", level)
        };

        let color = if charging {
            colors::ACCENT_BLUE
        } else if level > 50 {
            colors::SUCCESS
        } else if level > 20 {
            colors::WARNING
        } else {
            colors::ERROR
        };

        (text, color)
    }
}

impl Widget for BatteryWidget {
    fn name(&self) -> &'static str {
        "battery"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        let (text, color) = Self::get_battery();
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
