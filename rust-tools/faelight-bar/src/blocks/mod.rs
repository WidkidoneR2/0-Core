#![allow(dead_code)]
//! Block system - modular status indicators
use std::time::Instant;

/// Core block trait - all status indicators implement this
pub trait Block: Send {
    /// Unique block identifier
    fn name(&self) -> &str;
    
    /// Update block data
    fn update(&mut self) -> Result<(), String>;
    
    /// Get display text (with NerdFont icons)
    fn text(&self) -> String;
    
    /// Get text color [r, g, b, a]
    fn color(&self) -> [u8; 4];
    
    /// Handle click events
    /// Returns: Some(action) if this click should trigger something
    fn on_click(&mut self, button: u32) -> Option<BlockAction>;
    
    /// Update interval in milliseconds (adaptive - can change!)
    fn interval(&self) -> u64 { 1000 }
    
    /// Minimum width in pixels
    fn min_width(&self) -> i32 { 0 }
    
    /// Should show separator after this block?
    fn separator(&self) -> bool { true }
    
    /// Should this block be visible right now? (context-aware!)
    fn visible(&self) -> bool { true }
    
    /// Alignment
    fn align(&self) -> BlockAlign { BlockAlign::Left }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub enum BlockAction {
    None,
    Command(String),
    ShowMenu,
    Refresh,
    ToggleVisibility(String), // Toggle another block's visibility
}

/// Block manager - handles all blocks with error isolation
pub struct BlockManager {
    blocks: Vec<Box<dyn Block>>,
    last_update: Vec<Instant>,
    last_value: Vec<String>, // For adaptive refresh
}

impl BlockManager {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            last_update: Vec::new(),
            last_value: Vec::new(),
        }
    }
    
    pub fn add_block(&mut self, block: Box<dyn Block>) {
        self.last_update.push(Instant::now() - std::time::Duration::from_secs(9999));
        self.last_value.push(String::new());
        self.blocks.push(block);
    }
    
    /// Update all blocks (with adaptive refresh!)
    pub fn update_all(&mut self) {
        let now = Instant::now();
        
        for (i, block) in self.blocks.iter_mut().enumerate() {
            let last = self.last_update.get(i).copied().unwrap();
            let interval = block.interval();
            
            // Adaptive refresh: skip if not time yet
            if now.duration_since(last).as_millis() < interval as u128 {
                continue;
            }
            
            // Isolated error handling - block failure doesn't crash bar
            match block.update() {
                Ok(_) => {
                    let new_value = block.text();
                    
                    // Adaptive refresh: slow down if value unchanged
                    if let Some(old_value) = self.last_value.get(i) {
                        if new_value == *old_value {
                            // Value stable, can slow down updates
                            // (blocks handle this in their interval() method)
                        }
                    }
                    
                    self.last_value[i] = new_value;
                    self.last_update[i] = now;
                }
                Err(e) => {
                    eprintln!("[faelight-bar] Block '{}' error: {}", block.name(), e);
                    // Block continues to show last good value
                }
            }
        }
    }
    
    pub fn get(&self, name: &str) -> Option<&dyn Block> {
        self.blocks.iter()
            .find(|b| b.name() == name)
            .map(|b| &**b)
    }
    
    pub fn left_blocks(&self) -> impl Iterator<Item = &Box<dyn Block>> {
        self.blocks.iter()
            .filter(|b| b.align() == BlockAlign::Left && b.visible())
    }
    
    pub fn center_blocks(&self) -> impl Iterator<Item = &Box<dyn Block>> {
        self.blocks.iter()
            .filter(|b| b.align() == BlockAlign::Center && b.visible())
    }
    
    pub fn right_blocks(&self) -> impl Iterator<Item = &Box<dyn Block>> {
        self.blocks.iter()
            .filter(|b| b.align() == BlockAlign::Right && b.visible())
    }
    
    pub fn handle_click(&mut self, name: &str, button: u32) -> Option<BlockAction> {
        self.blocks.iter_mut()
            .find(|b| b.name() == name)
            .and_then(|b| b.on_click(button))
    }
}

// Individual block modules
pub mod profile;
pub mod time;
pub mod lock;
// More blocks will be added here
