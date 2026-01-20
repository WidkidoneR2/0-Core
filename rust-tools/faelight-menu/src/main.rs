//! faelight-menu v0.7.0 - Smart Power Menu
//! 🌲 Faelight Forest
//! 
//! Features:
//! - Red letters (not background) for dangerous actions
//! - Power state awareness (battery %)
//! - Smart defaults by time of day
//! - Minimal UI: arrow + color only (no background bars)
//! - Graceful shutdown with service cleanup and sync

use chrono::{Local, Timelike};
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
const DIM_COLOR: [u8; 4] = [0x7f, 0x8f, 0x77, 0xFF];
const WARN_COLOR: [u8; 4] = [0xe3, 0xc5, 0x6b, 0xFF];
const DANGER_COLOR: [u8; 4] = [0xe3, 0x6b, 0x6b, 0xFF];
const DANGER_DIM: [u8; 4] = [0xb3, 0x5b, 0x5b, 0xFF];

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Bold.ttf");

// ═══════════════════════════════════════════════════════════
// 📐 TYPOGRAPHY & LAYOUT
// ═══════════════════════════════════════════════════════════
const FONT_TITLE: f32 = 20.0;
const FONT_ITEM: f32 = 18.0;
const FONT_HINT: f32 = 14.0;
const ROW_HEIGHT: u32 = 44;
const ROW_START: u32 = 75;

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
// ⚡ POWER STATE HELPERS
// ═══════════════════════════════════════════════════════════

fn get_power_state() -> (Option<u8>, bool) {
    let capacity_path = "/sys/class/power_supply/BAT0/capacity";
    let status_path = "/sys/class/power_supply/BAT0/status";
    
    let capacity = std::fs::read_to_string(capacity_path)
        .ok()
        .and_then(|s| s.trim().parse().ok());
    
    let on_battery = std::fs::read_to_string(status_path)
        .map(|s| !s.contains("Charging"))
        .unwrap_or(false);
    
    (capacity, on_battery)
}

fn get_smart_default() -> usize {
    let hour = Local::now().hour();
    match hour {
        23..=24 | 0..=6 => 4,
        _ => 0,
    }
}

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
    battery_percent: Option<u8>,
    on_battery: bool,
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
        
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&BG_COLOR);
        }
        
        draw_border(canvas, width, height);
        
        draw_text(
            &mut self.glyph_cache,
            canvas,
            width,
            height,
            "⚡ Power Menu",
            20,
            20,
            BORDER_COLOR,
            FONT_TITLE,
        );
        
        for x in 15..width as usize - 15 {
            let idx = 55 * width as usize * 4 + x * 4;
            canvas[idx..idx + 4].copy_from_slice(&DIM_COLOR);
        }
        
        for (i, item) in MENU_ITEMS.iter().enumerate() {
            let y = ROW_START + i as u32 * ROW_HEIGHT;
            
            if i == 3 {
                let div_y = y - 18;
                for x in 20..width as usize - 20 {
                    let idx = div_y as usize * width as usize * 4 + x * 4;
                    if idx + 4 <= canvas.len() {
                        canvas[idx..idx + 4].copy_from_slice(&DIM_COLOR);
                    }
                }
            }
            
            let text_color = if item.dangerous {
                if i == selected {
                    if self.confirming {
                        DANGER_COLOR
                    } else {
                        WARN_COLOR
                    }
                } else {
                    DANGER_DIM
                }
            } else {
                if i == selected {
                    BORDER_COLOR
                } else {
                    DIM_COLOR
                }
            };
            
            let text = if self.confirming && i == selected && item.dangerous {
                format!("▶ {}  {} [CONFIRM]", item.icon, item.label)
            } else if i == selected {
                format!("▶ {}  {}", item.icon, item.label)
            } else {
                format!("  {}  {}", item.icon, item.label)
            };
            
            draw_text(
                &mut self.glyph_cache,
                canvas,
                width,
                height,
                &text,
                25,
                y,
                text_color,
                FONT_ITEM,
            );
        }
        
        if let Some(percent) = self.battery_percent {
            if self.on_battery {
                let icon = if percent < 20 { "⚠️" } else { "🔋" };
                let text = format!("Battery: {}% {}", percent, icon);
                draw_text(
                    &mut self.glyph_cache,
                    canvas,
                    width,
                    height,
                    &text,
                    15,
                    height - 50,
                    if percent < 20 { WARN_COLOR } else { DIM_COLOR },
                    FONT_HINT,
                );
            }
        }
        
        draw_text(
            &mut self.glyph_cache,
            canvas,
            width,
            height,
            "↑↓ or L/E/S/R/P  Enter Select  Esc Close",
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
        
        // Get home directory for script paths
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
        
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
                // Use graceful reboot script
                let script = format!("{}/0-core/scripts/graceful-reboot", home);
                Command::new(&script).spawn().ok();
            }
            "shutdown" => {
                // Use graceful poweroff script
                let script = format!("{}/0-core/scripts/graceful-poweroff", home);
                Command::new(&script).spawn().ok();
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
// 🔧 HANDLERS (unchanged)
// ═══════════════════════════════════════════════════════════

impl CompositorHandler for MenuState {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
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
    fn configure(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface, configure: LayerSurfaceConfigure, _: u32) {
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
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard {
            self.seat_state.get_keyboard(qh, &seat, None).ok();
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for MenuState {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: Modifiers, _: u32) {}
    
    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
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
    
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn update_repeat_info(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: RepeatInfo) {}
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
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("faelight-menu v0.7.0");
                std::process::exit(0);
            }
            "--help" | "-h" => {
                println!("faelight-menu v0.7.0 - Smart Power Menu for Faelight Forest");
                println!();
                println!("FEATURES:");
                println!("  • Minimal UI: arrow + color (no background bars)");
                println!("  • Red letters for dangerous actions");
                println!("  • Battery awareness when unplugged");
                println!("  • Smart defaults by time (shutdown at night, lock during day)");
                println!("  • Graceful shutdown (stops services, syncs filesystems, logs events)");
                println!();
                println!("USAGE:");
                println!("    faelight-menu");
                println!();
                println!("KEYBOARD SHORTCUTS:");
                println!("    ↑↓       Navigate menu");
                println!("    L        Lock screen");
                println!("    E        Logout");
                println!("    S        Suspend");
                println!("    R        Reboot (requires confirmation)");
                println!("    P        Shutdown (requires confirmation)");
                println!("    Enter    Select/Confirm");
                println!("    Esc      Close menu");
                println!();
                println!("OPTIONS:");
                println!("    -h, --help          Show this help");
                println!("    -v, --version       Show version");
                println!("    --health-check      Verify system dependencies");
                std::process::exit(0);
            }
            "--health-check" => {
                health_check();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Try 'faelight-menu --help' for usage information");
                std::process::exit(1);
            }
        }
    }
    
    eprintln!("⚡ faelight-menu v0.7.0 starting...");
    
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();
    
    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    
    let surface = compositor.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("faelight-menu"), None);
    
    layer_surface.set_anchor(Anchor::empty());
    layer_surface.set_size(WIDTH, HEIGHT);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
    layer_surface.commit();
    
    let pool = SlotPool::new(WIDTH as usize * HEIGHT as usize * 4, &shm)?;
    let glyph_cache = GlyphCache::new(FONT_DATA)?;
    
    let (battery_percent, on_battery) = get_power_state();
    let smart_default = get_smart_default();
    
    eprintln!("🔋 Power state: battery={:?}, on_battery={}", battery_percent, on_battery);
    eprintln!("🎯 Smart default: {} ({})", smart_default, MENU_ITEMS[smart_default].label);
    
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
        selected: smart_default,
        battery_percent,
        on_battery,
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
    println!("🏥 faelight-menu v0.7.0 health check");
    
    match Connection::connect_to_env() {
        Ok(_) => println!("✅ wayland: connected"),
        Err(e) => {
            eprintln!("❌ wayland: connection failed - {}", e);
            std::process::exit(1);
        }
    }
    
    match GlyphCache::new(FONT_DATA) {
        Ok(_) => println!("✅ font: loaded successfully"),
        Err(e) => {
            eprintln!("❌ font: failed to load - {}", e);
            std::process::exit(1);
        }
    }
    
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
    
    // Check graceful shutdown scripts
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
    let scripts = ["graceful-poweroff", "graceful-reboot"];
    for script in &scripts {
        let path = format!("{}/0-core/scripts/{}", home, script);
        if std::path::Path::new(&path).exists() {
            println!("✅ {}: installed", script);
        } else {
            eprintln!("⚠️  {}: not found", script);
        }
    }
    
    let (battery_percent, on_battery) = get_power_state();
    match battery_percent {
        Some(pct) => println!("✅ battery: {}% (on_battery: {})", pct, on_battery),
        None => println!("ℹ️  battery: not detected (desktop/AC only?)"),
    }
    
    let smart_default = get_smart_default();
    let hour = Local::now().hour();
    println!("✅ smart_default: {} at hour {} ({})", 
        smart_default, 
        hour,
        MENU_ITEMS[smart_default].label
    );
    
    println!("\n✅ All checks passed!");
}
