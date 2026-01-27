//! Bar mode rendering - status bar display

use faelight_core::GlyphCache;
use chrono::Local;
use std::process::Command;
use std::fs;
use std::env;

// Colors
const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const ACCENT_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const DIM_COLOR: [u8; 4] = [0x77, 0x7f, 0x6f, 0xFF];
const BLUE_COLOR: [u8; 4] = [0xff, 0xc8, 0x5c, 0xFF];
const AMBER_COLOR: [u8; 4] = [0x77, 0xc1, 0xf5, 0xFF];
const RED_COLOR: [u8; 4] = [0x70, 0x87, 0xd0, 0xFF];
const BG_COLOR: [u8; 4] = [0x11, 0x14, 0x0f, 0xFF];

const ICON_LOCKED: &str = "󰌾";
const ICON_UNLOCKED: &str = "󰌿";
const ICON_LAUNCHER: &str = "▶";
const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Regular.ttf");

lazy_static::lazy_static! {
    static ref GLYPH_CACHE: std::sync::Mutex<GlyphCache> = {
        std::sync::Mutex::new(
            GlyphCache::new(FONT_DATA).expect("Failed to load font")
        )
    };
}

pub fn render(canvas: &mut [u8], width: u32, _height: u32) -> Vec<(i32, i32, String)> {
    let mut cache = GLYPH_CACHE.lock().unwrap();
    let mut click_regions = Vec::new();
    
    // Top accent line
    let profile = get_current_profile();
    let accent = get_profile_color(&profile);
    
    for x in 0..width as usize {
        for y in 0..2 {
            let idx = (y * width as usize + x) * 4;
            if idx + 3 < canvas.len() {
                canvas[idx] = accent[0];
                canvas[idx + 1] = accent[1];
                canvas[idx + 2] = accent[2];
                canvas[idx + 3] = accent[3];
            }
        }
    }
    
    // Left side
    let mut x_pos = 10;
    
    // Profile (CLICKABLE)
    let profile_start = x_pos;
    let profile_icon = get_profile_icon(&profile);
    draw_text(&mut cache, canvas, width, profile_icon, x_pos, 8, accent);
    x_pos += 40;
    click_regions.push((profile_start, x_pos, "profile".to_string()));
    
    draw_gradient_separator(canvas, width, x_pos, DIM_COLOR);
    x_pos += 15;
    
    // Workspaces
    let (workspaces, active) = get_workspaces();
    for ws in &workspaces {
        let color = if *ws == active { ACCENT_COLOR } else { DIM_COLOR };
        let ws_str = format!("{}", ws);
        draw_text(&mut cache, canvas, width, &ws_str, x_pos, 8, color);
        x_pos += 18;
    }
    
    x_pos += 10;
    draw_gradient_separator(canvas, width, x_pos, DIM_COLOR);
    x_pos += 15;
    
    // Health
    let health = get_health();
    let health_color = if health >= 80 {
        ACCENT_COLOR
    } else if health >= 50 {
        AMBER_COLOR
    } else {
        RED_COLOR
    };
    let health_text = format!("● {}%", health);
    draw_text(&mut cache, canvas, width, &health_text, x_pos, 8, health_color);
    x_pos += 60;
    
    // Lock status
    let locked = is_core_locked();
    let lock_color = if locked { ACCENT_COLOR } else { AMBER_COLOR };
    let lock_icon = if locked { ICON_LOCKED } else { ICON_UNLOCKED };
    draw_text(&mut cache, canvas, width, lock_icon, x_pos, 8, lock_color);
    x_pos += 25;
    
    // Launcher icon (CLICKABLE)
    let launcher_start = x_pos;
    draw_text(&mut cache, canvas, width, ICON_LAUNCHER, x_pos, 8, BLUE_COLOR);
    x_pos += 25;
    click_regions.push((launcher_start, x_pos, "launcher".to_string()));
    
    // Center - active window
    let window_title = get_active_window();
    if !window_title.is_empty() {
        let title_width = window_title.len() as i32 * 8;
        let center_x = (width as i32 / 2) - (title_width / 2);
        draw_text(&mut cache, canvas, width, &window_title, center_x, 8, TEXT_COLOR);
    }
    
    // Right side
    let mut rx = width as i32 - 110;
    
    // Time
    let time_str = Local::now().format("%b %d %H:%M").to_string();
    draw_text(&mut cache, canvas, width, &time_str, rx, 8, AMBER_COLOR);
    
    rx -= 15;
    draw_gradient_separator(canvas, width, rx, DIM_COLOR);
    
    // Volume (CLICKABLE)
    rx -= 40;
    let vol_start = rx;
    let (vol, muted) = get_volume();
    let vol_color = if muted { DIM_COLOR } else { ACCENT_COLOR };
    let vol_text = if muted { "MUT".to_string() } else { format!("{}%", vol) };
    draw_text(&mut cache, canvas, width, &vol_text, rx, 8, vol_color);
    click_regions.push((vol_start, vol_start + 35, "volume".to_string()));
    
    rx -= 15;
    draw_gradient_separator(canvas, width, rx, DIM_COLOR);
    
    // WiFi
    rx -= 45;
    let (wifi_on, wifi_status) = get_wifi();
    let wifi_color = if wifi_on { ACCENT_COLOR } else { RED_COLOR };
    let wifi_text = format!("W:{}", wifi_status);
    draw_text(&mut cache, canvas, width, &wifi_text, rx, 8, wifi_color);
    
    rx -= 15;
    draw_gradient_separator(canvas, width, rx, DIM_COLOR);
    
    // Battery
    rx -= 45;
    let (bat_pct, charging) = get_battery();
    let bat_color = if bat_pct < 20 {
        RED_COLOR
    } else if bat_pct < 50 {
        AMBER_COLOR
    } else if charging {
        BLUE_COLOR
    } else {
        ACCENT_COLOR
    };
    let bat_text = format!("{}%{}", bat_pct, if charging { "+" } else { "" });
    draw_text(&mut cache, canvas, width, &bat_text, rx, 8, bat_color);
    
    rx -= 15;
    draw_gradient_separator(canvas, width, rx, DIM_COLOR);
    
    // VPN (CLICKABLE)
    rx -= 60;
    let vpn_start = rx;
    let (vpn_connected, vpn_status) = get_vpn_status();
    let vpn_color = if vpn_connected { ACCENT_COLOR } else { RED_COLOR };
    let vpn_text = format!("VPN:{}", vpn_status);
    draw_text(&mut cache, canvas, width, &vpn_text, rx, 8, vpn_color);
    click_regions.push((vpn_start, vpn_start + 55, "vpn".to_string()));
    
    click_regions
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
                
                if px >= 0 && px < width as i32 && py >= 0 && py < 32 {
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

fn draw_gradient_separator(canvas: &mut [u8], width: u32, x: i32, color: [u8; 4]) {
    let height = 32;
    let start_y = 6;
    let end_y = height - 6;
    
    for y in start_y..end_y {
        let progress = (y - start_y) as f32 / (end_y - start_y) as f32;
        let alpha = if progress < 0.2 {
            progress / 0.2
        } else if progress > 0.8 {
            (1.0 - progress) / 0.2
        } else {
            1.0
        };
        
        if x >= 0 && x < width as i32 {
            let idx = (y as usize * width as usize + x as usize) * 4;
            if idx + 3 < canvas.len() {
                canvas[idx] = ((1.0 - alpha) * BG_COLOR[0] as f32 + alpha * color[0] as f32) as u8;
                canvas[idx + 1] = ((1.0 - alpha) * BG_COLOR[1] as f32 + alpha * color[1] as f32) as u8;
                canvas[idx + 2] = ((1.0 - alpha) * BG_COLOR[2] as f32 + alpha * color[2] as f32) as u8;
                canvas[idx + 3] = 255;
            }
        }
    }
}

fn get_profile_icon(profile: &str) -> &'static str {
    match profile {
        "gaming" => "GAM",
        "work" => "WRK",
        "low-power" => "LOW",
        _ => "DEF",
    }
}

fn get_profile_color(profile: &str) -> [u8; 4] {
    match profile {
        "gaming" => RED_COLOR,
        "work" => BLUE_COLOR,
        "low-power" => AMBER_COLOR,
        _ => ACCENT_COLOR,
    }
}

fn get_current_profile() -> String {
    let home = env::var("HOME").unwrap_or_default();
    let path = format!("{}/.local/state/0-core/current-profile", home);
    fs::read_to_string(&path)
        .unwrap_or_else(|_| "default".to_string())
        .trim()
        .to_string()
}

fn get_vpn_status() -> (bool, String) {
    match Command::new("mullvad").arg("status").output() {
        Ok(out) => {
            let status = String::from_utf8_lossy(&out.stdout);
            let connected = status.contains("Connected");
            (connected, if connected { "ON".to_string() } else { "OFF".to_string() })
        }
        Err(_) => (false, "N/A".to_string()),
    }
}

fn get_battery() -> (u8, bool) {
    let capacity = fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/capacity"))
        .unwrap_or_else(|_| "0".to_string());
    
    let status = fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/status"))
        .unwrap_or_else(|_| "Unknown".to_string());
    
    let percent: u8 = capacity.trim().parse().unwrap_or(0);
    let charging = status.trim() == "Charging";
    (percent, charging)
}

fn get_wifi() -> (bool, String) {
    match Command::new("iwctl").args(["station", "wlan0", "show"]).output() {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            let connected = result.lines()
                .any(|line| line.trim().contains("State") && line.contains("connected"));
            (connected, if connected { "ON".to_string() } else { "OFF".to_string() })
        }
        Err(_) => (false, "N/A".to_string()),
    }
}

fn get_volume() -> (u8, bool) {
    match Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output() {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            let muted = result.contains("[MUTED]");
            let vol = result
                .split_whitespace()
                .nth(1)
                .and_then(|v| v.parse::<f32>().ok())
                .map(|v| (v * 100.0) as u8)
                .unwrap_or(0);
            (vol, muted)
        }
        Err(_) => (0, false),
    }
}

fn get_health() -> u8 {
    let home = env::var("HOME").unwrap_or_default();
    let core_path = format!("{}/0-core", home);
    let mut passed = 0;
    let total = 5;
    
    if fs::metadata(&core_path).is_ok() { passed += 1; }
    
    let scripts_path = format!("{}/scripts", core_path);
    if let Ok(entries) = fs::read_dir(&scripts_path) {
        if entries.count() > 5 { passed += 1; }
    }
    
    if let Ok(out) = Command::new("git")
        .args(["-C", &core_path, "status", "--porcelain"])
        .output() 
    {
        if out.stdout.is_empty() { passed += 1; }
    }
    
    let profile_path = format!("{}/.local/state/0-core/current-profile", home);
    if fs::metadata(&profile_path).is_ok() { passed += 1; }
    
    let version_path = format!("{}/VERSION", core_path);
    if fs::metadata(&version_path).is_ok() { passed += 1; }
    
    ((passed * 100) / total) as u8
}

fn is_core_locked() -> bool {
    let home = env::var("HOME").unwrap_or_default();
    let core_path = format!("{}/0-core", home);
    
    match Command::new("lsattr").args(["-d", &core_path]).output() {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            result
                .split_whitespace()
                .next()
                .map(|attrs| attrs.contains('i'))
                .unwrap_or(false)
        }
        Err(_) => false,
    }
}

fn sway_query(cmd: &str) -> Option<String> {
    Command::new("swaymsg")
        .args(["-t", cmd, "-r"])
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
}

fn get_workspaces() -> (Vec<i32>, i32) {
    let mut workspaces: Vec<i32> = vec![];
    let mut active: i32 = 1;
    
    if let Some(resp) = sway_query("get_workspaces") {
        for num in 1..=10 {
            let pattern = format!("\"num\":{}", num);
            let pattern2 = format!("\"num\": {}", num);
            if resp.contains(&pattern) || resp.contains(&pattern2) {
                workspaces.push(num);
            }
        }
        
        let focused_pattern = "\"focused\":true";
        let focused_pattern2 = "\"focused\": true";
        if let Some(focused_pos) = resp.find(focused_pattern).or_else(|| resp.find(focused_pattern2)) {
            let before = &resp[..focused_pos];
            let num_pattern = "\"num\":";
            let num_pattern2 = "\"num\": ";
            if let Some(num_pos) = before.rfind(num_pattern).or_else(|| before.rfind(num_pattern2)) {
                let after_num = &before[num_pos + 6..];
                let num_str: String = after_num.chars()
                    .skip_while(|c| !c.is_numeric())
                    .take_while(|c| c.is_numeric())
                    .collect();
                if let Ok(num) = num_str.parse() {
                    active = num;
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
    if let Some(resp) = sway_query("get_tree") {
        let focused_pattern = "\"focused\": true";
        if let Some(focused_pos) = resp.find(focused_pattern) {
            let after = &resp[focused_pos..];
            
            let app_pattern = "\"app_id\": \"";
            if let Some(app_pos) = after.find(app_pattern) {
                let start = app_pos + 11;
                if let Some(end) = after[start..].find('"') {
                    let app_id = &after[start..start + end];
                    if !app_id.is_empty() && app_id != "null" {
                        return app_id.to_string();
                    }
                }
            }
            
            let name_pattern = "\"name\": \"";
            if let Some(name_pos) = after.find(name_pattern) {
                let start = name_pos + 9;
                if let Some(end) = after[start..].find('"') {
                    let title = &after[start..start + end];
                    if !title.is_empty() && title != "null" {
                        if title.len() > 40 {
                            return format!("{}...", &title[..37]);
                        }
                        return title.to_string();
                    }
                }
            }
        }
    }
    String::new()
}
