//! Widget system for faelight-bar v4.0.0

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)] // Error variants used in Phase 2+
pub enum WidgetError {
    #[error("Failed to update widget: {0}")]
    Update(String),

    #[error("Failed to render widget: {0}")]
    Render(String),

    #[error("Click action failed: {0}")]
    Click(String),
}

#[allow(dead_code)] // Fields used in Phase 2+
pub struct RenderContext {
    pub width: u32,
    pub height: u32,
    pub x_offset: i32,
}

#[allow(dead_code)] // Fields used in Phase 2+
pub struct WidgetOutput {
    pub text: String,
    pub color: u32,
    pub width: i32,
    pub clickable: bool,
}

pub trait Widget: Send {
    /// Widget name for logging
    fn name(&self) -> &'static str;

    /// Update widget state (called periodically)
    fn update(&mut self) -> Result<(), WidgetError>;

    /// Render widget to string with color
    fn render(&self, ctx: &RenderContext) -> Result<WidgetOutput, WidgetError>;

    /// Get click region if clickable
    #[allow(dead_code)]
    fn click_region(&self) -> Option<(i32, i32)> {
        None
    }

    /// Handle click event
    #[allow(dead_code)]
    fn on_click(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    /// Get error state if widget is degraded
    #[allow(dead_code)]
    fn error_state(&self) -> Option<&str> {
        None
    }

    /// Reset widget state
    #[allow(dead_code)]
    fn reset(&mut self) {}
}

// Widget modules
pub mod clock;

pub use clock::ClockWidget;
