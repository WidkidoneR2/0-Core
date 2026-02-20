#![allow(dead_code)]
//! Lock widget - 0-Core immutable status

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use faelight_core::paths;
use std::process::Command;

pub struct LockWidget {
    locked: bool,
}

impl LockWidget {
    pub fn new() -> Self {
        Self { locked: false }
    }

    fn check_locked() -> bool {
        let output = Command::new("lsattr")
            .args(["-d"])
            .arg(paths::core_dir())
            .output();
        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                stdout
                    .split_whitespace()
                    .next()
                    .is_some_and(|attrs| attrs.contains('i'))
            }
            _ => false,
        }
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
