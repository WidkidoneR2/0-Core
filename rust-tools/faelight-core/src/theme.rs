//! Theme system for consistent styling across all tools

/// Theme configuration for Faelight tools
#[derive(Debug, Clone)]
pub struct Theme {
    // Background colors
    pub bg_primary: u32,
    pub bg_secondary: u32,
    pub bg_tertiary: u32,
    
    // Text colors
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_muted: u32,
    
    // Accent colors
    pub accent: u32,
    pub accent_hover: u32,
    pub danger: u32,
    pub warning: u32,
    pub success: u32,
    
    // Spacing
    pub padding: u32,
    pub gap: u32,
    pub border_width: u32,
    
    // Typography
    pub font_size_small: f32,
    pub font_size_normal: f32,
    pub font_size_large: f32,
}

impl Theme {
    /// Faelight Forest default theme (tropical sunset colors)
    pub fn faelight_default() -> Self {
        Self {
            // Backgrounds - Deep ocean blues
            bg_primary: 0x0f1411,     // Forest Night
            bg_secondary: 0x1a1f1c,   // Darker forest
            bg_tertiary: 0x252b28,    // Lighter forest
            
            // Text - Fog whites and greens
            text_primary: 0xd7e0da,   // Fog White
            text_secondary: 0xa8b5af, // Muted fog
            text_muted: 0x6b7973,     // Very muted
            
            // Accents - Neon cyan and sunset orange
            accent: 0x6be3a3,         // Faelight Green
            accent_hover: 0x5cc8ff,   // Faelight Blue
            danger: 0xff6b6b,         // Soft red
            warning: 0xf5c177,        // Amber Leaf
            success: 0x6be3a3,        // Faelight Green
            
            // Spacing
            padding: 8,
            gap: 8,
            border_width: 2,
            
            // Typography
            font_size_small: 11.0,
            font_size_normal: 14.0,
            font_size_large: 18.0,
        }
    }
    
    /// Dark variant (even darker backgrounds)
    pub fn faelight_dark() -> Self {
        let mut theme = Self::faelight_default();
        theme.bg_primary = 0x0a0d0b;
        theme.bg_secondary = 0x0f1411;
        theme.bg_tertiary = 0x1a1f1c;
        theme
    }
    
    /// Light variant (for daytime use)
    pub fn faelight_light() -> Self {
        Self {
            bg_primary: 0xf5f7f6,
            bg_secondary: 0xe8ede9,
            bg_tertiary: 0xd7e0da,
            
            text_primary: 0x1a1f1c,
            text_secondary: 0x3a4540,
            text_muted: 0x6b7973,
            
            accent: 0x4ac88f,
            accent_hover: 0x3ba8df,
            danger: 0xd94848,
            warning: 0xd9a247,
            success: 0x4ac88f,
            
            padding: 8,
            gap: 8,
            border_width: 2,
            
            font_size_small: 11.0,
            font_size_normal: 14.0,
            font_size_large: 18.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::faelight_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_creation() {
        let theme = Theme::faelight_default();
        assert_eq!(theme.bg_primary, 0x0f1411);
        assert_eq!(theme.accent, 0x6be3a3);
        assert_eq!(theme.padding, 8);
    }
    
    #[test]
    fn test_theme_variants() {
        let default_theme = Theme::faelight_default();
        let dark = Theme::faelight_dark();
        let light = Theme::faelight_light();
        
        // Dark should be darker than default
        assert!(dark.bg_primary < default_theme.bg_primary);
        
        // Light should be lighter than default
        assert!(light.bg_primary > default_theme.bg_primary);
    }
}
