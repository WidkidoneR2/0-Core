//! Wayland platform layer
//! 
//! Handles Wayland connection, surface, and buffer management

pub mod buffer;
pub mod surface;

pub use buffer::BufferManager;
pub use surface::BrowserSurface;
