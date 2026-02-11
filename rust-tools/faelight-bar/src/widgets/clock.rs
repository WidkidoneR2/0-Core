//! Clock widget - simplest widget for testing

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use chrono::Local;

pub struct ClockWidget {
    current_time: String,
}

impl ClockWidget {
    pub fn new() -> Self {
        Self {
            current_time: String::new(),
        }
    }
}

impl Widget for ClockWidget {
    fn name(&self) -> &'static str {
        "clock"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.current_time = Local::now().format("%H:%M").to_string();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        Ok(WidgetOutput {
            text: format!("🕐 {}", self.current_time),
            color: colors::FG,
            width: 80, // Approximate width in pixels
            clickable: false,
        })
    }
}
