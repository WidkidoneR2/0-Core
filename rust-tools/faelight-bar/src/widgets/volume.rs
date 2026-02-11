//! Volume widget - Shows mute status

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct VolumeWidget {
    muted: bool,
}

impl VolumeWidget {
    pub fn new() -> Self {
        Self { muted: false }
    }

    fn check_mute_status() -> bool {
        if let Ok(output) = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            let result = String::from_utf8_lossy(&output.stdout);
            result.contains("MUTED")
        } else {
            false
        }
    }
}

impl Widget for VolumeWidget {
    fn name(&self) -> &'static str {
        "volume"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.muted = Self::check_mute_status();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let (text, color) = if self.muted {
            ("VOL MUTE", colors::ERROR)
        } else {
            ("VOL ON", colors::FG)
        };

        Ok(WidgetOutput {
            text: text.to_string(),
            color,
            width: 70,
            clickable: true,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        // Toggle mute
        let _ = Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
            .spawn();
        Ok(())
    }
}
