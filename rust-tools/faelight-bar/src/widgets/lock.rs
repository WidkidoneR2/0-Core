#![allow(dead_code)]
//! Lock widget - 0-Core immutable status

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use faelight_core::paths;

pub struct LockWidget {
    locked: bool,
}

impl LockWidget {
    pub fn new() -> Self {
        Self { locked: false }
    }

    /// INT-251b: read authoritative lock state from runtime/.core-locked.
    /// Written by core-protect on lock/unlock. Faster and more reliable
    /// than parsing lsattr output, no subprocess needed.
    fn check_locked() -> bool {
        paths::core_dir().join("runtime").join(".core-locked").exists()
    }
}

impl Widget for LockWidget {
    fn name(&self) -> &'static str {
        "lock"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.locked = Self::check_locked();
        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let (text, color) = if self.locked {
            ("\u{1F512} LOCKED".to_string(), colors::ACCENT)
        } else {
            ("\u{1F513} UNLOCKED".to_string(), colors::WARNING)
        };
        Ok(WidgetOutput {
            text,
            color,
            width: 120,
            clickable: false,
        })
    }
}
