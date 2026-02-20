#![allow(dead_code, unused_imports)]
//! Widget system for faelight-bar v4.0.0

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum WidgetError {
    #[error("Failed to update widget: {0}")]
    Update(String),

    #[error("Failed to render widget: {0}")]
    Render(String),

    #[error("Click action failed: {0}")]
    Click(String),
}

#[allow(dead_code)]
pub struct RenderContext {
    pub width: u32,
    pub height: u32,
    pub x_offset: i32,
}

#[allow(dead_code)]
pub struct WidgetOutput {
    pub text: String,
    pub color: u32,
    pub width: i32,
    pub clickable: bool,
}

pub trait Widget: Send {
    fn name(&self) -> &'static str;
    fn update(&mut self) -> Result<(), WidgetError>;
    fn render(&self, ctx: &RenderContext) -> Result<WidgetOutput, WidgetError>;

    #[allow(dead_code)]
    fn click_region(&self) -> Option<(i32, i32)> {
        None
    }

    #[allow(dead_code)]
    fn on_click(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    #[allow(dead_code)]
    fn error_state(&self) -> Option<&str> {
        None
    }

    #[allow(dead_code)]
    fn reset(&mut self) {}
}

// Widget modules
pub mod clock;
pub mod profile;
pub mod volume;
pub mod vpn;

pub use clock::ClockWidget;
pub use profile::ProfileWidget;
pub use volume::VolumeWidget;
pub use vpn::VpnWidget;

// Battery widget
mod battery;
pub use battery::BatteryWidget;

// Network widget
mod network;
pub use network::NetworkWidget;

// Date widget
mod date;
pub use date::DateWidget;

// Lock widget
mod lock;
pub use lock::LockWidget;

// Zone widget
mod zone;
pub use zone::ZoneWidget;

// Health widget
mod health;
pub use health::HealthWidget;

// Search widget
