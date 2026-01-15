//! faelight-notify v0.8.0 - Typography Polish
//! 🌲 Faelight Forest

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
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
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};
use faelight_core::GlyphCache;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use zbus::{connection, interface};

const NOTIFY_WIDTH: u32 = 400;
const NOTIFY_HEIGHT: u32 = 80;
const MARGIN: u32 = 15;

const BG_COLOR: [u8; 4] = [0x1a, 0x1d, 0x18, 0xF5];
const BORDER_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const TITLE_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const DIM_COLOR: [u8; 4] = [0x7f, 0x8f, 0x77, 0xFF];
const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Regular.ttf");

// ═══════════════════════════════════════════════════════════
// 📐 TYPOGRAPHY
// ═══════════════════════════════════════════════════════════
const FONT_APP: f32 = 16.0;
const FONT_TITLE: f32 = 20.0;
const FONT_BODY: f32 = 17.0;
const FONT_BADGE: f32 = 14.0;

#[derive(Clone, Debug)]
struct Notification {
    app_name: String,
    summary: String,
    body: String,
    created: Instant,
    timeout_ms: i32,
}

impl Notification {
    fn is_expired(&self) -> bool {
        let timeout = if self.timeout_ms <= 0 { 5000 } else { self.timeout_ms };
        self.created.elapsed() > Duration::from_millis(timeout as u64)
    }
}

struct NotificationServer {
    notifications: Arc<Mutex<Vec<Notification>>>,
    next_id: Arc<Mutex<u32>>,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    fn get_capabilities(&self) -> Vec<String> {
        vec!["body".into()]
    }

    fn notify(&self, app_name: String, _replaces_id: u32, _app_icon: String, summary: String, body: String, _actions: Vec<String>, _hints: std::collections::HashMap<String, zbus::zvariant::OwnedValue>, expire_timeout: i32) -> u32 {
        let mut id = self.next_id.lock().expect("Failed to lock next_id mutex");
        *id += 1;
        let current_id = *id;
        drop(id);

        eprintln!("📨 {} - {}", summary, body);
        self.notifications.lock().expect("Failed to lock notifications mutex").push(Notification {
            app_name, summary, body,
            created: Instant::now(),
            timeout_ms: expire_timeout,
        });
        current_id
    }

    fn close_notification(&self, _id: u32) {}
    fn get_server_information(&self) -> (String, String, String, String) {
        ("faelight-notify".into(), "faelight".into(), "0.4.0".into(), "1.2".into())
    }
}

fn measure_text_width(cache: &mut GlyphCache, text: &str, size: f32) -> u32 {
    let mut width = 0.0;
    for ch in text.chars() {
        let metrics = cache.rasterize(ch, size).metrics;
        width += metrics.advance_width;
    }
    width as u32
}

fn truncate_text(cache: &mut GlyphCache, text: &str, max_width: u32, size: f32) -> String {
    let ellipsis = "...";
    let ellipsis_width = measure_text_width(cache, ellipsis, size);
    
    if measure_text_width(cache, text, size) <= max_width {
        return text.to_string();
    }
    
    let mut result = String::new();
    let mut current_width = 0;
    
    for ch in text.chars() {
        let metrics = cache.rasterize(ch, size).metrics;
        if current_width + metrics.advance_width as u32 + ellipsis_width > max_width {
            break;
        }
        result.push(ch);
        current_width += metrics.advance_width as u32;
    }
    
    result + ellipsis
}
fn draw_text(
    cache: &mut GlyphCache, canvas: &mut [u8], width: u32, height: u32, text: &str, x: u32, y: u32, color: [u8; 4], size: f32) {
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

fn draw_border(canvas: &mut [u8], width: u32, height: u32) {
    let stride = width as usize * 4;
    for t in 0..2usize {
        for x in 0..width as usize {
            canvas[t * stride + x * 4..t * stride + x * 4 + 4].copy_from_slice(&BORDER_COLOR);
            canvas[(height as usize - 1 - t) * stride + x * 4..(height as usize - 1 - t) * stride + x * 4 + 4].copy_from_slice(&BORDER_COLOR);
        }
        for y in 0..height as usize {
            canvas[y * stride + t * 4..y * stride + t * 4 + 4].copy_from_slice(&BORDER_COLOR);
            canvas[y * stride + (width as usize - 1 - t) * 4..y * stride + (width as usize - 1 - t) * 4 + 4].copy_from_slice(&BORDER_COLOR);
        }
    }
}

struct NotifyState {
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
    notifications: Arc<Mutex<Vec<Notification>>>,
    running: bool,
}

impl NotifyState {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        // Clean expired
        self.notifications.lock().expect("Failed to lock notifications mutex").retain(|n| !n.is_expired());

        let notifs = self.notifications.lock().expect("Failed to lock notifications mutex");
        let notif = notifs.first().cloned();
        let count = notifs.len();
        drop(notifs);

        let surface = match &self.layer_surface {
            Some(s) => s,
            None => return,
        };

        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let pool = match &mut self.pool {
            Some(p) => p,
            None => return,
        };

        let (buffer, canvas) = match pool.create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888) {
            Ok(b) => b,
            Err(_) => return,
        };

        // Fill based on notification state
        let bg = if notif.is_some() { BG_COLOR } else { TRANSPARENT };
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bg);
        }

        if let Some(n) = notif {
            draw_border(canvas, width, height);
            draw_text(&mut self.glyph_cache, canvas, width, height, &n.app_name, 12, 12, DIM_COLOR, FONT_APP);
            let summary = truncate_text(&mut self.glyph_cache, &n.summary, width - 24, FONT_TITLE);
            draw_text(&mut self.glyph_cache, canvas, width, height, &summary, 12, 28, TITLE_COLOR, FONT_TITLE);
            let body = truncate_text(&mut self.glyph_cache, &n.body, width - 24, FONT_BODY);
            draw_text(&mut self.glyph_cache, canvas, width, height, &body, 12, 50, TEXT_COLOR, FONT_BODY);
            if count > 1 {
                draw_text(&mut self.glyph_cache, canvas, width, height, &format!("+{}", count - 1), width - 35, 12, BORDER_COLOR, FONT_BADGE);
            }
        }

        surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        surface.wl_surface().frame(qh, surface.wl_surface().clone());
        surface.wl_surface().commit();
    }
}

impl CompositorHandler for NotifyState {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        self.draw(qh);
    }
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
}

impl OutputHandler for NotifyState {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for NotifyState {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) { self.running = false; }
    fn configure(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &LayerSurface, configure: LayerSurfaceConfigure, _: u32) {
        self.width = configure.new_size.0.max(NOTIFY_WIDTH);
        self.height = configure.new_size.1.max(NOTIFY_HEIGHT);
        self.configured = true;
        self.draw(qh);
    }
}

impl SeatHandler for NotifyState {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Pointer { self.seat_state.get_pointer(qh, &seat).ok(); }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for NotifyState {
    fn pointer_frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_pointer::WlPointer, events: &[PointerEvent]) {
        for event in events {
            if let PointerEventKind::Press { button: 272, .. } = event.kind {
                let mut notifs = self.notifications.lock().expect("Failed to lock notifications mutex");
                if !notifs.is_empty() { notifs.remove(0); eprintln!("🔕 Dismissed"); }
            }
        }
    }
}

impl ShmHandler for NotifyState {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm }
}

impl ProvidesRegistryState for NotifyState {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(NotifyState);
delegate_output!(NotifyState);
delegate_shm!(NotifyState);
delegate_seat!(NotifyState);
delegate_pointer!(NotifyState);
delegate_layer!(NotifyState);
delegate_registry!(NotifyState);

fn health_check() {
    println!("🏥 faelight-notify health check");
    
    // Check Wayland
    match Connection::connect_to_env() {
        Ok(_) => println!("✅ wayland: connected"),
        Err(e) => {
            eprintln!("❌ wayland: failed - {}", e);
            std::process::exit(1);
        }
    }
    
    // Check font
    match GlyphCache::new(FONT_DATA) {
        Ok(_) => println!("✅ font: loaded"),
        Err(e) => {
            eprintln!("❌ font: failed - {}", e);
            std::process::exit(1);
        }
    }
    
    // Check D-Bus
    match zbus::blocking::Connection::session() {
        Ok(_) => println!("✅ dbus: connected"),
        Err(e) => {
            eprintln!("❌ dbus: failed - {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n✅ Core checks passed!");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🌲 faelight-notify v0.8.0 starting...");
    // Check for health flag
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--health" || args[1] == "health") {
        health_check();
        return Ok(());
    }
    

    let notifications: Arc<Mutex<Vec<Notification>>> = Arc::new(Mutex::new(Vec::new()));
    let notifs_for_dbus = notifications.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let server = NotificationServer {
                notifications: notifs_for_dbus,
                next_id: Arc::new(Mutex::new(0)),
            };
            let _conn = connection::Builder::session()
                .expect("session")
                .name("org.freedesktop.Notifications")
                .expect("name")
                .serve_at("/org/freedesktop/Notifications", server)
                .expect("serve")
                .build().await.expect("conn");
            eprintln!("🔌 D-Bus ready");
            std::future::pending::<()>().await;
        });
    });

    std::thread::sleep(Duration::from_millis(100));

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;

    let surface = compositor.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("faelight-notify"), None);

    layer_surface.set_anchor(Anchor::TOP | Anchor::RIGHT);
    layer_surface.set_size(NOTIFY_WIDTH, NOTIFY_HEIGHT);
    layer_surface.set_margin(MARGIN as i32, MARGIN as i32, 0, 0);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer_surface.commit();

    let pool = SlotPool::new(NOTIFY_WIDTH as usize * NOTIFY_HEIGHT as usize * 4, &shm)?;
    let glyph_cache = GlyphCache::new(FONT_DATA)?;

    let mut state = NotifyState {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        compositor_state: compositor,
        shm, layer_shell,
        layer_surface: Some(layer_surface),
        pool: Some(pool),
        width: NOTIFY_WIDTH,
        height: NOTIFY_HEIGHT,
        configured: false,
        glyph_cache,
        notifications,
        running: true,
    };

    eprintln!("✅ faelight-notify running!");

    // Start frame loop
    if let Some(ref s) = state.layer_surface {
        s.wl_surface().frame(&qh, s.wl_surface().clone());
        s.wl_surface().commit();
    }

    while state.running {
        event_queue.blocking_dispatch(&mut state)?;
    }

    Ok(())
}

