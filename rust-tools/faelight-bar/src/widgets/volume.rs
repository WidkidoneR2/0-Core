//! Volume widget - Shows mute status

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use std::process::Command;

pub struct VolumeWidget {
    status: String,
}

impl VolumeWidget {
    pub fn new() -> Self {
        Self {
            status: String::from("♪ ??"),
        }
    }

    fn check_volume() -> (bool, String) {
        if let Ok(output) = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            if let Ok(result) = String::from_utf8(output.stdout) {
                let muted = result.contains("MUTED");
                let status = if muted { "× MUTE" } else { "♪ ON" };
                return (muted, status.to_string());
            }
        }
        (false, String::from("♪ ??"))
    }
}

impl Widget for VolumeWidget {
    fn name(&self) -> &'static str {
        "volume"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        let (_, status) = Self::check_volume();
        self.status = status;
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let (muted, _) = Self::check_volume();
        let color = if muted {
            0xFFFF6B6B // Red when muted
        } else {
            0xFFFFA500 // Orange when on
        };

        Ok(WidgetOutput {
            text: self.status.clone(),
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
