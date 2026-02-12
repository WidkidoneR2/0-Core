//! Lock widget with symbol

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct LockWidget {
    text: String,
}

impl LockWidget {
    pub fn new() -> Self {
        Self {
            text: String::from("◯ UNLOCKED"), // Simple circle
        }
    }
}

impl Widget for LockWidget {
    fn name(&self) -> &'static str {
        "lock"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        Ok(WidgetOutput {
            text: self.text.clone(),
            color: colors::FG,
            width: 140,
            clickable: true,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        let _ = Command::new("swaylock").arg("-f").spawn();
        Ok(())
    }
}
