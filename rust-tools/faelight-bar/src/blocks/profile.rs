#![allow(dead_code)]
//! Profile indicator block - shows current system profile
use super::{Block, BlockAction, BlockAlign};
use std::fs;

pub struct ProfileBlock {
    current: String,
    stable_count: u32, // For adaptive refresh
}

impl ProfileBlock {
    pub fn new() -> Self {
        Self {
            current: "default".to_string(),
            stable_count: 0,
        }
    }
    
    fn read_profile() -> Result<String, String> {
        let path = crate::paths::current_profile_path();
        fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("Failed to read profile: {}", e))
    }
}

impl Block for ProfileBlock {
    fn name(&self) -> &str { "profile" }
    
    fn update(&mut self) -> Result<(), String> {
        let new_profile = Self::read_profile()?;
        
        if new_profile == self.current {
            self.stable_count += 1;
        } else {
            self.stable_count = 0;
            self.current = new_profile;
        }
        
        Ok(())
    }
    
    fn text(&self) -> String {
        match self.current.as_str() {
            "gaming" => "󰊴".to_string(),
            "work" => "󰄄".to_string(),
            "low-power" => "󰾆".to_string(),
            _ => "󰀻".to_string(),
        }
    }
    
    fn color(&self) -> [u8; 4] {
        match self.current.as_str() {
            "gaming" => [0xa3, 0xe3, 0x6b, 0xFF],   // Accent green
            "work" => [0xff, 0xc8, 0x5c, 0xFF],     // Blue
            "low-power" => [0x77, 0xc1, 0xf5, 0xFF], // Amber
            _ => [0xa3, 0xe3, 0x6b, 0xFF],          // Default accent
        }
    }
    
    fn on_click(&mut self, button: u32) -> Option<BlockAction> {
        if button == 1 { // Left click
            let next = match self.current.as_str() {
                "default" => "gaming",
                "gaming" => "work",
                "work" => "low-power",
                _ => "default",
            };
            Some(BlockAction::Command(format!("profile {}", next)))
        } else {
            None
        }
    }
    
    fn interval(&self) -> u64 {
        // Adaptive refresh: slow down if stable
        if self.stable_count > 5 {
            10000 // 10 seconds if very stable
        } else if self.stable_count > 2 {
            5000  // 5 seconds if somewhat stable  
        } else {
            1000  // 1 second if changing
        }
    }
    
    fn min_width(&self) -> i32 { 40 }
    
    fn align(&self) -> BlockAlign { BlockAlign::Left }
}
