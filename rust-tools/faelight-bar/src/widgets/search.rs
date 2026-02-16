//! Search widget - Launches dmenu on click

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct SearchWidget {
    text: String,
}

impl SearchWidget {
    pub fn new() -> Self {
        Self {
            text: String::from("» SEARCH"),
        }
    }
}

impl Widget for SearchWidget {
    fn name(&self) -> &'static str {
        "search"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        Ok(WidgetOutput {
            text: self.text.clone(),
            color: colors::ACCENT,
            width: 90,
            clickable: true,
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        // Launch faelight-dmenu
        let _ = Command::new("foot")
            .args([
                "--app-id=faelight-palette-float",
                "-e",
                "/home/christian/0-core/target/release/faelight-palette",
            ])
            .spawn();
        Ok(())
    }
}
