//! faelight-core v0.1.0 - Shared Foundation Library
//! 
//! Provides common functionality for all Faelight tools:
//! - Glyph caching (70%+ CPU reduction)
//! - Error handling
//! - Future: Canvas, Theme, Wayland helpers

pub mod glyph;
pub mod error;

pub use glyph::GlyphCache;
pub use error::{FaelightError, Result};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glyph_cache_basic() {
        // Use a simple embedded font for testing
        let font_data = include_bytes!("/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf");
        let mut cache = GlyphCache::new(font_data).unwrap();
        
        // First rasterization (cache miss)
        let glyph1 = cache.rasterize('A', 16.0);
        assert!(glyph1.bitmap.len() > 0);
        
        // Second rasterization (cache hit)
        let glyph2 = cache.rasterize('A', 16.0);
        assert!(glyph2.bitmap.len() > 0);
        
        // Check stats
        let (hits, misses, hit_rate) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(hit_rate, 50.0);
    }
}
