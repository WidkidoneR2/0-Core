//! faelight-menu v0.5.0 - Clarity & Safety
//! 🌲 Faelight Forest

use faelight_core::GlyphCache;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RepeatInfo},
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

// ═══════════════════════════════════════════════════════════
// 🎨 FAELIGHT FOREST COLORS
// ═══════════════════════════════════════════════════════════
const WIDTH: u32 = 345;
const HEIGHT: u32 = 408;

const BG_COLOR: [u8; 4] = [0x14, 0x17, 0x11, 0xF8];
const BORDER_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const SELECTED_BG: [u8; 4] = [0x2a, 0x3a, 0x25, 0xFF];
const DIM_COLOR: [u8; 4] = [0x7f, 0x8f, 0x77, 0xFF];
const WARN_COLOR: [u8; 4] = [0xe3, 0x6b, 0x6b, 0xFF];
const DANGER_COLOR: [u8; 4] = [0x6b, 0x6b, 0xe3, 0xFF]; // RED in ARGB8888 format // RED for dangerous confirmation

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Bold.ttf");

// ═══════════════════════════════════════════════════════════
// 📐 TYPOGRAPHY & LAYOUT
// ═══════════════════════════════════════════════════════════
const FONT_TITLE: f32 = 20.0;
const FONT_ITEM: f32 = 18.0;
const FONT_HINT: f32 = 14.0;
const ROW_HEIGHT: u32 = 44;
const ROW_START: u32 = 75;
const DANGER_DIM: [u8; 4] = [0xb3, 0x5b, 0x5b, 0xFF]; // Desaturated warn - always visible
                                                      //

// ═══════════════════════════════════════════════════════════
// 📱 MENU ENTRIES
// ═══════════════════════════════════════════════════════════
struct MenuItem {
    icon: &'static str,
    label: &'static str,
    action: &'static str,
    dangerous: bool,
}

const MENU_ITEMS: &[MenuItem] = &[
    MenuItem {
        icon: "󰌾",
        label: "Lock",
        action: "lock",
        dangerous: false,
    },
    MenuItem {
        icon: "󰗽",
        label: "Logout",
        action: "logout",
        dangerous: false,
    },
    MenuItem {
        icon: "󰤄",
        label: "Suspend",
        action: "suspend",
        dangerous: false,
    },
    MenuItem {
        icon: "󰜉",
        label: "Reboot",
        action: "reboot",
        dangerous: true,
    },
    MenuItem {
        icon: "󰐥",
        label: "Shutdown",
        action: "shutdown",
        dangerous: true,
    },
];

// ═══════════════════════════════════════════════════════════
// 🖼️ DRAWING HELPERS
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
    cache: &mut GlyphCache,
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
    for ch in text.chars() {
        let glyph = cache.rasterize(ch, size);
        let metrics = &glyph.metrics;
        let bitmap = &glyph.bitmap;
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
struct MenuState {
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
    glyph_cache: GlyphCache,
    selected: usize,

    confirming: bool,
    running: bool,
}

impl MenuState {
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
        draw_text(&mut self.glyph_cache,
            canvas,
            width,
            height,
            "⚡ Power Menu",
            20,
            20,
            BORDER_COLOR,
            FONT_TITLE,
        );

        // Separator
        for x in 15..width as usize - 15 {
            let idx = 55 * width as usize * 4 + x * 4;
            canvas[idx..idx + 4].copy_from_slice(&DIM_COLOR);
        }

        // Draw menu items
        for (i, item) in MENU_ITEMS.iter().enumerate() {
            let y = ROW_START + i as u32 * ROW_HEIGHT;

            // Divider before dangerous actions
            if i == 3 {
                let div_y = y - 18;
                for x in 20..width as usize - 20 {
                    let idx = div_y as usize * width as usize * 4 + x * 4;
                    if idx + 4 <= canvas.len() {
                        canvas[idx..idx + 4].copy_from_slice(&DIM_COLOR);
                    }
                }
            }

            if i == selected {
                let bg_color = if self.confirming && item.dangerous {
                    DANGER_COLOR
                } else {
                    SELECTED_BG
                };
                draw_rect(
                    canvas,
                    width,
                    height,
                    10,
                    y - 8,
                    width - 20,
                    46,
                    bg_color,
                );
            }

            let text_color = if self.confirming && i == selected && item.dangerous {
                TEXT_COLOR
            } else if item.dangerous {
                if i == selected {
                    WARN_COLOR
                } else {
                    DANGER_DIM
                }
            } else {
                if i == selected {
                    BORDER_COLOR
                } else {
                    TEXT_COLOR
                }
            };

            let text = if self.confirming && i == selected && item.dangerous {
                format!("{}  {} - Press Enter to CONFIRM", item.icon, item.label)
            } else {
                format!("{}  {}", item.icon, item.label)
            };
            draw_text(&mut self.glyph_cache, canvas, width, height, &text, 25, y, text_color, FONT_ITEM,
            );
        }

        // Hint
        draw_text(&mut self.glyph_cache,
            canvas,
            width,
            height,
            "↑↓ Move  Enter Select  Esc Close",
            15,
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

    fn execute_selected(&self) {
        let item = &MENU_ITEMS[self.selected];
        eprintln!("⚡ Executing: {}", item.label);

        match item.action {
            "lock" => {
                Command::new("swaylock").spawn().ok();
            }
            "logout" => {
                Command::new("swaymsg").arg("exit").spawn().ok();
            }
            "suspend" => {
                Command::new("systemctl").arg("suspend").spawn().ok();
            }
            "reboot" => {
                Command::new("systemctl").arg("reboot").spawn().ok();
            }
            "shutdown" => {
                Command::new("systemctl").arg("poweroff").spawn().ok();
            }
            _ => {}
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = MENU_ITEMS.len() - 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected < MENU_ITEMS.len() - 1 {
            self.selected += 1;
        } else {
            self.selected = 0;
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🔧 HANDLERS
// ═══════════════════════════════════════════════════════════
impl CompositorHandler for MenuState {
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

impl OutputHandler for MenuState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for MenuState {
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

impl SeatHandler for MenuState {
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

impl KeyboardHandler for MenuState {
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
            Keysym::Escape | Keysym::q => {
                if self.confirming {
                    self.confirming = false;
                    self.draw();
                } else {
                    self.running = false;
                }
            }
            Keysym::Return | Keysym::KP_Enter => {
                let item = &MENU_ITEMS[self.selected];
                if item.dangerous {
                    if self.confirming {
                        self.execute_selected();
                        self.running = false;
                    } else {
                        self.confirming = true;
                        self.draw();
                    }
                } else {
                    self.execute_selected();
                    self.running = false;
                }
            }
            Keysym::Up | Keysym::k => {
                self.confirming = false;
                self.move_up();
                self.draw();
            }
            Keysym::Down | Keysym::j => {
                self.confirming = false;
                self.move_down();
                self.draw();
            }
            // Quick keys
            Keysym::l => {
                self.selected = 0;
                self.execute_selected();
                self.running = false;
            }
            Keysym::e => {
                self.selected = 1;
                self.execute_selected();
                self.running = false;
            }
            Keysym::s => {
                self.selected = 2;
                self.execute_selected();
                self.running = false;
            }
            Keysym::r => {
                self.selected = 3;
                if self.confirming && self.selected == 3 {
                    self.execute_selected();
                    self.running = false;
                } else {
                    self.confirming = true;
                    self.draw();
                }
            }
            Keysym::p => {
                self.selected = 4;
                if self.confirming && self.selected == 4 {
                    self.execute_selected();
                    self.running = false;
                } else {
                    self.confirming = true;
                    self.draw();
                }
            }
            _ => {}
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
        _: RepeatInfo,
    ) {
    }
}

impl ShmHandler for MenuState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for MenuState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(MenuState);
delegate_output!(MenuState);
delegate_shm!(MenuState);
delegate_seat!(MenuState);
delegate_keyboard!(MenuState);
delegate_layer!(MenuState);
delegate_registry!(MenuState);

// ═══════════════════════════════════════════════════════════
// 🚀 MAIN
// ═══════════════════════════════════════════════════════════
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "--health-check") {
        health_check();
        std::process::exit(0);
    }

    eprintln!("⚡ faelight-menu v0.5 starting...");

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;

    let surface = compositor.create_surface(&qh);
    let layer_surface =
        layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("faelight-menu"), None);

    layer_surface.set_anchor(Anchor::empty());
    layer_surface.set_size(WIDTH, HEIGHT);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
    layer_surface.commit();

    let pool = SlotPool::new(WIDTH as usize * HEIGHT as usize * 4, &shm)?;
    let glyph_cache = GlyphCache::new(FONT_DATA)?;

    let mut state = MenuState {
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
        glyph_cache,

        confirming: false,
        selected: 0,
        running: true,
    };

    while state.running {
        event_queue.blocking_dispatch(&mut state)?;
    }

    eprintln!("👋 Goodbye!");
    Ok(())
}



// ═══════════════════════════════════════════════════════════
// 🏥 HEALTH CHECK
// ═══════════════════════════════════════════════════════════

fn health_check() {
    println!("🏥 faelight-menu health check");
    
    // Check Wayland connection
    match Connection::connect_to_env() {
        Ok(_) => println!("✅ wayland: connected"),
        Err(e) => {
            eprintln!("❌ wayland: connection failed - {}", e);
            std::process::exit(1);
        }
    }
    
    // Check if we can load font
    match GlyphCache::new(FONT_DATA) {
        Ok(_) => println!("✅ font: loaded successfully"),
        Err(e) => {
            eprintln!("❌ font: failed to load - {}", e);
            std::process::exit(1);
        }
    }
    
    // Check if commands exist
    let commands = ["swaylock", "swaymsg", "systemctl"];
    for cmd in &commands {
        if let Ok(check) = std::process::Command::new("which").arg(cmd).output() {
            if check.status.success() {
                println!("✅ {}: installed", cmd);
            } else {
                eprintln!("⚠️  {}: not found", cmd);
            }
        }
    }
    
    println!("\n✅ Core checks passed!");
}
