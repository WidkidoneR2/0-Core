//! Menu mode rendering - compact dropdown near launcher icon

use crate::state::MenuState;
use faelight_core::GlyphCache;

const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const ACCENT_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const SELECTED_COLOR: [u8; 4] = [0x77, 0xc1, 0xf5, 0xFF];
const DIM_COLOR: [u8; 4] = [0x77, 0x7f, 0x6f, 0xFF];
const MENU_BG: [u8; 4] = [0x1a, 0x1e, 0x1a, 0xFF];  // Slightly lighter than main BG
const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Regular.ttf");

// Dropdown dimensions
const DROPDOWN_WIDTH: u32 = 400;
const DROPDOWN_X: i32 = 80;  // Start near launcher icon

lazy_static::lazy_static! {
    static ref GLYPH_CACHE: std::sync::Mutex<GlyphCache> = {
        std::sync::Mutex::new(
            GlyphCache::new(FONT_DATA).expect("Failed to load font")
        )
    };
}

pub fn render(menu: &MenuState, canvas: &mut [u8], width: u32, _height: u32, y_offset: i32) {
    let mut cache = GLYPH_CACHE.lock().unwrap();
    
    // Draw dropdown background box
    let box_height = 100;
    for y in y_offset..(y_offset + box_height) {
        for x in DROPDOWN_X..(DROPDOWN_X + DROPDOWN_WIDTH as i32) {
            if x >= 0 && x < width as i32 && y >= 0 {
                let idx = (y as usize * width as usize + x as usize) * 4;
                if idx + 3 < canvas.len() {
                    canvas[idx] = MENU_BG[0];
                    canvas[idx + 1] = MENU_BG[1];
                    canvas[idx + 2] = MENU_BG[2];
                    canvas[idx + 3] = MENU_BG[3];
                }
            }
        }
    }
    
    // Draw border
    draw_box_border(canvas, width, DROPDOWN_X, y_offset, DROPDOWN_WIDTH, box_height as u32, ACCENT_COLOR);
    
    // Input line with cursor
    let prompt = format!("> {}_", menu.input);
    draw_text(&mut cache, canvas, width, &prompt, DROPDOWN_X + 10, y_offset + 4, ACCENT_COLOR);
    
    // Results count (right side of dropdown)
    let count_text = format!("[{}]", menu.filtered.len());
    draw_text(&mut cache, canvas, width, &count_text, DROPDOWN_X + DROPDOWN_WIDTH as i32 - 50, y_offset + 4, DIM_COLOR);
    
    // Separator line
    let separator_y = y_offset + 22;
    for x in (DROPDOWN_X + 10)..(DROPDOWN_X + DROPDOWN_WIDTH as i32 - 10) {
        let idx = (separator_y as usize * width as usize + x as usize) * 4;
        if idx + 3 < canvas.len() {
            canvas[idx] = DIM_COLOR[0];
            canvas[idx + 1] = DIM_COLOR[1];
            canvas[idx + 2] = DIM_COLOR[2];
            canvas[idx + 3] = DIM_COLOR[3];
        }
    }
    
    // Results - show only 3-4 items
    let start_y = y_offset + 28;
    let items_to_show = menu.filtered.len().min(4);
    
    for (i, &idx) in menu.filtered.iter().take(items_to_show).enumerate() {
        let y = start_y + (i as i32 * 16);
        let color = if i == menu.selected {
            SELECTED_COLOR
        } else {
            TEXT_COLOR
        };
        
        let marker = if i == menu.selected { "▶" } else { " " };
        let item = &menu.items[idx];
        
        // Truncate to fit dropdown width
        let display_item = if item.display.len() > 45 {
            format!("{}...", &item.display[..42])
        } else {
            item.display.clone()
        };
        
        let text = format!("{} {}", marker, display_item);
        draw_text(&mut cache, canvas, width, &text, DROPDOWN_X + 10, y, color);
    }
    
    // Scroll indicator if more items
    if menu.filtered.len() > items_to_show {
        let more_text = format!("... {} more", menu.filtered.len() - items_to_show);
        draw_text(&mut cache, canvas, width, &more_text, DROPDOWN_X + 10, 
                 start_y + (items_to_show as i32 * 16) + 2, DIM_COLOR);
    }
}

fn draw_box_border(canvas: &mut [u8], width: u32, x: i32, y: i32, w: u32, h: u32, color: [u8; 4]) {
    // Top line
    for px in x..(x + w as i32) {
        if let Some(idx) = get_pixel_index(canvas, width, px, y) {
            canvas[idx..idx+4].copy_from_slice(&color);
        }
    }
    
    // Bottom line
    for px in x..(x + w as i32) {
        if let Some(idx) = get_pixel_index(canvas, width, px, y + h as i32 - 1) {
            canvas[idx..idx+4].copy_from_slice(&color);
        }
    }
    
    // Left line
    for py in y..(y + h as i32) {
        if let Some(idx) = get_pixel_index(canvas, width, x, py) {
            canvas[idx..idx+4].copy_from_slice(&color);
        }
    }
    
    // Right line
    for py in y..(y + h as i32) {
        if let Some(idx) = get_pixel_index(canvas, width, x + w as i32 - 1, py) {
            canvas[idx..idx+4].copy_from_slice(&color);
        }
    }
}

fn get_pixel_index(canvas: &[u8], width: u32, x: i32, y: i32) -> Option<usize> {
    if x >= 0 && x < width as i32 && y >= 0 && y < 132 {
        let idx = (y as usize * width as usize + x as usize) * 4;
        if idx + 3 < canvas.len() {
            return Some(idx);
        }
    }
    None
}

fn draw_text(cache: &mut GlyphCache, canvas: &mut [u8], width: u32, 
             text: &str, x: i32, y: i32, color: [u8; 4]) {
    let mut cursor_x = x;
    let font_size = 14.0;
    let baseline = y + 12;
    
    for ch in text.chars() {
        let glyph = cache.rasterize(ch, font_size);
        let metrics = &glyph.metrics;
        let bitmap = &glyph.bitmap;
        
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha == 0 { continue; }
                
                let px = cursor_x + metrics.xmin as i32 + col as i32;
                let py = baseline - metrics.height as i32 - metrics.ymin as i32 + row as i32;
                
                if px >= 0 && px < width as i32 && py >= 0 && py < 132 {
                    let idx = (py as usize * width as usize + px as usize) * 4;
                    if idx + 3 < canvas.len() {
                        let a = alpha as f32 / 255.0;
                        canvas[idx] = ((1.0 - a) * canvas[idx] as f32 + a * color[0] as f32) as u8;
                        canvas[idx + 1] = ((1.0 - a) * canvas[idx + 1] as f32 + a * color[1] as f32) as u8;
                        canvas[idx + 2] = ((1.0 - a) * canvas[idx + 2] as f32 + a * color[2] as f32) as u8;
                        canvas[idx + 3] = 255;
                    }
                }
            }
        }
        cursor_x += metrics.advance_width as i32;
    }
}
