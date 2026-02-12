//! Lock widget - Shows 0-Core lock status (read-only)

use super::{RenderContext, Widget, WidgetError, WidgetOutput};
use crate::render::colors;
use std::process::Command;

pub struct LockWidget {
    status: String,
    locked: bool,
}

impl LockWidget {
    pub fn new() -> Self {
        Self {
            status: String::from("◯ UNLOCKED"),
            locked: false,
        }
    }

    fn check_lock_status() -> bool {
        // Run the status-blocks/lock script
        if let Some(home) = dirs::home_dir() {
            let lock_script = home.join("0-core/status-blocks/lock");

            if let Ok(output) = Command::new(&lock_script).output() {
                if let Ok(result) = String::from_utf8(output.stdout) {
                    // Check if output contains 🔒 (locked)
                    return result.contains("🔒");
                }
            }
        }
        false
    }
}

impl Widget for LockWidget {
    fn name(&self) -> &'static str {
        "lock"
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.locked = Self::check_lock_status();

        if self.locked {
            self.status = String::from("● LOCKED");
        } else {
            self.status = String::from("◯ UNLOCKED");
        }

        Ok(())
    }

    fn render(&self, _ctx: &RenderContext) -> Result<WidgetOutput, WidgetError> {
        let color = if self.locked {
            0xFFFF6B6B // Red when locked
        } else {
            colors::SUCCESS // Green when unlocked
        };

        Ok(WidgetOutput {
            text: self.status.clone(),
            color,
            width: 140,
            clickable: false, // READ-ONLY - no click action!
        })
    }

    fn on_click(&mut self) -> Result<(), WidgetError> {
        // NO ACTION - this widget is read-only
        Ok(())
    }
}
