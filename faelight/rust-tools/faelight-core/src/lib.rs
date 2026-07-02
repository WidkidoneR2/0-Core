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
        let font_data = include_bytes!("/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf");
        let mut cache = GlyphCache::new(font_data).unwrap();

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
