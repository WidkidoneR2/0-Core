use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
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
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};
use fontdue::{Font, FontSettings};
use chrono::Local;
use std::os::unix::net::UnixStream;
use std::io::{Write, Read};
use std::env;

const BAR_HEIGHT: u32 = 32;
const BG_COLOR: [u8; 4] = [0x11, 0x14, 0x0f, 0xFF];
const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const ACCENT_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const DIM_COLOR: [u8; 4] = [0x77, 0x7f, 0x6f, 0xFF];

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/Adwaita/AdwaitaMono-Regular.ttf");

// Hyprland IPC
fn hyprland_query(cmd: &str) -> Option<String> {
    let his = env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    let xdg_runtime = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let socket_path = format!("{}/hypr/{}/.socket.sock", xdg_runtime, his);
    
    let mut stream = UnixStream::connect(&socket_path).ok()?;
    stream.write_all(cmd.as_bytes()).ok()?;
    
    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;
    Some(response)
}

fn get_workspaces() -> (Vec<i32>, i32) {
    let mut workspaces: Vec<i32> = vec![];
    let mut active: i32 = 1;
    
    // Get active workspace
    if let Some(resp) = hyprland_query("activeworkspace") {
        for line in resp.lines() {
            if line.starts_with("workspace ID") {
                // Format: "workspace ID 1 (1) on monitor..."
                if let Some(id_str) = line.split_whitespace().nth(2) {
                    active = id_str.parse().unwrap_or(1);
                }
                break;
            }
        }
    }
    
    // Get all workspaces
    if let Some(resp) = hyprland_query("workspaces") {
        for line in resp.lines() {
            if line.starts_with("workspace ID") {
                if let Some(id_str) = line.split_whitespace().nth(2) {
                    if let Ok(id) = id_str.parse::<i32>() {
                        if id > 0 && id <= 10 {
                            workspaces.push(id);
                        }
                    }
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
    if let Some(resp) = hyprland_query("activewindow") {
        for line in resp.lines() {
            if line.starts_with("title:") {
                let title = line.strip_prefix("title:").unwrap_or("").trim();
                // Truncate long titles
                if title.len() > 50 {
                    return format!("{}...", &title[..47]);
                }
                return title.to_string();
            }
        }
    }
    String::new()
}

fn draw_text(font: &Font, canvas: &mut [u8], width: u32, text: &str, x: i32, y: i32, color: [u8; 4]) {
    let mut cursor_x = x;
    let font_size = 16.0;
    let baseline = y + 14;

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

fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

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
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer_surface,
        width: 0,
        height: BAR_HEIGHT,
        configured: false,
        running: true,
        font,
    };

    println!("🌲 faelight-bar v0.2 starting...");

    while state.running {
        event_queue.blocking_dispatch(&mut state).expect("Event dispatch failed");
    }
}

struct BarState {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer_surface: LayerSurface,
    width: u32,
    height: u32,
    configured: bool,
    running: bool,
    font: Font,
}

impl BarState {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        if self.width == 0 {
            return;
        }

        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = self.pool
            .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
            .expect("Failed to create buffer");

        // Fill background
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = BG_COLOR[0];
            pixel[1] = BG_COLOR[1];
            pixel[2] = BG_COLOR[2];
            pixel[3] = BG_COLOR[3];
        }

        // Draw accent line at top
        for x in 0..width as usize {
            for y in 0..2 {
                let idx = (y * width as usize + x) * 4;
                if idx + 3 < canvas.len() {
                    canvas[idx] = ACCENT_COLOR[0];
                    canvas[idx + 1] = ACCENT_COLOR[1];
                    canvas[idx + 2] = ACCENT_COLOR[2];
                    canvas[idx + 3] = ACCENT_COLOR[3];
                }
            }
        }

        // LEFT: Workspaces
        let (workspaces, active) = get_workspaces();
        let mut x_pos = 15;
        for ws in &workspaces {
            let color = if *ws == active { ACCENT_COLOR } else { DIM_COLOR };
            let ws_str = format!("{}", ws);
            draw_text(&self.font, canvas, width, &ws_str, x_pos, 6, color);
            x_pos += 20;
        }

        // CENTER: Active window
        let window_title = get_active_window();
        if !window_title.is_empty() {
            let title_width = window_title.len() as i32 * 9; // approximate
            let center_x = (width as i32 / 2) - (title_width / 2);
            draw_text(&self.font, canvas, width, &window_title, center_x, 6, TEXT_COLOR);
        }

        // RIGHT: Time
        let time_str = Local::now().format("%H:%M").to_string();
        draw_text(&self.font, canvas, width, &time_str, width as i32 - 60, 6, TEXT_COLOR);

        // Attach buffer
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

impl ProvidesRegistryState for BarState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_compositor!(BarState);
delegate_output!(BarState);
delegate_layer!(BarState);
delegate_shm!(BarState);
delegate_registry!(BarState);
