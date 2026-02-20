#![allow(dead_code)]
//! Volume widget - actual percentage, no double-call

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct VolumeWidget {
    muted: bool,
    percent: u8,
}

impl VolumeWidget {
    pub fn new() -> Self {
        Self {
            muted: false,
            percent: 50,
        }
    }

    fn check_volume() -> (bool, u8) {
        if let Ok(output) = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            if let Ok(result) = String::from_utf8(output.stdout) {
                let muted = result.contains("MUTED");
                let percent = result
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<f32>().ok())
                    .map(|v| (v * 100.0) as u8)
                    .unwrap_or(0);
                return (muted, percent);
            }
        }
        (false, 0)
    }
}

impl Widget for VolumeWidget {
    fn name(&self) -> &'static str {
        "volume"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        let (muted, percent) = Self::check_volume();
        self.muted = muted;
        self.percent = percent;
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let (text, color) = if self.muted {
            ("\u{D7} MUTE".to_string(), colors::ERROR)
        } else {
            let color = if self.percent > 70 {
                colors::WARNING
            } else {
                colors::SUCCESS
            };
            (format!("\u{266A} {}%", self.percent), color)
        };
        Ok(WidgetOutput {
            text,
            color,
            width: 80,
            clickable: true,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        let _ = Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
            .spawn();
        Ok(())
    }
}
