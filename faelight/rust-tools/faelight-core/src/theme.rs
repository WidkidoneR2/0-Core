//! Theme system for consistent styling across all tools
//! INT-033: semantic color tokens for domain-aware coloring

// ── Truecolor helpers (RGB) ─────────────────────────────────────────────────
// These produce ANSI truecolor escape sequences for terminals that support it.
// Used by fsh prompt, faelight-bar, faelight-fm, and all ratatui tools.

pub fn fc(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}
pub fn fc_bold(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[1m\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}
pub fn fc_dim(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[2m\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

// ── Neon Candy Palette ──────────────────────────────────────────────────────
// The canonical color values for the Faelight Forest aesthetic.
// All tools reference these constants -- never hardcode RGB values elsewhere.

// Greens
pub const NEON_GREEN: (u8, u8, u8) = (57, 255, 20); // primary forest green
pub const FOREST_GREEN: (u8, u8, u8) = (107, 227, 163); // softer green
pub const MUTED_GREEN: (u8, u8, u8) = (100, 180, 100); // dimmed green

// Cyans / Blues
pub const NEON_CYAN: (u8, u8, u8) = (50, 220, 255); // cwd, links, info
pub const SOFT_CYAN: (u8, u8, u8) = (100, 200, 220); // secondary info
pub const NEON_BLUE: (u8, u8, u8) = (80, 140, 255); // git ahead

// Purples
pub const NEON_PURPLE: (u8, u8, u8) = (180, 130, 255); // active intent, philosophy
pub const SOFT_PURPLE: (u8, u8, u8) = (160, 120, 220); // planned intent
pub const MUTED_PURPLE: (u8, u8, u8) = (130, 100, 180); // dimmed purple

// Ambers / Yellows
pub const NEON_AMBER: (u8, u8, u8) = (255, 200, 50); // git dirty, warning
pub const SOFT_AMBER: (u8, u8, u8) = (220, 170, 80); // advisory

// Reds
pub const NEON_RED: (u8, u8, u8) = (255, 80, 80); // error, blocked, danger
pub const SOFT_RED: (u8, u8, u8) = (220, 100, 100); // soft error

// Whites / Grays
pub const FOG_WHITE: (u8, u8, u8) = (215, 224, 218); // primary text
pub const MUTED_GRAY: (u8, u8, u8) = (120, 140, 130); // dimmed text

// ── Semantic Color Tokens ───────────────────────────────────────────────────
// Map domain concepts to palette colors.
// Use these in all tools -- not the raw palette constants above.

// Intent status colors
pub const COLOR_INTENT_ACTIVE: (u8, u8, u8) = NEON_GREEN;
pub const COLOR_INTENT_PLANNED: (u8, u8, u8) = SOFT_PURPLE;
pub const COLOR_INTENT_COMPLETE: (u8, u8, u8) = MUTED_GREEN;
pub const COLOR_INTENT_BLOCKED: (u8, u8, u8) = NEON_RED;
pub const COLOR_INTENT_RESEARCH: (u8, u8, u8) = NEON_CYAN;
pub const COLOR_INTENT_EXPERIMENT: (u8, u8, u8) = NEON_PURPLE;

// Git state colors
pub const COLOR_GIT_CLEAN: (u8, u8, u8) = NEON_GREEN;
pub const COLOR_GIT_DIRTY: (u8, u8, u8) = NEON_AMBER;
pub const COLOR_GIT_AHEAD: (u8, u8, u8) = NEON_BLUE;
pub const COLOR_GIT_BEHIND: (u8, u8, u8) = SOFT_AMBER;
pub const COLOR_GIT_EXPERIMENTAL: (u8, u8, u8) = NEON_PURPLE;

// Health colors
pub const COLOR_HEALTH_PEAK: (u8, u8, u8) = NEON_GREEN;
pub const COLOR_HEALTH_ADVISORY: (u8, u8, u8) = NEON_AMBER;
pub const COLOR_HEALTH_CRITICAL: (u8, u8, u8) = NEON_RED;

// Prompt colors
pub const COLOR_PROMPT_CWD: (u8, u8, u8) = NEON_CYAN;
pub const COLOR_PROMPT_OK: (u8, u8, u8) = NEON_GREEN;
pub const COLOR_PROMPT_FAIL: (u8, u8, u8) = NEON_RED;
pub const COLOR_PROMPT_INTENT: (u8, u8, u8) = NEON_PURPLE;
pub const COLOR_PROMPT_BRANCH: (u8, u8, u8) = NEON_AMBER;

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
            bg_primary: 0x0f1411,   // Forest Night
            bg_secondary: 0x1a1f1c, // Darker forest
            bg_tertiary: 0x252b28,  // Lighter forest

            // Text - Fog whites and greens
            text_primary: 0xd7e0da,   // Fog White
            text_secondary: 0xa8b5af, // Muted fog
            text_muted: 0x6b7973,     // Very muted

            // Accents - Neon cyan and sunset orange
            accent: 0x6be3a3,       // Faelight Green
            accent_hover: 0x5cc8ff, // Faelight Blue
            danger: 0xff6b6b,       // Soft red
            warning: 0xf5c177,      // Amber Leaf
            success: 0x6be3a3,      // Faelight Green

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
