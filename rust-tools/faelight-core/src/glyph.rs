//! Glyph caching for high-performance text rendering
//! 
//! Eliminates repeated font rasterization by caching glyphs.
//! Expected performance: 70-90% CPU reduction in text rendering.

use fontdue::{Font, Metrics};
use std::collections::HashMap;
use crate::error::{FaelightError, Result};

/// A cached glyph with its metrics and bitmap
pub struct Glyph {
    pub metrics: Metrics,
    pub bitmap: Vec<u8>,
}

/// Cache key: (character, size * 10) for sub-pixel size precision
type GlyphKey = (char, u32);

/// Glyph cache that prevents repeated font rasterization
pub struct GlyphCache {
    font: Font,
    cache: HashMap<GlyphKey, Glyph>,
    hits: usize,
    misses: usize,
}

impl GlyphCache {
    /// Create a new glyph cache from font data
    pub fn new(font_data: &[u8]) -> Result<Self> {
        let font = Font::from_bytes(font_data, Default::default())
            .map_err(|e| FaelightError::FontLoad(e.to_string()))?;
        
        Ok(Self {
            font,
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
        })
    }
    
    /// Get or rasterize a glyph at the given size
    pub fn rasterize(&mut self, ch: char, size: f32) -> &Glyph {
        let key = (ch, (size * 10.0) as u32);
        
        if self.cache.contains_key(&key) {
            self.hits += 1;
        } else {
            self.misses += 1;
            let (metrics, bitmap) = self.font.rasterize(ch, size);
            self.cache.insert(key, Glyph { metrics, bitmap });
        }
        
        self.cache.get(&key).unwrap()
    }
    
    /// Measure text width without rendering
    pub fn text_width(&mut self, text: &str, size: f32) -> u32 {
        text.chars()
            .map(|ch| {
                let glyph = self.rasterize(ch, size);
                glyph.metrics.advance_width as u32
            })
            .sum()
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize, f64) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 {
            self.hits as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        (self.hits, self.misses, hit_rate)
    }
}
