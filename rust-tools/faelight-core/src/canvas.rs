//! Canvas abstraction for Wayland rendering
//! 
//! Provides a simple drawing surface with SHM buffer management.

use crate::glyph::GlyphCache;

/// RGB color as u32 (0xRRGGBB)
pub type Color = u32;

/// A drawing canvas backed by a pixel buffer
pub struct Canvas {
    buffer: Vec<u32>,
    width: u32,
    height: u32,
}

impl Canvas {
    /// Create a new canvas with given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            buffer: vec![0; size],
            width,
            height,
        }
    }
    
    /// Get the raw buffer (for SHM)
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.buffer.as_ptr() as *const u8,
                self.buffer.len() * 4,
            )
        }
    }
    
    /// Get buffer size in bytes
    pub fn byte_size(&self) -> usize {
        self.buffer.len() * 4
    }
    
    /// Get dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Clear the canvas with a color
    pub fn clear(&mut self, color: Color) {
        self.buffer.fill(color);
    }
    
    /// Draw a filled rectangle
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for py in y..(y + h).min(self.height) {
            for px in x..(x + w).min(self.width) {
                let idx = (py * self.width + px) as usize;
                self.buffer[idx] = color;
            }
        }
    }
    
    /// Draw text at position (returns width drawn)
    pub fn draw_text(
        &mut self,
        cache: &mut GlyphCache,
        text: &str,
        x: u32,
        y: u32,
        size: f32,
        color: Color,
    ) -> u32 {
        let mut cursor_x = x;
        
        for ch in text.chars() {
            let glyph = cache.rasterize(ch, size);
            let glyph_width = glyph.metrics.width;
            let glyph_height = glyph.metrics.height;
            
            // Draw glyph bitmap
            for gy in 0..glyph_height {
                for gx in 0..glyph_width {
                    let bitmap_idx = (gy * glyph_width + gx) as usize;
                    let alpha = glyph.bitmap[bitmap_idx];
                    
                    if alpha > 0 {
                        let px = cursor_x + gx as u32;
                        let py = y + gy as u32;
                        
                        if px < self.width && py < self.height {
                            let canvas_idx = (py * self.width + px) as usize;
                            
                            // Simple alpha blending
                            if alpha == 255 {
                                self.buffer[canvas_idx] = color;
                            } else {
                                let bg = self.buffer[canvas_idx];
                                let blended = blend_color(bg, color, alpha);
                                self.buffer[canvas_idx] = blended;
                            }
                        }
                    }
                }
            }
            
            cursor_x += glyph.metrics.advance_width as u32;
        }
        
        cursor_x - x
    }
    
    /// Resize the canvas
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.buffer.resize((width * height) as usize, 0);
    }
}

/// Blend foreground color onto background with alpha
fn blend_color(bg: Color, fg: Color, alpha: u8) -> Color {
    let alpha = alpha as u32;
    let inv_alpha = 255 - alpha;
    
    let bg_r = (bg >> 16) & 0xFF;
    let bg_g = (bg >> 8) & 0xFF;
    let bg_b = bg & 0xFF;
    
    let fg_r = (fg >> 16) & 0xFF;
    let fg_g = (fg >> 8) & 0xFF;
    let fg_b = fg & 0xFF;
    
    let r = (fg_r * alpha + bg_r * inv_alpha) / 255;
    let g = (fg_g * alpha + bg_g * inv_alpha) / 255;
    let b = (fg_b * alpha + bg_b * inv_alpha) / 255;
    
    (r << 16) | (g << 8) | b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new(100, 50);
        assert_eq!(canvas.dimensions(), (100, 50));
        assert_eq!(canvas.byte_size(), 100 * 50 * 4);
    }
    
    #[test]
    fn test_canvas_clear() {
        let mut canvas = Canvas::new(100, 50);
        canvas.clear(0xFF0000); // Red
        
        // Check a few pixels
        assert_eq!(canvas.buffer[0], 0xFF0000);
        assert_eq!(canvas.buffer[100], 0xFF0000);
    }
    
    #[test]
    fn test_canvas_rect() {
        let mut canvas = Canvas::new(100, 50);
        canvas.clear(0x000000); // Black
        canvas.draw_rect(10, 10, 20, 15, 0xFFFFFF); // White rect
        
        // Inside rect should be white
        let idx = (15 * 100 + 15) as usize;
        assert_eq!(canvas.buffer[idx], 0xFFFFFF);
        
        // Outside rect should be black
        assert_eq!(canvas.buffer[0], 0x000000);
    }
}
