//! faelight-bar v3.0.0 -- forest status bar
//! Layer: smithay-client-toolkit (wlr-layer-shell)
//! Text:  cosmic-text + swash (same library faelight-term uses internally)
//! Pixel: SHM ARGB8888 -- simple, fast, beautiful

use futures_util::StreamExt as _;
use cosmic_text::{Attrs, Buffer, Color as TColor, Family, FontSystem, Metrics, Shaping, SwashCache};
use rusqlite::Connection as Db;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler,
            LayerSurface, LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::time::{Duration, Instant};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::wp::fractional_scale::v1::client::{
    wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
    wp_fractional_scale_v1::{self, WpFractionalScaleV1},
};
use wayland_protocols::wp::viewporter::client::{
    wp_viewport::WpViewport, wp_viewporter::WpViewporter,
};

// -- Constants ----------------------------------------------------------------

const BAR_HEIGHT: u32 = 30;
const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 22.0;
const UPDATE_MS: u64 = 1000;
const PAD: f32 = 12.0;
const FONT_FAMILY: &str = "JetBrainsMono Nerd Font";
const IC_CPU: &str = "";
const IC_RAM: &str = "󰍛";
const IC_WIFI: &str = "󰖩";
const IC_WIFI_OFF: &str = "󰖪";
const IC_BATT: &str = "󰁹";
const IC_CHARGE: &str = "󰂄";
const IC_CLOCK: &str = "󰥔";
const IC_HEART: &str = "";
const IC_GIT: &str = "";

// Background: #11140F forest green (ARGB little-endian bytes = B G R A)
const SEP_L: &str = "";
const SEP_R: &str = "";
const BG: [u8; 4] = [0x18, 0x24, 0x1B, 0xFF]; // #1B2418 forest green

// Text colors (RGBA for cosmic-text)
// INT-033: neon candy palette -- matches theme.rs semantic tokens
fn dim()    -> TColor { TColor::rgba(0x78, 0x8C, 0x82, 0xFF) } // muted gray
fn text()   -> TColor { TColor::rgba(0xD7, 0xE0, 0xDA, 0xFF) } // fog white
fn cyan()   -> TColor { TColor::rgba(0x32, 0xDC, 0xFF, 0xFF) } // neon cyan  (50, 220, 255)
fn green()  -> TColor { TColor::rgba(0x39, 0xFF, 0x14, 0xFF) } // neon green (57, 255, 20)
fn amber()  -> TColor { TColor::rgba(0xFF, 0xC8, 0x32, 0xFF) } // neon amber (255, 200, 50)
fn red()    -> TColor { TColor::rgba(0xFF, 0x50, 0x50, 0xFF) } // neon red   (255, 80, 80)
fn purple() -> TColor { TColor::rgba(0xB4, 0x82, 0xFF, 0xFF) } // neon purple (180, 130, 255)

// -- Forest State -------------------------------------------------------------

#[derive(Default, Clone)]
struct ForestState {
    health: u8,
    intent_title: String,
    friday: Option<(String, f64)>,
    git_branch: String,
    git_clean: bool,
    battery: Option<u8>,
    charging: bool,
    wifi_connected: bool,
    clock: String,
    cpu: u8,
    ram: u8,
}

impl ForestState {
    fn refresh() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let health: u8 = std::fs::read_to_string("/etc/faelight/HEALTH")
            .ok().and_then(|s| s.trim().trim_end_matches('%').parse().ok())
            .or_else(|| std::fs::read_to_string(
                format!("{}/.cache/faelight/health-status", home))
                .ok().and_then(|s| s.trim().parse().ok()))
            .unwrap_or(100);
        let intent_title = std::fs::read_to_string("/etc/faelight/INTENT")
            .unwrap_or_default().trim().to_string();
        let intent_title = if intent_title.len() > 55 {
            format!("{}...", &intent_title[..52])
        } else { intent_title };
        let friday = read_friday(&format!("{}/0-core/runtime/state.db", home));
        let go = |args: &[&str]| {
            std::process::Command::new("git").args(args)
                .current_dir(format!("{}/0-core", home)).output().ok()
        };
        let git_clean = go(&["status","--porcelain"])
            .map(|o| o.stdout.is_empty()).unwrap_or(true);
        let git_branch = go(&["branch","--show-current"])
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "main".into());
        let bat = "/sys/class/power_supply/BAT1";
        let battery: Option<u8> = std::fs::read_to_string(format!("{}/capacity", bat))
            .ok().and_then(|s| s.trim().parse().ok());
        let charging = std::fs::read_to_string(format!("{}/status", bat))
            .map(|s| s.trim() == "Charging").unwrap_or(false);
        let wifi_connected = std::fs::read_dir("/sys/class/net").ok()
            .and_then(|entries| entries.flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with("wl"))
                .find_map(|e| {
                    let name = e.file_name();
                    std::fs::read_to_string(
                        format!("/sys/class/net/{}/operstate", name.to_string_lossy()))
                        .ok().map(|s| s.trim() == "up")
                }))
            .unwrap_or(false);
        let clock = chrono::Local::now().format("%a %d  %H:%M").to_string();
        let cpu = read_cpu();
        let ram = read_ram();
        ForestState { health, intent_title, friday,
            git_branch, git_clean, battery, charging, wifi_connected, clock, cpu, ram }
    }
}

fn read_cpu() -> u8 {
    static PREV: std::sync::Mutex<Option<(u64, u64)>> = std::sync::Mutex::new(None);
    let stat = match std::fs::read_to_string("/proc/stat") { Ok(s) => s, Err(_) => return 0 };
    let line = match stat.lines().next() { Some(l) => l, None => return 0 };
    let vals: Vec<u64> = line.split_whitespace().skip(1).filter_map(|v| v.parse().ok()).collect();
    if vals.len() < 4 { return 0; }
    let idle = vals[3] + vals.get(4).copied().unwrap_or(0);
    let total: u64 = vals.iter().sum();
    let mut prev = match PREV.lock() { Ok(p) => p, Err(_) => return 0 };
    let pct = if let Some((pt, pi)) = *prev {
        let dt = total.saturating_sub(pt);
        let di = idle.saturating_sub(pi);
        if dt > 0 { ((dt.saturating_sub(di) as f64 / dt as f64) * 100.0).round() as u8 } else { 0 }
    } else { 0 };
    *prev = Some((total, idle));
    pct.min(100)
}
fn read_ram() -> u8 {
    let mem = match std::fs::read_to_string("/proc/meminfo") { Ok(s) => s, Err(_) => return 0 };
    let (mut total, mut avail) = (0u64, 0u64);
    for line in mem.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total = v.split_whitespace().next().and_then(|x| x.parse().ok()).unwrap_or(0);
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            avail = v.split_whitespace().next().and_then(|x| x.parse().ok()).unwrap_or(0);
        }
    }
    if total == 0 { return 0; }
    ((total.saturating_sub(avail) as f64 / total as f64) * 100.0).round() as u8
}
fn read_friday(db: &str) -> Option<(String, f64)> {
    let conn = Db::open(db).ok()?;
    let cutoff = chrono::Utc::now().timestamp() - 300;
    conn.query_row(
        "SELECT action, confidence FROM friday_patterns
         WHERE confidence >= 0.75 AND last_seen > ?1
         ORDER BY confidence DESC LIMIT 1",
        rusqlite::params![cutoff],
        |r| Ok((r.get::<_,String>(0)?, r.get::<_,f64>(1)?)),
    ).ok()
}

// -- Pixel rendering ----------------------------------------------------------

/// Blend a cosmic-text pixel into the ARGB8888 SHM canvas.
/// Wayland ARGB8888 in little-endian memory: [B, G, R, A]
#[inline]
fn blend(canvas: &mut [u8], px: i32, py: i32, w: u32, h: u32, color: TColor) {
    let a = color.a() as u32;
    if a == 0 { return; }
    let cw = unsafe { CANVAS_W };
    let ch = unsafe { CANVAS_H };
    for dy in 0..h as i32 {
        for dx in 0..w as i32 {
            let x = px + dx;
            let y = py + dy;
            if x < 0 || y < 0 || x >= cw || y >= ch { continue; }
            let i = (y as usize * cw as usize + x as usize) * 4;
            if i + 3 >= canvas.len() { continue; }
            if a == 255 {
                canvas[i]   = color.b();
                canvas[i+1] = color.g();
                canvas[i+2] = color.r();
                canvas[i+3] = 255;
            } else {
                let inv = 255 - a;
                canvas[i]   = ((canvas[i]   as u32 * inv + color.b() as u32 * a) / 255) as u8;
                canvas[i+1] = ((canvas[i+1] as u32 * inv + color.g() as u32 * a) / 255) as u8;
                canvas[i+2] = ((canvas[i+2] as u32 * inv + color.r() as u32 * a) / 255) as u8;
                canvas[i+3] = 255;
            }
        }
    }
}

// Thread-local canvas dimensions for blend() helper
static mut CANVAS_W: i32 = 1920;
static mut CANVAS_H: i32 = 28;

/// Draw text into SHM canvas at (x_off, y_off) with given color.
fn draw_text(
    canvas: &mut [u8],
    font_system: &mut FontSystem,
    swash: &mut SwashCache,
    text: &str,
    x_off: i32,
    y_off: i32,
    max_w: f32,
    color: TColor,
) {
    let mut buf = Buffer::new(font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
    buf.set_size(font_system, Some(max_w), Some(LINE_HEIGHT + 4.0));
    buf.set_text(font_system, text,
        Attrs::new().family(Family::Name(FONT_FAMILY)), Shaping::Basic);
    buf.shape_until_scroll(font_system, false);
    buf.draw(font_system, swash, color, |x, y, w, h, c| {
        blend(canvas, x_off + x, y_off + y, w, h, c);
    });
}

/// Measure rendered text width.
fn measure_text(font_system: &mut FontSystem, text: &str, max_w: f32) -> f32 {
    let mut buf = Buffer::new(font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
    buf.set_size(font_system, Some(max_w), Some(LINE_HEIGHT + 4.0));
    buf.set_text(font_system, text, Attrs::new().family(Family::Name(FONT_FAMILY)), Shaping::Basic);
    buf.shape_until_scroll(font_system, false);
    buf.layout_runs().map(|r| r.line_w).fold(0.0f32, f32::max)
}

/// Main draw: fill background + render three zones.
fn draw_frame(
    canvas: &mut [u8],
    phys_w: u32,
    phys_h: u32,
    font_system: &mut FontSystem,
    swash: &mut SwashCache,
    forest: &ForestState,
) {
    // Set canvas dimensions for blend helper
    unsafe { CANVAS_W = phys_w as i32; CANVAS_H = phys_h as i32; }

    // Fill background
    for pixel in canvas.chunks_exact_mut(4) {
        pixel.copy_from_slice(&BG);
    }

    let third  = phys_w as f32 / 3.0;
    let y_text = ((phys_h as f32 - LINE_HEIGHT) / 2.0) as i32;

    // -- LEFT: lock(color) · health(color) · git(white) --------------------
    let mut lx = PAD;
    let h_str = format!("{} {}%", IC_HEART, forest.health);
    // INT-033: semantic health thresholds -- peak>=95, advisory>=80, critical<80
    let h_color = if forest.health >= 95 { green() }
        else if forest.health >= 80 { amber() } else { red() };
    let h_w = measure_text(font_system, &h_str, 80.0);
    draw_text(canvas, font_system, swash, &h_str, lx as i32, y_text, 80.0, h_color);
    lx += h_w + 8.0;
    let lsep_w = measure_text(font_system, SEP_R, 30.0);
    draw_text(canvas, font_system, swash, SEP_R, lx as i32, y_text, 30.0, dim());
    lx += lsep_w + 8.0;
    let git_sym = if forest.git_clean { "" } else { "*" };
    let git_str = format!("{} {}{}", IC_GIT, forest.git_branch, git_sym);
    draw_text(canvas, font_system, swash, &git_str, lx as i32, y_text, third - lx, text());

    // -- CENTER: friday or intent --------------------------------------------
    let (center, center_color) = if let Some((ref msg, conf)) = forest.friday {
        (format!("🌲 {}  · {:.0}%", msg, conf * 100.0), cyan())
    } else if !forest.intent_title.is_empty() {
        // INT-033: active intent uses neon purple
        (forest.intent_title.clone(), purple())
    } else {
        ("Faelight Forest 14.0.0".to_string(), dim())
    };
    // Measure center text width then offset to visually center it
    {
        let mut cb = cosmic_text::Buffer::new(font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
        cb.set_size(font_system, Some(third), Some(LINE_HEIGHT + 4.0));
        cb.set_text(font_system, &center,
            Attrs::new().family(Family::Name(FONT_FAMILY)), Shaping::Basic);
        cb.shape_until_scroll(font_system, false);
        let text_w = cb.layout_runs().map(|r| r.line_w).fold(0.0f32, f32::max);
        let x_center = (third + (third - text_w) / 2.0).max(third) as i32;
        cb.draw(font_system, swash, center_color, |x, y, w, h, c| {
            blend(canvas, x_center + x, y_text + y, w, h, c);
        });
    }

    // -- RIGHT: draw inward from right edge: clock | battery | wifi ---------
    {
        let mut rx = phys_w as f32 - PAD;
        // Clock -- amber
        let clock_str = format!("{} {}", IC_CLOCK, forest.clock);
        let clock_w = measure_text(font_system, &clock_str, third);
        rx -= clock_w;
        draw_text(canvas, font_system, swash, &clock_str,
            rx as i32, y_text, third, amber());
        rx -= 5.0;
        let sep_w = measure_text(font_system, SEP_L, 30.0);
        rx -= sep_w;
        draw_text(canvas, font_system, swash, SEP_L, rx as i32, y_text, 30.0, dim());
        rx -= 5.0;
        // Battery -- green>=95 cyan>=50 amber>=20 red<20
        if let Some(pct) = forest.battery {
            let bat_color = if pct >= 95 { green() } else if pct >= 50 { cyan() }
                else if pct >= 20 { amber() } else { red() };
            let bat_icon = if forest.charging { IC_CHARGE } else { IC_BATT };
            let bat_str = format!("{} {}%", bat_icon, pct);
            let bat_w = measure_text(font_system, &bat_str, third);
            rx -= bat_w;
            draw_text(canvas, font_system, swash, &bat_str,
                rx as i32, y_text, third, bat_color);
            rx -= 5.0;
            let sep_w = measure_text(font_system, SEP_L, 30.0);
            rx -= sep_w;
            draw_text(canvas, font_system, swash, SEP_L, rx as i32, y_text, 30.0, dim());
            rx -= 5.0;
        }
        // WiFi -- green=up red=down
        let wifi_str = if forest.wifi_connected { IC_WIFI } else { IC_WIFI_OFF };
        let wifi_color = if forest.wifi_connected { green() } else { red() };
        let wifi_w = measure_text(font_system, wifi_str, 60.0);
        rx -= wifi_w;
        draw_text(canvas, font_system, swash, wifi_str,
            rx as i32, y_text, 60.0, wifi_color);
        rx -= 5.0;
        let sep_w = measure_text(font_system, SEP_L, 30.0);
        rx -= sep_w;
        draw_text(canvas, font_system, swash, SEP_L, rx as i32, y_text, 30.0, dim());
        rx -= 5.0;
        // RAM -- green<50 amber<80 red>=80
        let ram_str = format!("{} {}%", IC_RAM, forest.ram);
        let ram_color = if forest.ram < 50 { green() }
            else if forest.ram < 80 { amber() } else { red() };
        let ram_w = measure_text(font_system, &ram_str, 80.0);
        rx -= ram_w;
        draw_text(canvas, font_system, swash, &ram_str,
            rx as i32, y_text, 80.0, ram_color);
        rx -= 5.0;
        let sep_w = measure_text(font_system, SEP_L, 30.0);
        rx -= sep_w;
        draw_text(canvas, font_system, swash, SEP_L, rx as i32, y_text, 30.0, dim());
        rx -= 5.0;
        // CPU -- green<50 amber<80 red>=80
        let cpu_str = format!("{} {}%", IC_CPU, forest.cpu);
        let cpu_color = if forest.cpu < 50 { green() }
            else if forest.cpu < 80 { amber() } else { red() };
        let cpu_w = measure_text(font_system, &cpu_str, 80.0);
        rx -= cpu_w;
        draw_text(canvas, font_system, swash, &cpu_str,
            rx as i32, y_text, 80.0, cpu_color);
    }
}

// -- D-Bus signals -----------------------------------------------------------

#[derive(Debug)]
enum ForestSignal {
    HealthChanged(u8),
    IntentChanged(String),
}

fn spawn_dbus_thread(tx: std::sync::mpsc::Sender<ForestSignal>) {
    // Thread 1 -- Health signals
    let tx1 = tx.clone();
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all().build() {
            Ok(r) => r,
            Err(e) => { eprintln!("bar dbus rt: {e}"); return; }
        };
        rt.block_on(async move {
        let conn = match zbus::Connection::session().await {
            Ok(c) => c,
            Err(e) => { eprintln!("bar dbus: {e}"); return; }
        };
        let proxy = match zbus::Proxy::new(
            &conn, "org.faelight.Forest",
            "/org/faelight/Forest/Health",
            "org.faelight.Forest.Health",
        ).await {
            Ok(p) => p,
            Err(e) => { eprintln!("bar dbus health proxy: {e}"); return; }
        };
        // Initial value
        if let Ok(h) = proxy.get_property::<u32>("HealthPercent").await {
            let _ = tx1.send(ForestSignal::HealthChanged(h as u8));
        }
        // Signal subscription
        if let Ok(mut sigs) = proxy.receive_signal("HealthChanged").await {
            while let Some(sig) = sigs.next().await {
                if let Ok((_, new)) = sig.body().deserialize::<(u32, u32)>() {
                    let _ = tx1.send(ForestSignal::HealthChanged(new as u8));
                }
            }
        }
        });
    });

    // Thread 2 -- Intent signals
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all().build() {
            Ok(r) => r,
            Err(e) => { eprintln!("bar dbus rt2: {e}"); return; }
        };
        rt.block_on(async move {
        let conn = match zbus::Connection::session().await {
            Ok(c) => c,
            Err(e) => { eprintln!("bar dbus: {e}"); return; }
        };
        let proxy = match zbus::Proxy::new(
            &conn, "org.faelight.Forest",
            "/org/faelight/Forest/Intent",
            "org.faelight.Forest.Intent",
        ).await {
            Ok(p) => p,
            Err(e) => { eprintln!("bar dbus intent proxy: {e}"); return; }
        };
        // Initial value
        if let Ok(i) = proxy.get_property::<String>("ActiveIntent").await {
            let _ = tx.send(ForestSignal::IntentChanged(i));
        }
        // Signal subscription
        if let Ok(mut sigs) = proxy.receive_signal("IntentChanged").await {
            while let Some(sig) = sigs.next().await {
                if let Ok((_, new)) = sig.body().deserialize::<(String, String)>() {
                    let _ = tx.send(ForestSignal::IntentChanged(new));
                }
            }
        }
        });
    });
}

// -- App Struct ---------------------------------------------------------------

struct FaelightBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layer: LayerSurface,
    pool: SlotPool,
    width: u32,
    scale_120: u32,
    viewport: Option<WpViewport>,
    configured: bool,
    font_system: FontSystem,
    swash: SwashCache,
    forest: ForestState,
    last_update: Instant,
    signal_rx: std::sync::mpsc::Receiver<ForestSignal>,
}

impl FaelightBar {
    fn draw(&mut self) {
        if !self.configured { return; }
        let scale = self.scale_120 as f64 / 120.0;
        let phys_w = ((self.width as f64 * scale).ceil() as u32).max(1);
        let phys_h = ((BAR_HEIGHT as f64 * scale).ceil() as u32).max(1);
        let stride = (phys_w * 4) as i32;
        let (buffer, canvas) = match self.pool.create_buffer(
            phys_w as i32, phys_h as i32, stride, wl_shm::Format::Argb8888) {
            Ok(b) => b,
            Err(e) => { eprintln!("bar: buffer error: {e}"); return; }
        };
        draw_frame(canvas, phys_w, phys_h,
            &mut self.font_system, &mut self.swash, &self.forest);
        self.layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer.wl_surface().damage_buffer(0, 0, phys_w as i32, phys_h as i32);
        if let Some(ref vp) = self.viewport {
            vp.set_destination(self.width as i32, BAR_HEIGHT as i32);
        }
        self.layer.wl_surface().commit();
    }
}

impl LayerShellHandler for FaelightBar {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        std::process::exit(0);
    }
    fn configure(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &LayerSurface, configure: LayerSurfaceConfigure, _: u32) {
        let (w, _) = configure.new_size;
        if w > 0 { self.width = w; }
        self.configured = true;
        self.draw();
    }
}

impl ShmHandler for FaelightBar {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm }
}

delegate_compositor!(FaelightBar);
delegate_output!(FaelightBar);
delegate_shm!(FaelightBar);
delegate_layer!(FaelightBar);
delegate_registry!(FaelightBar);

impl CompositorHandler for FaelightBar {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: i32) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: u32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
}
impl OutputHandler for FaelightBar {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}
impl ProvidesRegistryState for FaelightBar {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState];
}
impl Dispatch<WpFractionalScaleManagerV1, ()> for FaelightBar {
    fn event(_: &mut Self, _: &WpFractionalScaleManagerV1,
        _: wayland_protocols::wp::fractional_scale::v1::client
            ::wp_fractional_scale_manager_v1::Event,
        _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<WpFractionalScaleV1, ()> for FaelightBar {
    fn event(state: &mut Self, _: &WpFractionalScaleV1,
        event: wp_fractional_scale_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event {
            if scale != state.scale_120 {
                state.scale_120 = scale;
                eprintln!("🌲 bar: scale {}/120 = {:.3}x", scale, scale as f64 / 120.0);
            }
        }
    }
}
impl Dispatch<WpViewporter, ()> for FaelightBar {
    fn event(_: &mut Self, _: &WpViewporter,
        _: wayland_protocols::wp::viewporter::client::wp_viewporter::Event,
        _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<WpViewport, ()> for FaelightBar {
    fn event(_: &mut Self, _: &WpViewport,
        _: wayland_protocols::wp::viewporter::client::wp_viewport::Event,
        _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

// -- main ---------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    eprintln!("🌲 faelight-bar v3.0.0 -- cosmic-text renderer starting...");
    let conn = Connection::connect_to_env()?;
    let (globals, mut eq) = registry_queue_init(&conn)?;
    let qh = eq.handle();
    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let frac_mgr = globals.bind::<WpFractionalScaleManagerV1, _, _>(&qh, 1..=1, ()).ok();
    let viewporter = globals.bind::<WpViewporter, _, _>(&qh, 1..=1, ()).ok();
    let surface = compositor.create_surface(&qh);
    let _frac = frac_mgr.as_ref().map(|m| m.get_fractional_scale(&surface, &qh, ()));
    let viewport = viewporter.as_ref().map(|vp| vp.get_viewport(&surface, &qh, ()));
    let layer = layer_shell.create_layer_surface(
        &qh, surface, Layer::Top, Some("faelight-bar"), None);
    layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer.set_size(0, BAR_HEIGHT);
    layer.set_exclusive_zone(BAR_HEIGHT as i32);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.commit();
    let pool = SlotPool::new(3840 * BAR_HEIGHT as usize * 4 * 4, &shm)?;
    let font_system = FontSystem::new();
    let swash = SwashCache::new();
    let (sig_tx, sig_rx) = std::sync::mpsc::channel::<ForestSignal>();
    spawn_dbus_thread(sig_tx);
    let mut app = FaelightBar {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm, layer, pool, width: 1920, scale_120: 120,
        viewport, configured: false,
        font_system, swash,
        forest: ForestState::refresh(),
        last_update: Instant::now(),
        signal_rx: sig_rx,
    };
    eq.roundtrip(&mut app)?;
    app.draw();
    eprintln!("✅ faelight-bar v3 live -- three zones");
    loop {
        eq.flush()?;
        let _ = eq.dispatch_pending(&mut app);
        // D-Bus signals -- instant update on health/intent change
        while let Ok(signal) = app.signal_rx.try_recv() {
            match signal {
                ForestSignal::HealthChanged(h) => {
                    app.forest.health = h;
                    app.draw();
                }
                ForestSignal::IntentChanged(title) => {
                    app.forest.intent_title = if title.len() > 55 {
                        format!("{}...", &title[..52])
                    } else { title };
                    app.draw();
                }
            }
        }
        if app.last_update.elapsed() >= Duration::from_millis(UPDATE_MS) {
            app.last_update = Instant::now();
            app.forest = ForestState::refresh();
            app.draw();
            eq.flush().ok();
            let _ = eq.dispatch_pending(&mut app);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
