#![allow(dead_code)]
//! Clock widget - Shows current time

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use chrono::Local;

pub struct ClockWidget {
    time: String,
}

impl ClockWidget {
    pub fn new() -> Self {
        Self {
            time: String::new(),
        }
    }
}

impl Widget for ClockWidget {
    fn name(&self) -> &'static str {
        "clock"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.time = Local::now().format("%H:%M").to_string();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        Ok(WidgetOutput {
            text: self.time.clone(), // Just time, no emoji
            color: 0xFFFFFFFF,       // Bright white
            width: 40,
            clickable: false,
        })
    }
}
