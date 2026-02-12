//! Battery widget - Shows battery charge percentage

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use std::fs;

pub struct BatteryWidget {
    charge: String,
}

impl BatteryWidget {
    pub fn new() -> Self {
        Self {
            charge: String::from("▮ ??"),
        }
    }

    fn get_battery_level() -> Option<u8> {
        if let Ok(capacity) = fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
            return capacity.trim().parse().ok();
        }
        if let Ok(capacity) = fs::read_to_string("/sys/class/power_supply/BAT1/capacity") {
            return capacity.trim().parse().ok();
        }
        None
    }

    fn get_charging_status() -> bool {
        if let Ok(status) = fs::read_to_string("/sys/class/power_supply/BAT0/status") {
            return status.trim() == "Charging";
        }
        if let Ok(status) = fs::read_to_string("/sys/class/power_supply/BAT1/status") {
            return status.trim() == "Charging";
        }
        false
    }
}

impl Widget for BatteryWidget {
    fn name(&self) -> &'static str {
        "battery"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        if let Some(level) = Self::get_battery_level() {
            let charging = Self::get_charging_status();
            self.charge = if charging {
                format!("▮ {}%+", level)
            } else {
                format!("▮ {}%", level)
            };
        } else {
            self.charge = String::from("▮ ??");
        }
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let color = if self.charge.contains("??") {
            0xFFAAAAAA // Gray for unknown
        } else if self.charge.contains("+") {
            0xFF00FF9F // Green when charging
        } else {
            let level: u8 = self
                .charge
                .trim_start_matches("BAT:")
                .parse()
                .unwrap_or(100);

            if level < 20 {
                0xFFFF6B6B // Red when low
            } else {
                0xFFFFD700 // Gold/Yellow normal
            }
        };

        Ok(WidgetOutput {
            text: self.charge.clone(),
            color,
            width: 60,
            clickable: false,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}
