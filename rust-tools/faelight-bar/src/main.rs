use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState, pointer::{PointerEvent, PointerEventKind, PointerHandler}},
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};
use fontdue::{Font, FontSettings};
use chrono::Local;
use std::env;
use std::fs;
use std::process::Command;

const BAR_HEIGHT: u32 = 32;
const BG_COLOR: [u8; 4] = [0x11, 0x14, 0x0f, 0xFF];
const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const ACCENT_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const DIM_COLOR: [u8; 4] = [0x77, 0x7f, 0x6f, 0xFF];
const BLUE_COLOR: [u8; 4] = [0xff, 0xc8, 0x5c, 0xFF];
const AMBER_COLOR: [u8; 4] = [0x77, 0xc1, 0xf5, 0xFF];
const RED_COLOR: [u8; 4] = [0x70, 0x87, 0xd0, 0xFF];

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/Adwaita/AdwaitaMono-Regular.ttf");

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
    let output = Command::new("mullvad").arg("status").output();
    match output {
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
    let output = Command::new("iwctl")
        .args(["station", "wlan0", "show"])
        .output();
    
    match output {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            let mut connected = false;
            
            for line in result.lines() {
                let trimmed = line.trim();
                if trimmed.contains("State") && trimmed.contains("connected") {
                    connected = true;
                    break;
                }
            }
            
            if connected {
                (true, "ON".to_string())
            } else {
                (false, "OFF".to_string())
            }
        }
        Err(_) => (false, "N/A".to_string()),
    }
}

fn get_volume() -> (u8, bool) {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output();
    
    match output {
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
    
    if fs::metadata(&core_path).is_ok() {
        passed += 1;
    }
    
    let scripts_path = format!("{}/scripts", core_path);
    if let Ok(entries) = fs::read_dir(&scripts_path) {
        if entries.count() > 5 {
            passed += 1;
        }
    }
    
    let git_status = Command::new("git")
        .args(["-C", &core_path, "status", "--porcelain"])
        .output();
    if let Ok(out) = git_status {
        if out.stdout.is_empty() {
            passed += 1;
        }
    }
    
    let profile_path = format!("{}/.local/state/0-core/current-profile", home);
    if fs::metadata(&profile_path).is_ok() {
        passed += 1;
    }
    
    let version_path = format!("{}/VERSION", core_path);
    if fs::metadata(&version_path).is_ok() {
        passed += 1;
    }
    
    ((passed * 100) / total) as u8
}

fn is_core_locked() -> bool {
    let home = env::var("HOME").unwrap_or_default();
    let core_path = format!("{}/0-core", home);
    
    let output = Command::new("lsattr")
        .args(["-d", &core_path])
        .output();
    
    match output {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            if let Some(attrs) = result.split_whitespace().next() {
                attrs.contains('i')
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

fn sway_query(cmd: &str) -> Option<String> {
    let output = Command::new("swaymsg")
        .args(["-t", cmd, "-r"])
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

fn get_workspaces() -> (Vec<i32>, i32) {
    let mut workspaces: Vec<i32> = vec![];
    let mut active: i32 = 1;
    
    if let Some(resp) = sway_query("get_workspaces") {
        // Parse JSON for workspace numbers
        for num in 1..=10 {
            let pattern = format!("\"num\":{}", num);
            let pattern2 = format!("\"num\": {}", num);
            if resp.contains(&pattern) || resp.contains(&pattern2) {
                workspaces.push(num);
            }
        }
        
        // Find focused workspace
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

fn draw_text(font: &Font, canvas: &mut [u8], width: u32, text: &str, x: i32, y: i32, color: [u8; 4]) {
    let mut cursor_x = x;
    let font_size = 14.0;
    let baseline = y + 12;
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha == 0 {
                    continue;
                }
                
                let px = cursor_x + metrics.xmin as i32 + col as i32;
                let py = baseline - metrics.height as i32 - metrics.ymin as i32 + row as i32;
                
                if px >= 0 && px < width as i32 && py >= 0 && py < BAR_HEIGHT as i32 {
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

fn handle_click(action: &str) {
    match action {
        "vpn" => {
            let status = Command::new("mullvad").arg("status").output();
            if let Ok(out) = status {
                let result = String::from_utf8_lossy(&out.stdout);
                if result.contains("Connected") {
                    Command::new("mullvad").arg("disconnect").spawn().ok();
                } else {
                    Command::new("mullvad").arg("connect").spawn().ok();
                }
            }
        }
        "volume" => {
            Command::new("wpctl").args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"]).spawn().ok();
        }
        "profile" => {
            let current = get_current_profile();
            let next = match current.as_str() {
                "default" => "gaming",
                "gaming" => "work", 
                "work" => "low-power",
                _ => "default",
            };
            Command::new("profile").arg(next).spawn().ok();
        }
        _ => {}
    }
}

fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
    let qh = event_queue.handle();
    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");
    let seat_state = SeatState::new(&globals, &qh);
    let surface = compositor.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Top,
        Some("faelight-bar"),
        None,
    );
    layer_surface.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer_surface.set_size(0, BAR_HEIGHT);
    layer_surface.set_exclusive_zone(BAR_HEIGHT as i32);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer_surface.commit();
    let pool = SlotPool::new(4096 * BAR_HEIGHT as usize * 4, &shm).expect("Failed to create pool");
    let font = Font::from_bytes(FONT_DATA, FontSettings::default()).expect("Failed to load font");
    let mut state = BarState {
        registry_state: RegistryState::new(&globals),
        seat_state,
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer_surface,
        width: 0,
        height: BAR_HEIGHT,
        configured: false,
        running: true,
        font,
        click_regions: Vec::new(),
        pointer_x: 0.0,
    };
    println!("🌲 faelight-bar v0.7 starting (Sway Edition)...");
    while state.running {
        event_queue.blocking_dispatch(&mut state).expect("Event dispatch failed");
    }
}

struct BarState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer_surface: LayerSurface,
    width: u32,
    height: u32,
    configured: bool,
    running: bool,
    font: Font,
    click_regions: Vec<(i32, i32, String)>,
    pointer_x: f64,
}

impl BarState {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        if self.width == 0 {
            return;
        }
        self.click_regions.clear();
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;
        let (buffer, canvas) = self.pool
            .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
            .expect("Failed to create buffer");
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = BG_COLOR[0];
            pixel[1] = BG_COLOR[1];
            pixel[2] = BG_COLOR[2];
            pixel[3] = BG_COLOR[3];
        }
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
        // === LEFT SIDE ===
        let mut x_pos = 10;
        // Profile indicator (clickable)
        let profile_start = x_pos;
        let profile_icon = get_profile_icon(&profile);
        draw_text(&self.font, canvas, width, profile_icon, x_pos, 8, accent);
        x_pos += 40;
        self.click_regions.push((profile_start, x_pos, "profile".to_string()));
        draw_text(&self.font, canvas, width, "|", x_pos, 8, DIM_COLOR);
        x_pos += 15;
        let (workspaces, active) = get_workspaces();
        for ws in &workspaces {
            let color = if *ws == active { ACCENT_COLOR } else { DIM_COLOR };
            let ws_str = format!("{}", ws);
            draw_text(&self.font, canvas, width, &ws_str, x_pos, 8, color);
            x_pos += 18;
        }
        // Health & Lock
        x_pos += 10;
        draw_text(&self.font, canvas, width, "|", x_pos, 8, DIM_COLOR);
        x_pos += 15;
        
        let health = get_health();
        let health_color = if health >= 80 { ACCENT_COLOR } else if health >= 50 { AMBER_COLOR } else { RED_COLOR };
        let health_text = format!("{}%", health);
        draw_text(&self.font, canvas, width, &health_text, x_pos, 8, health_color);
        x_pos += 35;
        
        let locked = is_core_locked();
        let lock_color = if locked { ACCENT_COLOR } else { AMBER_COLOR };
        let lock_text = if locked { "LCK" } else { "UNL" };
        draw_text(&self.font, canvas, width, lock_text, x_pos, 8, lock_color);
        // === CENTER ===
        let window_title = get_active_window();
        if !window_title.is_empty() {
            let title_width = window_title.len() as i32 * 8;
            let center_x = (width as i32 / 2) - (title_width / 2);
            draw_text(&self.font, canvas, width, &window_title, center_x, 8, TEXT_COLOR);
        }
        // === RIGHT SIDE ===
        let mut rx = width as i32 - 110;
        let time_str = Local::now().format("%b %d %H:%M").to_string();
        draw_text(&self.font, canvas, width, &time_str, rx, 8, TEXT_COLOR);
        rx -= 15;
        draw_text(&self.font, canvas, width, "|", rx, 8, DIM_COLOR);
        // Volume (clickable)
        rx -= 40;
        let vol_start = rx;
        let (vol, muted) = get_volume();
        let vol_color = if muted { DIM_COLOR } else { TEXT_COLOR };
        let vol_text = if muted { "MUT".to_string() } else { format!("{}%", vol) };
        draw_text(&self.font, canvas, width, &vol_text, rx, 8, vol_color);
        self.click_regions.push((vol_start, vol_start + 35, "volume".to_string()));
        rx -= 15;
        draw_text(&self.font, canvas, width, "|", rx, 8, DIM_COLOR);
        rx -= 45;
        let (wifi_on, wifi_status) = get_wifi();
        let wifi_color = if wifi_on { BLUE_COLOR } else { DIM_COLOR };
        let wifi_text = format!("W:{}", wifi_status);
        draw_text(&self.font, canvas, width, &wifi_text, rx, 8, wifi_color);
        rx -= 15;
        draw_text(&self.font, canvas, width, "|", rx, 8, DIM_COLOR);
        rx -= 45;
        let (bat_pct, charging) = get_battery();
        let bat_color = if bat_pct < 20 { RED_COLOR } else if charging { BLUE_COLOR } else { TEXT_COLOR };
        let bat_text = format!("{}%{}", bat_pct, if charging { "+" } else { "" });
        draw_text(&self.font, canvas, width, &bat_text, rx, 8, bat_color);
        rx -= 15;
        draw_text(&self.font, canvas, width, "|", rx, 8, DIM_COLOR);
        // VPN (clickable)
        rx -= 60;
        let vpn_start = rx;
        let (vpn_connected, vpn_status) = get_vpn_status();
        let vpn_color = if vpn_connected { BLUE_COLOR } else { RED_COLOR };
        let vpn_text = format!("VPN:{}", vpn_status);
        draw_text(&self.font, canvas, width, &vpn_text, rx, 8, vpn_color);
        self.click_regions.push((vpn_start, vpn_start + 55, "vpn".to_string()));
        self.layer_surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer_surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        self.layer_surface.wl_surface().frame(qh, self.layer_surface.wl_surface().clone());
        self.layer_surface.commit();
    }
}

impl CompositorHandler for BarState {
    fn scale_factor_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _new_factor: i32) {}
    fn transform_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _new_transform: wl_output::Transform) {}
    fn frame(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _time: u32) {
        self.draw(qh);
    }
}

impl OutputHandler for BarState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
}

impl LayerShellHandler for BarState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.running = false;
    }
    fn configure(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, _layer: &LayerSurface, configure: LayerSurfaceConfigure, _serial: u32) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }
        self.configured = true;
        self.draw(qh);
    }
}

impl ShmHandler for BarState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SeatHandler for BarState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Pointer {
            self.seat_state.get_pointer(qh, &seat).ok();
        }
    }
    fn remove_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat, _capability: Capability) {}
    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
}

impl PointerHandler for BarState {
    fn pointer_frame(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _pointer: &wl_pointer::WlPointer, events: &[PointerEvent]) {
        for event in events {
            match event.kind {
                PointerEventKind::Motion { .. } => {
                    self.pointer_x = event.position.0;
                }
                PointerEventKind::Press { button, .. } => {
                    if button == 272 {
                        let x = self.pointer_x as i32;
                        for (start, end, action) in &self.click_regions {
                            if x >= *start && x <= *end {
                                handle_click(action);
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl ProvidesRegistryState for BarState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(BarState);
delegate_output!(BarState);
delegate_layer!(BarState);
delegate_shm!(BarState);
delegate_registry!(BarState);
delegate_seat!(BarState);
delegate_pointer!(BarState);
