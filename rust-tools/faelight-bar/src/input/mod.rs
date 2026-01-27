//! Input handling subsystem

pub mod keyboard;

pub use keyboard::{handle_key_press, KeyAction, execute_command};
