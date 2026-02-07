#![allow(dead_code)]
//! Lock status block - shows if 0-Core is locked/unlocked
use super::{Block, BlockAction, BlockAlign};

const ICON_LOCKED: &str = "󰌾";
const ICON_UNLOCKED: &str = "󰌿";

pub struct LockBlock {
    #[allow(dead_code)]
    locked: bool,
}

impl LockBlock {
    pub fn new() -> Self {
        Self {
            locked: false,
        }
    }
    
    #[allow(dead_code)]
    fn is_locked() -> Result<bool, String> {
        let path = crate::paths::core_lock_path();
        Ok(path.exists())
    }
}

impl Block for LockBlock {
    fn name(&self) -> &str { "lock" }
    
    fn update(&mut self) -> Result<(), String> {
        self.locked = Self::is_locked()?;
        Ok(())
    }
    
    fn text(&self) -> String {
        if self.locked {
            ICON_LOCKED.to_string()
        } else {
            ICON_UNLOCKED.to_string()
        }
    }
    
    fn color(&self) -> [u8; 4] {
        if self.locked {
            [0xa3, 0xe3, 0x6b, 0xFF] // Accent green when locked
        } else {
            [0x77, 0xc1, 0xf5, 0xFF] // Amber when unlocked
        }
    }
    
    fn on_click(&mut self, button: u32) -> Option<BlockAction> {
        if button == 1 { // Left click toggles lock
            let cmd = if self.locked {
                "unlock-core"
            } else {
                "lock-core"
            };
            Some(BlockAction::Command(cmd.to_string()))
        } else {
            None
        }
    }
    
    fn interval(&self) -> u64 {
        1000 // Check every second (lock state can change quickly)
    }
    
    fn min_width(&self) -> i32 { 25 }
    
    fn align(&self) -> BlockAlign { BlockAlign::Left }
}
