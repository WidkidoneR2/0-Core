#![allow(dead_code)]
//! Time display block - shows current time
use super::{Block, BlockAction, BlockAlign};
use chrono::Local;

pub struct TimeBlock {
    time_str: String,
}

impl TimeBlock {
    pub fn new() -> Self {
        Self {
            time_str: String::new(),
        }
    }
}

impl Block for TimeBlock {
    fn name(&self) -> &str { "time" }
    
    fn update(&mut self) -> Result<(), String> {
        self.time_str = Local::now().format("%b %d %H:%M").to_string();
        Ok(())
    }
    
    fn text(&self) -> String {
        self.time_str.clone()
    }
    
    fn color(&self) -> [u8; 4] {
        [0x77, 0xc1, 0xf5, 0xFF] // Amber
    }
    
    fn on_click(&mut self, _button: u32) -> Option<BlockAction> {
        None // Time not clickable (yet - could open calendar!)
    }
    
    fn interval(&self) -> u64 {
        60000 // Update every minute
    }
    
    fn separator(&self) -> bool { false } // Last block, no separator
    
    fn align(&self) -> BlockAlign { BlockAlign::Right }
}
