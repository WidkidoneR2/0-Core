//! faelight-bar render - clean rebuild v5.0.0

use chrono::Local;
use faelight_core::GlyphCache;
use rusqlite;
use std::fs;
use std::process::Command;

// Color palette - matches faelight-fm/palette
const BG: [u8; 4] = [0x11, 0x14, 0x0f, 0xFF];
const FG: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const GREEN: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const BLUE: [u8; 4] = [0x6b, 0xa3, 0xe3, 0xFF];
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

fn get_profile() -> String {
    let path = faelight_core::paths::current_profile_file();
    fs::read_to_string(&path)
        .unwrap_or_else(|_| "default".to_string())
        .trim()
        .to_string()
}

fn profile_label_color(profile: &str) -> (&'static str, [u8; 4]) {
    match profile {
        "gaming" => ("GAME", RED),
        "work" => ("WORK", BLUE),
        _ => ("DEF", GREEN),
    }
}

fn get_focused_cwd() -> Option<String> {
    // INT-180: use niri msg focused-window to get pid, then read /proc/<pid>/cwd
    let out = Command::new("niri")
        .args(["msg", "-j", "focused-window"])
        .output()
        .ok()?;
    let json = String::from_utf8(out.stdout).ok()?;
    // Extract pid from JSON: {"pid": 12345, ...}
    let pid_key = "\"pid\":";
    let pid_pos = json.find(pid_key)?;
    let after = &json[pid_pos + pid_key.len()..];
    let pid_str: String = after
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let pid: u32 = pid_str.parse().ok()?;
    fs::read_link(format!("/proc/{}/cwd", pid))
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
}

fn get_zone() -> (String, [u8; 4]) {
    // Get zone from focused window cwd, fall back to faelight-zone
    let cwd = get_focused_cwd().unwrap_or_default();
    let label = if !cwd.is_empty() {
        let out = Command::new("faelight-zone")
            .arg("--label")
            .env("PWD", &cwd)
            .output()
            .ok();
        out.and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .trim()
            .to_lowercase()
    } else {
        Command::new("faelight-zone")
            .arg("--label")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
            .trim()
            .to_lowercase()
    };

    match label.as_str() {
        "home" => ("HOME".to_string(), GREEN),
        "core" => ("CORE".to_string(), GREEN),
        "work" => ("WORK".to_string(), BLUE),
        "gaming" => ("GAME".to_string(), RED),
        "focus" => ("FOCUS".to_string(), AMBER),
        "learning" | "learn" => ("LEARN".to_string(), BLUE),
        "src" => ("SRC".to_string(), BLUE),
        s if !s.is_empty() => (s.to_uppercase(), FG),
        _ => ("HOME".to_string(), GREEN),
    }
}

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

fn get_health() -> (String, [u8; 4]) {
    // Read from cache file written by doctor — fast, no subprocess
    let home = std::env::var("HOME").unwrap_or_default();
    let cache_file = std::path::PathBuf::from(&home).join(".cache/faelight/health-status");
    let num = fs::read_to_string(&cache_file)
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0);
    if num == 0 {
        return ("HP:??".to_string(), DIM);
    }
    let color = if num >= 95 {
        GREEN
    } else if num >= 80 {
        AMBER
    } else {
        RED
    };
    (format!("HP:{}%", num), color)
}

fn get_workspaces() -> (Vec<i32>, i32) {
    let mut workspaces = vec![];
    let mut active = 1i32;

    // INT-180: niri workspaces
    if let Ok(out) = Command::new("niri")
        .args(["msg", "-j", "workspaces"])
        .output()
    {
        let resp = String::from_utf8_lossy(&out.stdout);
        for num in 1..=10 {
            if resp.contains(&format!("\"num\":{}", num))
                || resp.contains(&format!("\"num\": {}", num))
            {
                workspaces.push(num);
            }
        }
        if let Some(pos) = resp
            .find("\"focused\":true")
            .or_else(|| resp.find("\"focused\": true"))
        {
            let before = &resp[..pos];
            if let Some(npos) = before
                .rfind("\"num\":")
                .or_else(|| before.rfind("\"num\": "))
            {
                let after = &before[npos + 6..];
                let num_str: String = after
                    .chars()
                    .skip_while(|c| !c.is_ascii_digit())
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(n) = num_str.parse() {
                    active = n;
                }
            }
        }
    }

    workspaces.sort();
    if workspaces.is_empty() {
        workspaces = vec![1];
    }
    (workspaces, active)
}

fn get_active_window() -> String {
    // INT-180: use niri msg focused-window JSON
    let out = Command::new("niri")
        .args(["msg", "-j", "focused-window"])
        .output()
        .ok();
    out.and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|json| {
            // Extract title from {"title": "...", ...}
            let key = "\"title\":\"";
            let pos = json.find(key)?;
            let after = &json[pos + key.len()..];
            let end = after.find('\"')?;
            let title = &after[..end];
            if title.len() > 40 {
                Some(format!("{}...", &title[..37]))
            } else {
                Some(title.to_string())
            }
        })
        .unwrap_or_default()
}

fn get_vpn() -> (&'static str, [u8; 4]) {
    let connected = Command::new("mullvad")
        .arg("status")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.to_lowercase().contains("connected"))
        .unwrap_or(false);
    if connected {
        ("VPN ON", GREEN)
    } else {
        ("VPN OFF", RED)
    }
}

fn get_battery() -> (String, [u8; 4]) {
    let cap = fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/capacity"))
        .unwrap_or_default();
    let status = fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/status"))
        .unwrap_or_default();

    let level: u8 = cap.trim().parse().unwrap_or(0);
    let charging = status.trim() == "Charging";

    let text = if charging {
        format!("+{}%", level)
    } else {
        format!("BAT:{}%", level)
    };
    let color = if charging {
        BLUE
    } else if level > 50 {
        GREEN
    } else if level > 20 {
        AMBER
    } else {
        RED
    };
    (text, color)
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

fn get_volume() -> (String, [u8; 4]) {
    if let Ok(out) = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
    {
        if let Ok(result) = String::from_utf8(out.stdout) {
            if result.contains("MUTED") {
                return ("MUTE".to_string(), RED);
            }
            if let Some(val) = result
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<f32>().ok())
            {
                let pct = (val * 100.0) as u8;
                let color = if pct > 80 {
                    RED
                } else if pct > 60 {
                    AMBER
                } else {
                    GREEN
                };
                return (format!("VOL:{}%", pct), color);
            }
        }
    }
    ("VOL:??".to_string(), DIM)
}

#[allow(dead_code)]
fn get_active_intent() -> Option<(u32, String)> {
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

pub fn render(canvas: &mut [u8], width: u32, _height: u32, scale: f32) {
    let mut cache = GLYPH_CACHE.lock().unwrap();
    set_render_scale(scale);

    // Gather all data upfront
    let profile = get_profile();
    let (prof_label, prof_color) = profile_label_color(&profile);
    let (zone_text, zone_color) = get_zone();
    let (lock_icon, lock_color) = get_lock();
    let (health_text, health_color) = get_health();
    let (workspaces, active_ws) = get_workspaces();
    let window = get_active_window();
    let (vpn_text, vpn_color) = get_vpn();
    let (bat_text, bat_color) = get_battery();
    let (wifi_text, wifi_color) = get_wifi();
    let (vol_text, vol_color) = get_volume();
    let time_str = Local::now().format("%b %d  %H:%M").to_string();

    // Top accent line — profile color
    draw_top_accent(canvas, width, prof_color);

    // ── LEFT SIDE ─────────────────────────────────────────────────
    let mut x = 10i32;

    // Profile
    x = draw_text(&mut cache, canvas, width, prof_label, x, prof_color);
    x += 8;
    draw_separator(canvas, width, x);
    x += 12;

    // Workspaces
    for ws in &workspaces {
        let color = if *ws == active_ws { prof_color } else { DIM };
        let ws_str = ws.to_string();
        x = draw_text(&mut cache, canvas, width, &ws_str, x, color);
        x += 4;
    }
    x += 4;
    draw_separator(canvas, width, x);
    x += 12;

    // Zone (real-time)
    x = draw_text(&mut cache, canvas, width, &zone_text, x, zone_color);
    x += 8;
    draw_separator(canvas, width, x);
    x += 12;

    // Lock
    x = draw_text(&mut cache, canvas, width, lock_icon, x, lock_color);
    x += 8;
    draw_separator(canvas, width, x);
    x += 12;

    // Health
    draw_text(&mut cache, canvas, width, &health_text, x, health_color);

    // ── CENTER: active window ──────────────────────────────────────
    if !window.is_empty() {
        let w = text_width(&mut cache, &window);
        let cx = (width as i32 / 2) - (w / 2);
        if cx > 0 {
            draw_text(&mut cache, canvas, width, &window, cx, FG);
        }
    }

    // ── RIGHT SIDE (position right-to-left) ───────────────────────
    let padding = 12i32;

    // Time
    let time_w = text_width(&mut cache, &time_str);
    let mut rx = width as i32 - time_w - padding;
    draw_text(&mut cache, canvas, width, &time_str, rx, AMBER);

    rx -= padding;
    draw_separator(canvas, width, rx);
    rx -= padding;

    // Volume
    let vol_w = text_width(&mut cache, &vol_text);
    rx -= vol_w;
    draw_text(&mut cache, canvas, width, &vol_text, rx, vol_color);

    rx -= padding;
    draw_separator(canvas, width, rx);
    rx -= padding;

    // WiFi
    let wifi_w = text_width(&mut cache, &wifi_text);
    rx -= wifi_w;
    draw_text(&mut cache, canvas, width, &wifi_text, rx, wifi_color);

    rx -= padding;
    draw_separator(canvas, width, rx);
    rx -= padding;

    // Battery
    let bat_w = text_width(&mut cache, &bat_text);
    rx -= bat_w;
    draw_text(&mut cache, canvas, width, &bat_text, rx, bat_color);

    rx -= padding;
    draw_separator(canvas, width, rx);
    rx -= padding;

    // VPN
    let vpn_w = text_width(&mut cache, vpn_text);
    rx -= vpn_w;
    draw_text(&mut cache, canvas, width, vpn_text, rx, vpn_color);
}
