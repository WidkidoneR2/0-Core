//! faelight-launcher v0.4 - Static List
//! 🌲 Faelight Forest

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler,
            LayerSurface, LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};
use fontdue::{Font, FontSettings};
use std::process::Command;

// ═══════════════════════════════════════════════════════════
// 🎨 FAELIGHT FOREST COLORS
// ═══════════════════════════════════════════════════════════
const WIDTH: u32 = 500;
const HEIGHT: u32 = 680;

const BG_COLOR: [u8; 4] = [0x14, 0x17, 0x11, 0xF8];
const BORDER_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const SELECTED_BG: [u8; 4] = [0x2a, 0x3a, 0x25, 0xFF];
const DIM_COLOR: [u8; 4] = [0x7f, 0x8f, 0x77, 0xFF];

// ═══════════════════════════════════════════════════════════
// 📐 TYPOGRAPHY & LAYOUT
// ═══════════════════════════════════════════════════════════
const FONT_TITLE: f32 = 24.0;
const FONT_SEARCH: f32 = 16.0;
const FONT_ITEM: f32 = 18.0;
const FONT_HINT: f32 = 16.0;
const ROW_HEIGHT: u32 = 46;
const ROW_START: u32 = 95;

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Bold.ttf");

// ═══════════════════════════════════════════════════════════
// 📱 APP ENTRIES
// ═══════════════════════════════════════════════════════════
struct AppEntry {
    name: &'static str,
    exec: &'static str,
    icon: &'static str,
}
const APPS: &[AppEntry] = &[
    AppEntry { name: "Browser", exec: "brave", icon: "󰖟" },
    AppEntry { name: "btop", exec: "foot -e btop", icon: "󰄪" },
    AppEntry { name: "Discord", exec: "discord", icon: "󰙯" },
    AppEntry { name: "Editor", exec: "foot -e nvim", icon: "" },
    AppEntry { name: "Filen", exec: "filen-desktop", icon: "󰅟" },
    AppEntry { name: "Files", exec: "thunar", icon: "󰉋" },
    AppEntry { name: "KeePassXC", exec: "keepassxc", icon: "󰌋" },
    AppEntry { name: "LazyGit", exec: "foot -e lazygit", icon: "󰊢" },
    AppEntry { name: "Notesnook", exec: "notesnook", icon: "󱞁" },
    AppEntry { name: "Terminal", exec: "foot", icon: "" },
    AppEntry { name: "Tutanota", exec: "tutanota-desktop", icon: "󰇮" },
    AppEntry { name: "Yazi", exec: "foot -e yazi", icon: "󰉖" },
];


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

fn fuzzy_match(query: &str, target: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let query = query.to_lowercase();
    let target = target.to_lowercase();
    
    // Simple substring match first
    if target.contains(&query) {
        return true;
    }
    
    // Fuzzy: all query chars must appear in order
    let mut query_chars = query.chars().peekable();
    for ch in target.chars() {
        if query_chars.peek() == Some(&ch) {
            query_chars.next();
        }
    }
    query_chars.peek().is_none()
}

fn filter_apps(query: &str) -> Vec<&'static AppEntry> {
    APPS.iter().filter(|app| fuzzy_match(query, app.name)).collect()
}
// ═══════════════════════════════════════════════════════════
// 🖼️ DRAWING HELPERS (standalone)
// ═══════════════════════════════════════════════════════════
fn draw_border(canvas: &mut [u8], width: u32, height: u32) {
    let stride = width as usize * 4;
    for x in 0..width as usize {
        canvas[x * 4..x * 4 + 4].copy_from_slice(&BORDER_COLOR);
        canvas[(height as usize - 1) * stride + x * 4..(height as usize - 1) * stride + x * 4 + 4].copy_from_slice(&BORDER_COLOR);
    }
    for y in 0..height as usize {
        canvas[y * stride..y * stride + 4].copy_from_slice(&BORDER_COLOR);
        canvas[y * stride + (width as usize - 1) * 4..y * stride + (width as usize - 1) * 4 + 4].copy_from_slice(&BORDER_COLOR);
    }
}

fn draw_rect(canvas: &mut [u8], width: u32, height: u32, x: u32, y: u32, w: u32, h: u32, color: [u8; 4]) {
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

fn draw_text(font: &Font, canvas: &mut [u8], width: u32, height: u32, text: &str, x: u32, y: u32, color: [u8; 4], size: f32) {
    let stride = width as usize * 4;
    let mut cursor_x = x as usize;
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, size);
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let alpha = bitmap[row * metrics.width + col];
                if alpha > 0 {
                    let px = cursor_x + col;
                    let py = y as usize + row;
                    if px < width as usize && py < height as usize {
                        let idx = py * stride + px * 4;
                        let a = alpha as f32 / 255.0;
                        canvas[idx] = ((1.0 - a) * canvas[idx] as f32 + a * color[0] as f32) as u8;
                        canvas[idx + 1] = ((1.0 - a) * canvas[idx + 1] as f32 + a * color[1] as f32) as u8;
                        canvas[idx + 2] = ((1.0 - a) * canvas[idx + 2] as f32 + a * color[2] as f32) as u8;
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

        let (buffer, canvas) = match pool.create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888) {
            Ok(b) => b,
            Err(_) => return,
        };

        // Clear background
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&BG_COLOR);
        }

        draw_border(canvas, width, height);
        draw_text(&self.font, canvas, width, height, "󰐅 Faelight Launcher", 20, 18, BORDER_COLOR, FONT_TITLE);
        
        // Search box
        let search_display = if self.search_query.is_empty() {
            "Type to search...".to_string()
        } else {
            self.search_query.clone()
        };
        let search_color = if self.search_query.is_empty() { DIM_COLOR } else { TEXT_COLOR };
        draw_rect(canvas, width, height, 15, 45, width - 30, 28, SELECTED_BG);
        draw_text(&self.font, canvas, width, height, &format!("🔍 {}", search_display), 20, 50, search_color, FONT_SEARCH);

        // Separator line
        let sep_y = 80;
        for x in 10..width as usize - 10 {
            let idx = sep_y * width as usize * 4 + x * 4;
            canvas[idx..idx + 4].copy_from_slice(&DIM_COLOR);
        }

        // Draw apps
        let filtered_apps = filter_apps(&self.search_query);
        for (i, app) in filtered_apps.iter().enumerate() {
            let y = ROW_START + i as u32 * ROW_HEIGHT;
            
            if i == selected {
                draw_rect(canvas, width, height, 10, y - 6, width - 20, 40, SELECTED_BG);
            }

            let color = if i == selected { BORDER_COLOR } else { TEXT_COLOR };
            let text = format!("{}  {}", app.icon, app.name);
            draw_text(&self.font, canvas, width, height, &text, 15, y, color, FONT_ITEM);
        }

        draw_text(&self.font, canvas, width, height, "↑↓ Navigate  Enter Launch  Esc Close", 20, height - 35, DIM_COLOR, FONT_HINT);

        if let Some(ref surface) = self.layer_surface {
            surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
            surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
            surface.wl_surface().commit();
        }
    }

    fn launch_selected(&self) {
        let filtered_apps = filter_apps(&self.search_query);
        if filtered_apps.is_empty() { return; }
        let app = filtered_apps[self.selected.min(filtered_apps.len() - 1)];
        eprintln!("🚀 Launching: {}", app.name);
        
        let parts: Vec<&str> = app.exec.split_whitespace().collect();
        if let Some((cmd, args)) = parts.split_first() {
            Command::new(cmd).args(args).spawn().ok();
        }
    }

    fn move_up(&mut self) {
        let filtered_len = filter_apps(&self.search_query).len();
        if filtered_len == 0 { return; }
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = filtered_len - 1;
        }
    }

    fn move_down(&mut self) {
        let filtered_len = filter_apps(&self.search_query).len();
        if filtered_len == 0 { return; }
        if self.selected < filtered_len - 1 {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🔧 HANDLERS
// ═══════════════════════════════════════════════════════════
impl CompositorHandler for LauncherState {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
}

impl OutputHandler for LauncherState {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for LauncherState {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.running = false;
    }
    fn configure(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface, configure: LayerSurfaceConfigure, _: u32) {
        self.width = configure.new_size.0.max(WIDTH);
        self.height = configure.new_size.1.max(HEIGHT);
        self.configured = true;
        self.draw();
    }
}

impl SeatHandler for LauncherState {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard {
            self.seat_state.get_keyboard(qh, &seat, None).ok();
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for LauncherState {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: Modifiers, _: u32) {}

    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        match event.keysym {
            Keysym::Escape => {
                    if !self.search_query.is_empty() {
                        self.search_query.clear();
                        self.selected = 0;
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
                self.draw();
            }
            _ => {
                if let Some(ch) = keysym_to_char(event.keysym) {
                    self.search_query.push(ch);
                    self.selected = 0;
                    self.draw();
                }
            }
        }
    }

    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn update_repeat_info(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: smithay_client_toolkit::seat::keyboard::RepeatInfo) {}
}

impl ShmHandler for LauncherState {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm }
}

impl ProvidesRegistryState for LauncherState {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
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
// 🚀 MAIN
// ═══════════════════════════════════════════════════════════
fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🌲 faelight-launcher v0.4 starting...");

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;

    let surface = compositor.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("faelight-launcher"), None);

    layer_surface.set_anchor(Anchor::empty());
    layer_surface.set_size(WIDTH, HEIGHT);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
    layer_surface.commit();

    let pool = SlotPool::new(WIDTH as usize * HEIGHT as usize * 4, &shm)?;
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
        running: true,
    };

    while state.running {
        event_queue.blocking_dispatch(&mut state)?;
    }

    eprintln!("👋 Goodbye!");
    Ok(())
}
