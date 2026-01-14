//! Wayland layer-shell helpers for consistent surface configuration

/// Layer for layer-shell surfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Bottom,
    Top,
    Overlay,
}

/// Anchor points for layer surfaces
#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl Anchor {
    pub const TOP: Self = Self { top: true, right: false, bottom: false, left: false };
    pub const BOTTOM: Self = Self { top: false, right: false, bottom: true, left: false };
    pub const LEFT: Self = Self { top: false, right: false, bottom: false, left: true };
    pub const RIGHT: Self = Self { top: false, right: true, bottom: false, left: false };
    
    pub const TOP_LEFT: Self = Self { top: true, right: false, bottom: false, left: true };
    pub const TOP_RIGHT: Self = Self { top: true, right: true, bottom: false, left: false };
    pub const BOTTOM_LEFT: Self = Self { top: false, right: false, bottom: true, left: true };
    pub const BOTTOM_RIGHT: Self = Self { top: false, right: true, bottom: true, left: false };
    
    pub const ALL: Self = Self { top: true, right: true, bottom: true, left: true };
}

/// Configuration for creating a layer surface
#[derive(Debug, Clone)]
pub struct LayerSurfaceConfig {
    /// Width in pixels (0 for compositor-decided)
    pub width: u32,
    
    /// Height in pixels (0 for compositor-decided)
    pub height: u32,
    
    /// Where to anchor the surface
    pub anchor: Anchor,
    
    /// Which layer to place surface on
    pub layer: Layer,
    
    /// Margins (top, right, bottom, left)
    pub margin: (i32, i32, i32, i32),
    
    /// Exclusive zone (-1 for entire surface, 0 for none)
    pub exclusive_zone: i32,
    
    /// Whether surface should receive keyboard input
    pub keyboard_interactivity: bool,
}

impl LayerSurfaceConfig {
    /// Config for a top bar (like faelight-bar)
    pub fn top_bar(height: u32) -> Self {
        Self {
            width: 0,  // Full width
            height,
            anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            layer: Layer::Top,
            margin: (0, 0, 0, 0),
            exclusive_zone: height as i32,
            keyboard_interactivity: false,
        }
    }
    
    /// Config for a centered popup (like faelight-menu)
    pub fn centered_popup(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            anchor: Anchor::ALL,  // Center
            layer: Layer::Overlay,
            margin: (0, 0, 0, 0),
            exclusive_zone: 0,
            keyboard_interactivity: true,
        }
    }
    
    /// Config for a notification (top-right)
    pub fn notification(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            anchor: Anchor::TOP_RIGHT,
            layer: Layer::Overlay,
            margin: (8, 8, 0, 0),
            exclusive_zone: 0,
            keyboard_interactivity: false,
        }
    }
    
    /// Config for a launcher (centered, fullscreen)
    pub fn launcher() -> Self {
        Self {
            width: 0,
            height: 0,
            anchor: Anchor::ALL,
            layer: Layer::Overlay,
            margin: (0, 0, 0, 0),
            exclusive_zone: 0,
            keyboard_interactivity: true,
        }
    }
}

impl std::ops::BitOr for Anchor {
    type Output = Self;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            top: self.top || rhs.top,
            right: self.right || rhs.right,
            bottom: self.bottom || rhs.bottom,
            left: self.left || rhs.left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_anchor_combinations() {
        let top_left = Anchor::TOP | Anchor::LEFT;
        assert!(top_left.top);
        assert!(top_left.left);
        assert!(!top_left.right);
        assert!(!top_left.bottom);
    }
    
    #[test]
    fn test_layer_surface_presets() {
        let bar = LayerSurfaceConfig::top_bar(32);
        assert_eq!(bar.height, 32);
        assert_eq!(bar.exclusive_zone, 32);
        
        let popup = LayerSurfaceConfig::centered_popup(400, 300);
        assert_eq!(popup.width, 400);
        assert!(popup.keyboard_interactivity);
        
        let notif = LayerSurfaceConfig::notification(300, 100);
        assert!(notif.anchor.top);
        assert!(notif.anchor.right);
    }
}
