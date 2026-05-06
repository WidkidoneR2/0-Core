//! faelight-bar render - clean rebuild v5.0.0

use chrono::Local;
use faelight_core::GlyphCache;
use rusqlite;
use std::fs;
use std::process::Command;

// Color palette - matches faelight-fm/palette
const BG: [u8; 4] = [0x11, 0x14, 0x0f, 0xFF];
const GREEN: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const AMBER: [u8; 4] = [0xff, 0xaa, 0x00, 0xFF];
const RED: [u8; 4] = [0xff, 0x6b, 0x6b, 0xFF];
const DIM: [u8; 4] = [0x55, 0x60, 0x50, 0xFF];

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Regular.ttf");
const FONT_SIZE_BASE: f32 = 13.5;

use std::cell::Cell;
thread_local! {
    static RENDER_SCALE: Cell<f32> = const { Cell::new(1.0) };
}

fn set_render_scale(scale: f32) {
    RENDER_SCALE.with(|s| s.set(scale));
}

fn current_font_size() -> f32 {
    RENDER_SCALE.with(|s| FONT_SIZE_BASE * s.get())
}

lazy_static::lazy_static! {
    static ref HEALTH_CACHE: std::sync::Mutex<(String, [u8; 4], std::time::Instant)> =
        std::sync::Mutex::new((
            "HP:??".to_string(),
            DIM,
            std::time::Instant::now() - std::time::Duration::from_secs(60),
        ));
    static ref GLYPH_CACHE: std::sync::Mutex<GlyphCache> = {
        std::sync::Mutex::new(GlyphCache::new(FONT_DATA).expect("Failed to load font"))
    };
}

// ─── Data gathering ──────────────────────────────────────────────────────────

fn get_lock() -> (&'static str, [u8; 4]) {
    // INT-251b: read authoritative lock state from runtime/.core-locked.
    // Written by core-protect on lock/unlock. Faster than lsattr subprocess
    // and survives our INT-251 chattr-skip-runtime/ refactor.
    let locked = faelight_core::paths::core_dir()
        .join("runtime")
        .join(".core-locked")
        .exists();
    if locked {
        ("\u{F033E}", GREEN) //  nerd font lock icon - green = protected
    } else {
        ("\u{F033F}", AMBER) //  nerd font unlock icon - amber = working
    }
}

fn get_wifi() -> (String, [u8; 4]) {
    if let Ok(out) = Command::new("nmcli")
        .args(["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
    {
        if let Ok(result) = String::from_utf8(out.stdout) {
            for line in result.lines() {
                if line.starts_with("yes:") {
                    let ssid = line.trim_start_matches("yes:").trim();
                    let label = if ssid.is_empty() {
                        "ON".to_string()
                    } else {
                        // truncate long SSIDs
                        if ssid.len() > 10 {
                            format!("{}…", &ssid[..9])
                        } else {
                            ssid.to_string()
                        }
                    };
                    return (format!("W:{}", label), GREEN);
                }
            }
        }
    }
    ("W:OFF".to_string(), RED)
}

pub fn get_active_intent() -> Option<(u32, String)> {
    let home = std::env::var("HOME").unwrap_or_default();
    let intents_dir = std::path::PathBuf::from(&home).join("0-core/intents/future");
    let entries = fs::read_dir(&intents_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&path).ok()?;
        if !content.contains("status: in-progress") {
            continue;
        }
        let id = content.lines()
            .find(|l| l.starts_with("id:"))
            .and_then(|l| l.trim_start_matches("id:").trim().parse::<u32>().ok())?;
        let title = content.lines()
            .find(|l| l.starts_with("title:"))
            .map(|l| {
                l.trim_start_matches("title:")
                    .trim()
                    .trim_matches('"')
                    .to_string()
            })?;
        let short = if title.len() > 32 {
            format!("{}…", &title[..32])
        } else {
            title
        };
        return Some((id, short));
    }
    None
}
pub fn get_friday_signal() -> Option<String> {
    let db_path = faelight_core::paths::core_dir()
        .join("runtime")
        .join("state.db");
    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .ok()?;
    let result: rusqlite::Result<(String, f64)> = conn.query_row(
        "SELECT friday_brief, brief_confidence FROM synthesis_snapshots          ORDER BY id DESC LIMIT 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );
    match result {
        Ok((brief, confidence)) if confidence >= 0.85 && !brief.is_empty() => {
            let short = if brief.len() > 38 {
                format!("{}…", &brief[..38])
            } else {
                brief
            };
            Some(short)
        }
        _ => None,
    }
}
// ─── Drawing ─────────────────────────────────────────────────────────────────

fn draw_text(
    cache: &mut GlyphCache,
    canvas: &mut [u8],
    width: u32,
    text: &str,
    x: i32,
    color: [u8; 4],
) -> i32 {
    let mut cx = x;
    let bar_h = (32.0 * current_font_size() / FONT_SIZE_BASE) as i32;
    let baseline = (bar_h as f32 * 0.68) as i32; // vertical center scaled

    for ch in text.chars() {
        let glyph = cache.rasterize(ch, current_font_size());
        let metrics = &glyph.metrics;
        let bitmap = &glyph.bitmap;

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha == 0 {
                    continue;
                }

                let px = cx + metrics.xmin + col as i32;
                let py = baseline - metrics.height as i32 - metrics.ymin + row as i32;

                if px >= 0 && px < width as i32 && py >= 0 && py < bar_h {
                    let idx = (py as usize * width as usize + px as usize) * 4;
                    if idx + 3 < canvas.len() {
                        let a = alpha as f32 / 255.0;
                        canvas[idx] = ((1.0 - a) * canvas[idx] as f32 + a * color[2] as f32) as u8;
                        canvas[idx + 1] =
                            ((1.0 - a) * canvas[idx + 1] as f32 + a * color[1] as f32) as u8;
                        canvas[idx + 2] =
                            ((1.0 - a) * canvas[idx + 2] as f32 + a * color[0] as f32) as u8;
                        canvas[idx + 3] = 255;
                    }
                }
            }
        }
        cx += metrics.advance_width as i32;
    }
    cx // return new x position
}

fn text_width(cache: &mut GlyphCache, text: &str) -> i32 {
    text.chars()
        .map(|ch| {
            cache
                .rasterize(ch, current_font_size())
                .metrics
                .advance_width as i32
        })
        .sum()
}

fn draw_separator(canvas: &mut [u8], width: u32, x: i32) {
    let stride = width as usize * 4;
    for y in 6..26usize {
        let fade = if y < 10 {
            (y - 6) as f32 / 4.0
        } else if y > 22 {
            (26 - y) as f32 / 4.0
        } else {
            1.0
        };
        if x >= 0 && x < width as i32 {
            let idx = y * stride + x as usize * 4;
            if idx + 3 < canvas.len() {
                canvas[idx] = ((1.0 - fade) * BG[2] as f32 + fade * DIM[2] as f32) as u8;
                canvas[idx + 1] = ((1.0 - fade) * BG[1] as f32 + fade * DIM[1] as f32) as u8;
                canvas[idx + 2] = ((1.0 - fade) * BG[0] as f32 + fade * DIM[0] as f32) as u8;
                canvas[idx + 3] = 255;
            }
        }
    }
}

fn draw_top_accent(canvas: &mut [u8], width: u32, color: [u8; 4]) {
    let stride = width as usize * 4;
    for x in 0..width as usize {
        for y in 0..2usize {
            let idx = y * stride + x * 4;
            if idx + 3 < canvas.len() {
                canvas[idx] = color[2];
                canvas[idx + 1] = color[1];
                canvas[idx + 2] = color[0];
                canvas[idx + 3] = 255;
            }
        }
    }
}

// ─── Main render entry ────────────────────────────────────────────────────────

pub fn render(canvas: &mut [u8], width: u32, _height: u32, scale: f32, center_text: &str) {
    let mut cache = GLYPH_CACHE.lock().unwrap();
    set_render_scale(scale);

    // Gather data
    let (lock_icon, lock_color) = get_lock();
    let (wifi_text, wifi_color) = get_wifi();
    let time_str = Local::now().format("%a %b %d · %H:%M").to_string();

    // Top accentâgreen locked, amber unlocked
    draw_top_accent(canvas, width, lock_color);

    // ââ LEFT: lock icon only ââââââââââââââââââââââââââââââââ
    let lock_x = draw_text(&mut cache, canvas, width, lock_icon, 14, lock_color);
    let lock_label = if lock_color == GREEN { " LOCKED" } else { " OPEN" };
    let label_x = draw_text(&mut cache, canvas, width, lock_label, lock_x + 4, lock_color);
    draw_separator(canvas, width, label_x + 8);

    // ââ CENTER: intent or Friday signal âââââââââââââââââââââââ
    if !center_text.is_empty() {
        let w = text_width(&mut cache, center_text);
        let cx = (width as i32 / 2) - (w / 2);
        if cx > 0 {
            let color = if center_text.starts_with("Friday:") { AMBER } else { GREEN };
            draw_text(&mut cache, canvas, width, center_text, cx, color);
        }
    }

    // ââ RIGHT: WiFi Â· date/time âââââââââââââââââââââââââââââââââââââââ
    let padding = 14i32;
    let time_w = text_width(&mut cache, &time_str);
    let mut rx = width as i32 - time_w - padding;
    draw_text(&mut cache, canvas, width, &time_str, rx, AMBER);

    rx -= padding;
    draw_separator(canvas, width, rx);
    rx -= padding;

    let wifi_w = text_width(&mut cache, &wifi_text);
    rx -= wifi_w;
    draw_text(&mut cache, canvas, width, &wifi_text, rx, wifi_color);

    // Separator: right zone left edge
    draw_separator(canvas, width, rx - 10);
}
