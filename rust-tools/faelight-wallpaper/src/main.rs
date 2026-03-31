//! faelight-wallpaper v0.1.0
//! 🌲 Rust wallpaper daemon — replaces swaybg
//! Uses wlr-layer-shell to render background on all outputs.
//! Health-reactive: color shifts subtly when forest health changes.

use anyhow::Result;
use clap::Parser;
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
use std::path::PathBuf;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

// ─── THEME ───────────────────────────────────────────────────────────────────
// Colors as ARGB u32 — same format as faelight-bar
// Format: 0xAARRGGBB → .to_le_bytes() → [B, G, R, A]
// wl_shm Argb8888 little-endian: bytes in memory are [B, G, R, A]
// So u32.to_le_bytes() = [byte0=B, byte1=G, byte2=R, byte3=A]
// To encode: 0xFF_RR_GG_BB where to_le_bytes gives [BB, GG, RR, FF] = [B,G,R,A] ✓
// Correct: RGB(0x0f, 0x14, 0x11) → need [B=0x11, G=0x14, R=0x0f, A=0xff]
// as u32 le: byte0=0x11 byte1=0x14 byte2=0x0f byte3=0xff → 0xff0f1411
const COLOR_HEALTHY: u32 = 0xFF0F1411; // [B=0x11,G=0x14,R=0x0F,A=0xFF]
const COLOR_WARN: u32 = 0xFF0A1314;
const COLOR_CRITICAL: u32 = 0xFF0C0C18;

// ─── CLI ─────────────────────────────────────────────────────────────────────
#[derive(Parser)]
#[command(
    name = "faelight-wallpaper",
    about = "🌲 Rust wallpaper daemon",
    version = "0.1.0"
)]
struct Cli {
    /// Hex color (e.g. #0f1411) — overrides health-reactive mode
    #[arg(short, long)]
    color: Option<String>,
    /// Disable health-reactive color shifting
    #[arg(long)]
    static_color: bool,
    /// Health check — print status and exit
    #[arg(long)]
    health: bool,
}

// ─── STATE ───────────────────────────────────────────────────────────────────
struct WallpaperState {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layer: LayerSurface,
    pool: SlotPool,
    width: u32,
    height: u32,
    configured: bool,
    color: u32,
    health_reactive: bool,
    last_health_check: std::time::Instant,
}

impl WallpaperState {
    fn draw(&mut self) {
        let width = self.width.max(1);
        let height = self.height.max(1);
        let stride = (width * 4) as i32;

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(_) => return,
        };

        let color = self.color.to_le_bytes();
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }

        self.layer
            .wl_surface()
            .attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        self.layer.wl_surface().commit();
    }

    fn update_health_color(&mut self) {
        if !self.health_reactive {
            return;
        }
        if self.last_health_check.elapsed() < std::time::Duration::from_secs(30) {
            return;
        }
        self.last_health_check = std::time::Instant::now();

        let health = read_health_from_db();
        self.color = match health {
            h if h >= 95 => COLOR_HEALTHY,
            h if h >= 80 => COLOR_WARN,
            _ => COLOR_CRITICAL,
        };
    }
}

fn read_health_from_db() -> u32 {
    let db_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home/christian"))
        .join("0-core/runtime/state.db");

    if !db_path.exists() {
        return 95;
    }

    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return 95,
    };

    let result: rusqlite::Result<String> = conn.query_row(
        "SELECT payload FROM events WHERE domain='doctor' ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get(0),
    );

    result
        .ok()
        .and_then(|p| serde_json::from_str::<serde_json::Value>(&p).ok())
        .and_then(|v| v["detail"]["health"].as_u64())
        .map(|h| h as u32)
        .unwrap_or(95)
}

fn parse_hex_color(hex: &str) -> u32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0x0f) as u32;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0x14) as u32;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0x11) as u32;
        // Encode as LE u32: [B, G, R, A] bytes → b | (g<<8) | (r<<16) | (0xFF<<24)
        0xFF000000 | (r << 16) | (g << 8) | b
    } else {
        COLOR_HEALTHY
    }
}

// ─── HANDLERS ────────────────────────────────────────────────────────────────
impl LayerShellHandler for WallpaperState {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        std::process::exit(0);
    }
    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        let (w, h) = configure.new_size;
        if w > 0 {
            self.width = w;
        }
        if h > 0 {
            self.height = h;
        }
        self.configured = true;
        self.draw();
    }
}

impl CompositorHandler for WallpaperState {
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

impl OutputHandler for WallpaperState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl ShmHandler for WallpaperState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for WallpaperState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_compositor!(WallpaperState);
delegate_layer!(WallpaperState);
delegate_output!(WallpaperState);
delegate_registry!(WallpaperState);
delegate_shm!(WallpaperState);

// ─── MAIN ────────────────────────────────────────────────────────────────────
fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.health {
        println!("faelight-wallpaper v0.1.0 — healthy");
        return Ok(());
    }

    let initial_color = if let Some(ref hex) = cli.color {
        parse_hex_color(hex)
    } else {
        let health = read_health_from_db();
        match health {
            h if h >= 95 => COLOR_HEALTHY,
            h if h >= 80 => COLOR_WARN,
            _ => COLOR_CRITICAL,
        }
    };

    let conn = Connection::connect_to_env()?;
    let (globals, mut queue) = registry_queue_init::<WallpaperState>(&conn)?;
    let qh = queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;

    let surface = compositor.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Background,
        Some("faelight-wallpaper"),
        None,
    );

    // Cover entire screen
    layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
    layer.set_size(0, 0);
    layer.set_exclusive_zone(-1);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.commit();

    let pool = SlotPool::new(3840 * 2160 * 4, &shm)?;

    let mut state = WallpaperState {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        layer,
        pool,
        width: 1920,
        height: 1080,
        configured: false,
        color: initial_color,
        health_reactive: !cli.static_color && cli.color.is_none(),
        last_health_check: std::time::Instant::now(),
    };

    loop {
        queue.blocking_dispatch(&mut state)?;
        state.update_health_color();
        if state.configured {
            state.draw();
        }
    }
}
