//! faelight-launcher v3.1 - Refined UI
//! 🌲 Faelight Forest

use std::time::Duration;
use std::env;
use fontdue::{Font, FontSettings};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::process::Command;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};


mod desktop;
mod search;
use desktop::{DesktopEntry, icons::IconConfig};
// ═══════════════════════════════════════════════════════════
// 🎨 FAELIGHT FOREST COLORS
// ═══════════════════════════════════════════════════════════
const WIDTH: u32 = 420;
const HEIGHT: u32 = 680;

const BG_COLOR: [u8; 4] = [0x14, 0x17, 0x11, 0xD0];
const BORDER_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const SELECTED_BG: [u8; 4] = [0x3a, 0x4a, 0x35, 0xFF];
const DIM_COLOR: [u8; 4] = [0x7f, 0x8f, 0x77, 0xFF];

// ═══════════════════════════════════════════════════════════
// 📐 TYPOGRAPHY & LAYOUT
// ═══════════════════════════════════════════════════════════
const FONT_TITLE: f32 = 28.0;
const FONT_SEARCH: f32 = 19.0;
const FONT_ITEM: f32 = 22.0;
const FONT_SUBTITLE: f32 = 14.0;
const FONT_HINT: f32 = 16.0;
const ROW_HEIGHT: u32 = 52;
const ROW_START: u32 = 110;
const MAX_VISIBLE: usize = 10;

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf");

// ═══════════════════════════════════════════════════════════
// 📱 APP ENTRIES
// ═══════════════════════════════════════════════════════════
#[derive(Debug, Clone)]
struct AppEntry {
    name: String,
    exec: String,
    icon: String,
}

impl From<&DesktopEntry> for AppEntry {
    fn from(entry: &DesktopEntry) -> Self {
        let icon_config = IconConfig::load();
        let icon = icon_config.get_icon(&entry.clean_exec(), &entry.categories);
        
        AppEntry {
            name: entry.name.clone(),
            exec: entry.clean_exec(),
            icon,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 📈 FRECENCY TRACKING
// ═══════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Default)]
struct LaunchHistory {
    launches: HashMap<String, AppHistory>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppHistory {
    count: u32,
    last_used: u64,
}

impl LaunchHistory {
    fn load() -> Self {
        let path = Self::history_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(history) = serde_json::from_str(&data) {
                    return history;
                }
            }
        }
        Self::default()
    }

    fn save(&self) {
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.launches) {
            std::fs::write(&path, json).ok();
        }
    }

    fn history_path() -> std::path::PathBuf {
        let home = std::env::var("HOME").expect("HOME not set");
        std::path::PathBuf::from(home).join(".local/state/faelight/launcher-history.json")
    }

    fn record_launch(&mut self, app_name: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        self.launches
            .entry(app_name.to_string())
            .and_modify(|h| {
                h.count += 1;
                h.last_used = now;
            })
            .or_insert(AppHistory {
                count: 1,
                last_used: now,
            });
    }

    fn frecency_score(&self, app_name: &str) -> f32 {
        if let Some(history) = self.launches.get(app_name) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            let age_hours = (now - history.last_used) / 3600;
            let recency = if age_hours < 1 {
                1.0
            } else if age_hours < 24 {
                0.8
            } else if age_hours < 168 {
                0.5
            } else {
                0.2
            };
            let frequency = (history.count as f32).ln().max(0.0) / 5.0;
            recency * 0.6 + frequency.min(1.0) * 0.4
        } else {
            0.0
        }
    }
}



// Fuzzy matching
fn keysym_to_char(keysym: Keysym) -> Option<char> {
    match keysym {
        Keysym::a => Some('a'),
        Keysym::b => Some('b'),
        Keysym::c => Some('c'),
        Keysym::d => Some('d'),
        Keysym::e => Some('e'),
        Keysym::f => Some('f'),
        Keysym::g => Some('g'),
        Keysym::h => Some('h'),
        Keysym::i => Some('i'),
        Keysym::j => Some('j'),
        Keysym::k => Some('k'),
        Keysym::l => Some('l'),
        Keysym::m => Some('m'),
        Keysym::n => Some('n'),
        Keysym::o => Some('o'),
        Keysym::p => Some('p'),
        Keysym::q => Some('q'),
        Keysym::r => Some('r'),
        Keysym::s => Some('s'),
        Keysym::t => Some('t'),
        Keysym::u => Some('u'),
        Keysym::v => Some('v'),
        Keysym::w => Some('w'),
        Keysym::x => Some('x'),
        Keysym::y => Some('y'),
        Keysym::z => Some('z'),
        Keysym::space => Some(' '),
        _ => None,
    }
}

fn fuzzy_score(query: &str, target: &str) -> i32 {
    if query.is_empty() {
        return 100;
    }
    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();

    // Exact match
    if target_lower == query_lower {
        return 1000;
    }
    // Starts with
    if target_lower.starts_with(&query_lower) {
        return 500;
    }
    // Contains substring
    if target_lower.contains(&query_lower) {
        return 300;
    }
    // Fuzzy match (chars in order)
    let mut score = 100;
    let mut query_chars = query_lower.chars().peekable();
    let mut consecutive = 0;
    for ch in target_lower.chars() {
        if query_chars.peek() == Some(&ch) {
            query_chars.next();
            consecutive += 1;
            score += consecutive * 10;
        } else {
            consecutive = 0;
        }
    }
    if query_chars.peek().is_none() {
        score
    } else {
        0
    }
}

fn filter_apps(query: &str, apps: &[AppEntry], history: &LaunchHistory) -> Vec<AppEntry> {
    let mut results: Vec<(AppEntry, f32)> = apps
        .iter()
        .map(|app| {
            let fuzzy = fuzzy_score(query, &app.name) as f32;
            let frecency = history.frecency_score(&app.name) * 100.0;
            (app.clone(), fuzzy + frecency)
        })
        .filter(|(_, score)| *score > 0.0)
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.into_iter().map(|(app, _)| app).collect()
}
// ═══════════════════════════════════════════════════════════
// 🖼️ DRAWING HELPERS (standalone)
// ═══════════════════════════════════════════════════════════
fn draw_border(canvas: &mut [u8], width: u32, height: u32) {
    let stride = width as usize * 4;
    for x in 0..width as usize {
        canvas[x * 4..x * 4 + 4].copy_from_slice(&BORDER_COLOR);
        canvas[(height as usize - 1) * stride + x * 4..(height as usize - 1) * stride + x * 4 + 4]
            .copy_from_slice(&BORDER_COLOR);
    }
    for y in 0..height as usize {
        canvas[y * stride..y * stride + 4].copy_from_slice(&BORDER_COLOR);
        canvas[y * stride + (width as usize - 1) * 4..y * stride + (width as usize - 1) * 4 + 4]
            .copy_from_slice(&BORDER_COLOR);
    }
}

fn draw_rect(
    canvas: &mut [u8],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: [u8; 4],
) {
    let stride = width as usize * 4;
    for row in y..y + h {
        for col in x..x + w {
            if col < width && row < height {
                let idx = row as usize * stride + col as usize * 4;
                canvas[idx..idx + 4].copy_from_slice(&color);
            }
        }
    }
}

fn draw_text(
    font: &Font,
    canvas: &mut [u8],
    width: u32,
    height: u32,
    text: &str,
    x: u32,
    y: u32,
    color: [u8; 4],
    size: f32,
) {
    let stride = width as usize * 4;
    let mut cursor_x = x as usize;
    
    // Calculate baseline offset (max ascent for this size)
    let baseline_offset = (size * 0.7) as i32; // Calculate baseline for proper text alignment
    
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, size);
        
        // Calculate vertical position with proper baseline alignment
        let glyph_y = y as i32;
        
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha > 0 {
                    let px = cursor_x + col;
                    let py = (glyph_y + row as i32) as usize;
                    
                    if px < width as usize && py < height as usize {
                        let idx = py * stride + px * 4;
                        let a = alpha as f32 / 255.0;
                        canvas[idx] = ((1.0 - a) * canvas[idx] as f32 + a * color[0] as f32) as u8;
                        canvas[idx + 1] =
                            ((1.0 - a) * canvas[idx + 1] as f32 + a * color[1] as f32) as u8;
                        canvas[idx + 2] =
                            ((1.0 - a) * canvas[idx + 2] as f32 + a * color[2] as f32) as u8;
                        canvas[idx + 3] = 255;
                    }
                }
            }
        }
        cursor_x += metrics.advance_width as usize;
    }
}
// ═══════════════════════════════════════════════════════════
// 🖼️ STATE
// ═══════════════════════════════════════════════════════════
struct LauncherState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    #[allow(dead_code)]
    compositor_state: CompositorState,
    shm: Shm,
    #[allow(dead_code)]
    layer_shell: LayerShell,
    layer_surface: Option<LayerSurface>,
    pool: Option<SlotPool>,
    width: u32,
    height: u32,
    configured: bool,
    font: Font,
    selected: usize,
    search_query: String,
    running: bool,
    history: LaunchHistory,
    apps: Vec<AppEntry>,
    scroll_offset: usize,
}

impl LauncherState {
    fn draw(&mut self) {
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;
        let selected = self.selected;

        let pool = match &mut self.pool {
            Some(p) => p,
            None => return,
        };

        let (buffer, canvas) = match pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(_) => return,
        };

        // Clear background
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&BG_COLOR);
        }

        draw_border(canvas, width, height);
        draw_text(
            &self.font,
            canvas,
            width,
            height,
            "󰐅 Faelight Launcher",
            20,
            18,
            BORDER_COLOR,
            FONT_TITLE,
        );

        // Search box
        let search_display = if self.search_query.is_empty() {
            "Type to search...".to_string()
        } else {
            self.search_query.clone()
        };
        let search_color = if self.search_query.is_empty() {
            DIM_COLOR
        } else {
            TEXT_COLOR
        };
        draw_rect(canvas, width, height, 15, 45, width - 30, 32, SELECTED_BG);
        draw_text(
            &self.font,
            canvas,
            width,
            height,
            &format!("🔍 {}", search_display),
            20,
            50,
            search_color,
            FONT_SEARCH,
        );

        // Separator line
        let sep_y = 80;
        for x in 10..width as usize - 10 {
            let idx = sep_y * width as usize * 4 + x * 4;
            canvas[idx..idx + 4].copy_from_slice(&DIM_COLOR);
        }

        // Draw apps
        let filtered_apps = universal_search(&self.search_query, &self.apps, &self.history);
        
        // Calculate visible window
        let visible_start = self.scroll_offset;
        let visible_end = (self.scroll_offset + MAX_VISIBLE).min(filtered_apps.len());
        
        for (i, result) in filtered_apps.iter().enumerate().skip(visible_start).take(visible_end - visible_start) {
            let display_index = i - visible_start;
            let y = ROW_START + display_index as u32 * ROW_HEIGHT;


            let color = if i == selected {
                BORDER_COLOR
            } else {
                TEXT_COLOR
            };
            // Two-line display
            match result {
                search::SearchResult::App { name, icon, score, .. } => {
                    // Line 1: Icon + Name + Score
                    let line1 = format!("{}  {}", icon, name);
                    draw_text(&self.font, canvas, width, height, &line1, 35, y, color, FONT_ITEM);
                    
                    // Line 2: Description (dimmed)
                    draw_text(&self.font, canvas, width, height, "Application", 40, y + 28, DIM_COLOR, FONT_SUBTITLE);
                }
                search::SearchResult::File { name, path, modified, score, .. } => {
                    // Line 1: Icon + Name + Time
                    let time = format_time_ago(*modified);
                    let line1 = format!("📄  {}  {}", name, time);
                    draw_text(&self.font, canvas, width, height, &line1, 35, y, color, FONT_ITEM);
                    
                    // Line 2: Smart path (dimmed)
                    let short_path = smart_path(path);
                    draw_text(&self.font, canvas, width, height, &short_path, 40, y + 28, DIM_COLOR, FONT_SUBTITLE);
                }
            }
        }

        draw_text(
            &self.font,
            canvas,
            width,
            height,
            "↑↓ Navigate  Enter Launch  Esc Close",
            20,
            height - 25,
            DIM_COLOR,
            FONT_HINT,
        );

        if let Some(ref surface) = self.layer_surface {
            surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
            surface
                .wl_surface()
                .damage_buffer(0, 0, width as i32, height as i32);
            surface.wl_surface().commit();
        }
    }

    fn launch_selected(&mut self) {
        let filtered_apps = universal_search(&self.search_query, &self.apps, &self.history);
        if filtered_apps.is_empty() {
            return;
        }
        let result = &filtered_apps[self.selected.min(filtered_apps.len() - 1)];
        match result {
            search::SearchResult::App { name, exec, .. } => {
                eprintln!("🚀 Launching: {}", name);
                let parts: Vec<&str> = exec.split_whitespace().collect();
                if let Some((cmd, args)) = parts.split_first() {
                    Command::new(cmd).args(args).spawn().ok();
                }
                self.history.record_launch(name);
                self.history.save();
            }
            search::SearchResult::File { name, path, .. } => {
                eprintln!("📂 Opening: {}", name);
                Command::new("xdg-open").arg(path).spawn().ok();
            }
        }
    }

    fn move_up(&mut self) {
        let filtered_len = universal_search(&self.search_query, &self.apps, &self.history).len();
        if filtered_len == 0 {
            return;
        }
        if self.selected > 0 {
            self.selected -= 1;
            // Scroll up if needed
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        } else {
            self.selected = filtered_len - 1;
            self.scroll_offset = filtered_len.saturating_sub(MAX_VISIBLE);
        }
    }

    fn move_down(&mut self) {
        let filtered_len = universal_search(&self.search_query, &self.apps, &self.history).len();
        if filtered_len == 0 {
            return;
        }
        if self.selected < filtered_len - 1 {
            self.selected += 1;
            // Scroll down if needed
            if self.selected >= self.scroll_offset + MAX_VISIBLE {
                self.scroll_offset = self.selected - MAX_VISIBLE + 1;
            }
        } else {
            self.selected = 0;
            self.scroll_offset = 0;
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🔧 HANDLERS
// ═══════════════════════════════════════════════════════════
impl CompositorHandler for LauncherState {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for LauncherState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for LauncherState {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.running = false;
    }
    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        self.width = configure.new_size.0.max(WIDTH);
        self.height = configure.new_size.1.max(HEIGHT);
        self.configured = true;
        self.draw();
    }
}

impl SeatHandler for LauncherState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard {
            self.seat_state.get_keyboard(qh, &seat, None).ok();
        }
    }
    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _: Capability,
    ) {
    }
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for LauncherState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
    }
    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
    ) {
    }
    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: Modifiers,
        _: u32,
    ) {
    }

    fn press_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        match event.keysym {
            Keysym::Escape => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.selected = 0;
                    self.scroll_offset = 0;
                    self.draw();
                } else {
                    self.running = false;
                }
            }
            Keysym::Return | Keysym::KP_Enter => {
                self.launch_selected();
                self.running = false;
            }
            Keysym::Up | Keysym::k => {
                self.move_up();
                self.draw();
            }
            Keysym::Down | Keysym::j => {
                self.move_down();
                self.draw();
            }
            Keysym::BackSpace => {
                self.search_query.pop();
                self.selected = 0;
                self.scroll_offset = 0;
                self.draw();
            }
            _ => {
                if let Some(ch) = keysym_to_char(event.keysym) {
                    self.search_query.push(ch);
                    self.selected = 0;
                    self.scroll_offset = 0;
                    self.draw();
                }
            }
        }
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }
    fn update_repeat_info(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: smithay_client_toolkit::seat::keyboard::RepeatInfo,
    ) {
    }
}

impl ShmHandler for LauncherState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for LauncherState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(LauncherState);
delegate_output!(LauncherState);
delegate_shm!(LauncherState);
delegate_seat!(LauncherState);
delegate_keyboard!(LauncherState);
delegate_layer!(LauncherState);
delegate_registry!(LauncherState);

// ═══════════════════════════════════════════════════════════
fn health_check() {
    println!("🏥 faelight-launcher health check");
    
    // Check Wayland
    match Connection::connect_to_env() {
        Ok(_) => println!("✅ wayland: connected"),
        Err(e) => {
            eprintln!("❌ wayland: failed - {}", e);
            std::process::exit(1);
        }
    }
    
    // Check font
    let font_data = include_bytes!("/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf");
    match Font::from_bytes(font_data as &[u8], FontSettings::default()) {
        Ok(_) => println!("✅ font: loaded"),
        Err(e) => {
            eprintln!("❌ font: failed - {}", e);
            std::process::exit(1);
        }
    }
    
    // Check desktop files
    let desktop_dirs = [
        "/usr/share/applications",
        &format!("{}/.local/share/applications", env::var("HOME").unwrap_or_default()),
    ];
    
    for dir in desktop_dirs {
        if std::path::Path::new(dir).exists() {
            println!("✅ desktop files: {} exists", dir);
        } else {
            eprintln!("⚠️  desktop files: {} not found", dir);
        }
    }
    
    println!("\n✅ Core checks passed!");
}

// 🚀 MAIN
// ═══════════════════════════════════════════════════════════
fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🌲 faelight-launcher v3.1.0 starting...");
    // Check for health flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "--health" || args[1] == "health") {
        health_check();
        return Ok(());
    }
    

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;

    let surface = compositor.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Overlay,
        Some("faelight-launcher"),
        None,
    );

    layer_surface.set_anchor(Anchor::empty());
    layer_surface.set_size(WIDTH, HEIGHT);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
    layer_surface.commit();

    let pool = SlotPool::new(WIDTH as usize * HEIGHT as usize * 4, &shm)?;
        // Scan XDG desktop entries
    println!("🌲 Faelight Launcher v2.0 - Loading...");
    let entries = desktop::scan_applications();
    let apps: Vec<AppEntry> = entries.iter().map(|e| e.into()).collect();
    println!("📱 Discovered {} applications", apps.len());
    
    let font = Font::from_bytes(FONT_DATA, FontSettings::default())?;

    let mut state = LauncherState {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        compositor_state: compositor,
        shm,
        layer_shell,
        layer_surface: Some(layer_surface),
        pool: Some(pool),
        width: WIDTH,
        height: HEIGHT,
        configured: false,
        font,
        selected: 0,
        search_query: String::new(),
        history: LaunchHistory::load(),
        running: true,
        apps,
        scroll_offset: 0,
    };

    while state.running {
        event_queue.blocking_dispatch(&mut state)?;
    }

    eprintln!("👋 Goodbye!");
    Ok(())
}

// ═══════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════
// 🎨 DISPLAY HELPERS
// ═══════════════════════════════════════════════════════════

/// Format elapsed time (e.g., "2h ago", "3d ago")
fn format_time_ago(modified: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let elapsed = now.saturating_sub(modified);
    
    if elapsed < 3600 {
        format!("{}m ago", elapsed / 60)
    } else if elapsed < 86400 {
        format!("{}h ago", elapsed / 3600)
    } else if elapsed < 604800 {
        format!("{}d ago", elapsed / 86400)
    } else {
        format!("{}w ago", elapsed / 604800)
    }
}

/// Smart path truncation (show parent dir + filename)
fn smart_path(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let short = path.replace(&home, "~");
    
    // If path is short enough, return it
    if short.len() <= 50 {
        return short;
    }
    
    // Otherwise, show last 2 path components
    let parts: Vec<&str> = short.split('/').collect();
    if parts.len() >= 3 {
        format!(".../{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
    } else {
        short
    }
}
// 🔍 UNIVERSAL SEARCH
// ═══════════════════════════════════════════════════════════
fn universal_search(
    query: &str,
    apps: &[AppEntry],
    history: &LaunchHistory,
) -> Vec<search::SearchResult> {
    use search::{SearchResult, files};
    
    let mut results = Vec::new();
    
    // Search apps
    for app in apps {
        let fuzzy = fuzzy_score(query, &app.name) as f32;
        let frecency = history.frecency_score(&app.name) * 100.0;
        let score = fuzzy + frecency;
        
        if score > 0.0 {
            results.push(SearchResult::App {
                name: app.name.clone(),
                exec: app.exec.clone(),
                icon: app.icon.clone(),
                score,
            });
        }
    }
    
    // Search files (only if query has 2+ chars to avoid noise)
    if query.len() >= 2 {
        let file_config = files::FileSearchConfig::default();
        let file_results = files::search_files(query, &file_config);
        results.extend(file_results);
    }
    
    // Sort by score (highest first)
    results.sort();
    results
}
