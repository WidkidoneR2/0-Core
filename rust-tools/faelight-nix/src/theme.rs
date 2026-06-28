//! theme.rs -- candy-neon forest palette (lifted from faelight-fm ui/mod.rs).
//! Single source of color truth for faelight-nix so the tool matches the forest.

use ratatui::style::Color;

pub const GREEN:     Color = Color::Rgb(57,  255, 20);  // neon lime  -- active, selection
pub const CYAN:      Color = Color::Rgb(50,  220, 255); // neon cyan  -- focus, keys, labels
pub const YELLOW:    Color = Color::Rgb(255, 200, 50);  // amber      -- version, warnings
pub const MAGENTA:   Color = Color::Rgb(180, 130, 255); // purple     -- accents
pub const GRAY:      Color = Color::Rgb(120, 140, 130); // muted gray -- secondary, hints
pub const WHITE:     Color = Color::Rgb(215, 224, 218); // fog white  -- primary text
pub const BG_SEL:    Color = Color::Rgb(22,  35,  25);  // forest night -- selection bg
pub const BG:        Color = Color::Rgb(8,   13,  8);   // deep forest black -- app bg
