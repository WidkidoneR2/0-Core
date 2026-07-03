//! faelight-core v1.0.0 - Shared Foundation Library
//!
//! Provides common functionality for all Faelight tools:
//! - Glyph caching (70%+ CPU reduction)
//! - Canvas drawing primitives
//! - Theme system (consistent styling)
//! - Wayland helpers (layer-shell configs)
//! - Error handling

pub mod canvas;
pub mod error;
pub mod glyph;
pub mod paths;
pub mod theme;
pub mod wayland;

pub use canvas::Canvas;
pub use error::{FaelightError, Result};
pub use glyph::GlyphCache;
pub use theme::Theme;
pub use wayland::{Anchor, Layer, LayerSurfaceConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_cache_basic() {
        // INT-106: load the font at runtime and skip gracefully if absent (e.g. the
        // Nix build sandbox has no system fonts). Previously include_bytes! with an
        // absolute path hard-failed the test build. GlyphCache::new takes font bytes,
        // so production is unaffected -- only this test referenced a system path.
        let font_path = "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf";
        let font_data = match std::fs::read(font_path) {
            Ok(d) => d,
            Err(_) => return, // font not available in this environment -- skip
        };
        let mut cache = GlyphCache::new(&font_data).unwrap();

        let glyph1 = cache.rasterize('A', 16.0);
        assert!(!glyph1.bitmap.is_empty());

        let glyph2 = cache.rasterize('A', 16.0);
        assert!(!glyph2.bitmap.is_empty());

        let (hits, misses, hit_rate) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(hit_rate, 50.0);
    }
}
