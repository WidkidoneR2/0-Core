//! Date widget - Shows current date

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use chrono::Local;

pub struct DateWidget {
    date: String,
}

impl DateWidget {
    pub fn new() -> Self {
        Self {
            date: String::new(),
        }
    }
}

impl Widget for DateWidget {
    fn name(&self) -> &'static str {
        "date"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.date = Local::now().format("%b %d").to_string();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        Ok(WidgetOutput {
            text: self.date.clone(),
            color: 0xFF87CEEB, // Sky blue
            width: 60,
            clickable: false,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}
