//! faelight-notify - Rust Notification Daemon
//! 🌲 Faelight Forest v6.2

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
use fontdue::{Font, FontSettings};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::os::unix::io::AsFd;
use tokio::sync::mpsc;
use zbus::{connection, interface};

// ═══════════════════════════════════════════════════════════
// 🎨 FAELIGHT FOREST COLORS
// ═══════════════════════════════════════════════════════════
const NOTIFY_WIDTH: u32 = 350;
const NOTIFY_HEIGHT: u32 = 80;
const MARGIN: u32 = 10;

const BG_COLOR: [u8; 4] = [0x1c, 0x1f, 0x1a, 0xF0];
const BORDER_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const TEXT_COLOR: [u8; 4] = [0xda, 0xe0, 0xd7, 0xFF];
const TITLE_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
const DIM_COLOR: [u8; 4] = [0x7f, 0x8f, 0x77, 0xFF];

const FONT_DATA: &[u8] = include_bytes!("/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf");

// ═══════════════════════════════════════════════════════════
// 📨 NOTIFICATION STRUCTURE
// ═══════════════════════════════════════════════════════════
#[derive(Clone, Debug)]
struct Notification {
    #[allow(dead_code)]
    id: u32,
    app_name: String,
    summary: String,
    body: String,
    timeout: i32,
    created: Instant,
}

impl Notification {
    fn is_expired(&self) -> bool {
        if self.timeout <= 0 {
            self.created.elapsed() > Duration::from_secs(5)
        } else {
            self.created.elapsed() > Duration::from_millis(self.timeout as u64)
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🔌 D-BUS INTERFACE
// ═══════════════════════════════════════════════════════════
struct NotificationServer {
    sender: mpsc::UnboundedSender<Notification>,
    next_id: Arc<Mutex<u32>>,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    fn get_capabilities(&self) -> Vec<String> {
        vec!["body".to_string(), "body-markup".to_string()]
    }

    fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        _app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        _hints: std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id > 0 {
            replaces_id
        } else {
            let mut next = self.next_id.lock().unwrap();
            *next += 1;
            *next
        };

        let notification = Notification {
            id,
            app_name,
            summary,
            body,
            timeout: expire_timeout,
            created: Instant::now(),
        };

        eprintln!("📨 {} - {}", notification.summary, notification.body);
        let _ = self.sender.send(notification);
        id
    }

    fn close_notification(&self, _id: u32) {}

    fn get_server_information(&self) -> (String, String, String, String) {
        ("faelight-notify".into(), "faelight".into(), "0.1.0".into(), "1.2".into())
    }
}

// ═══════════════════════════════════════════════════════════
// 🖼️ RENDERING HELPERS
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
// 🖼️ WAYLAND STATE
// ═══════════════════════════════════════════════════════════
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
    font: Font,
    notifications: Arc<Mutex<Vec<Notification>>>,
    running: bool,
}

impl NotifyState {
    fn draw(&mut self) {
        let mut notifs = self.notifications.lock().unwrap();
        notifs.retain(|n| !n.is_expired());
        
        let has_notifs = !notifs.is_empty();
        let notif_data = notifs.first().cloned();
        drop(notifs);

        if !has_notifs {
            if let Some(ref surface) = self.layer_surface {
                surface.wl_surface().attach(None, 0, 0);
                surface.wl_surface().commit();
            }
            return;
        }

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

        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&BG_COLOR);
        }

        draw_border(canvas, width, height);

        if let Some(n) = notif_data {
            draw_text(&self.font, canvas, width, height, &n.app_name, 10, 8, DIM_COLOR, 12.0);
            draw_text(&self.font, canvas, width, height, &n.summary, 10, 28, TITLE_COLOR, 16.0);
            let body = if n.body.len() > 45 { format!("{}...", &n.body[..42]) } else { n.body };
            draw_text(&self.font, canvas, width, height, &body, 10, 52, TEXT_COLOR, 14.0);
        }

        if let Some(ref surface) = self.layer_surface {
            surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
            surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
            surface.wl_surface().commit();
        }
    }
}

impl CompositorHandler for NotifyState {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, _qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        self.draw();
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
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.running = false;
    }
    fn configure(&mut self, _: &Connection, _qh: &QueueHandle<Self>, _: &LayerSurface, configure: LayerSurfaceConfigure, _: u32) {
        self.width = configure.new_size.0.max(NOTIFY_WIDTH);
        self.height = configure.new_size.1.max(NOTIFY_HEIGHT);
        self.configured = true;
        self.draw();
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
                let mut notifs = self.notifications.lock().unwrap();
                if !notifs.is_empty() {
                    notifs.remove(0);
                    eprintln!("🔕 Dismissed");
                }
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

// ═══════════════════════════════════════════════════════════
// 🚀 MAIN
// ═══════════════════════════════════════════════════════════
fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("🌲 faelight-notify v0.1.0 starting...");

    let notifications: Arc<Mutex<Vec<Notification>>> = Arc::new(Mutex::new(Vec::new()));
    let notifications_clone = notifications.clone();
    
    // D-Bus in separate thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let next_id = Arc::new(Mutex::new(0u32));

            let server = NotificationServer { sender: tx, next_id };

            let conn = connection::Builder::session()
                .expect("D-Bus session")
                .name("org.freedesktop.Notifications")
                .expect("D-Bus name")
                .serve_at("/org/freedesktop/Notifications", server)
                .expect("D-Bus serve")
                .build()
                .await
                .expect("D-Bus connection");

            eprintln!("🔌 D-Bus ready");

            loop {
                tokio::select! {
                    _ = conn.executor().tick() => {},
                    Some(notif) = rx.recv() => {
                        notifications_clone.lock().unwrap().push(notif);
                    }
                }
            }
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
    let font = Font::from_bytes(FONT_DATA, FontSettings::default())?;

    let mut state = NotifyState {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        compositor_state: compositor,
        shm,
        layer_shell,
        layer_surface: Some(layer_surface),
        pool: Some(pool),
        width: NOTIFY_WIDTH,
        height: NOTIFY_HEIGHT,
        configured: false,
        font,
        notifications,
        running: true,
    };

    eprintln!("✅ faelight-notify running!");

    // Non-blocking event loop with periodic redraw
    let fd = conn.as_fd();
    use std::os::unix::io::BorrowedFd;
    
    
    while state.running {
        // Poll with timeout
        let mut fds = [nix::poll::PollFd::new(fd, nix::poll::PollFlags::POLLIN)];
        let _ = nix::poll::poll(&mut fds, nix::poll::PollTimeout::from(100u16));
        
        // Dispatch any pending events
        event_queue.dispatch_pending(&mut state)?;
        conn.flush()?;
        
        // Redraw
        if state.configured {
            state.draw();
        }
    }

    Ok(())
}
