// faelight-notify v4.0.0 — Freedesktop Spec, zbus D-Bus, Wayland Native
// INT-141 — Follows exact faelight-bar Wayland pattern

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
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

mod dbus;
mod queue;
mod render;

pub use render::*;

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub urgency: Urgency,
    pub timeout: i32,
    pub created: Instant,
    pub display_start: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

impl Urgency {
    pub fn from_hints(
        hints: &std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
    ) -> Self {
        if let Some(v) = hints.get("urgency") {
            if let Ok(u) = u8::try_from(v) {
                return match u {
                    0 => Self::Low,
                    2 => Self::Critical,
                    _ => Self::Normal,
                };
            }
        }
        Self::Normal
    }
    pub fn border_color(&self) -> [u8; 4] {
        match self {
            Self::Low => render::BORDER_LOW,
            Self::Normal => render::BORDER_NORMAL,
            Self::Critical => render::BORDER_CRITICAL,
        }
    }
    pub fn timeout_ms(&self) -> u64 {
        match self {
            Self::Critical => 10000,
            Self::Low => 5000,
            Self::Normal => 6000,
        }
    }
}

pub type NotifQueue = Arc<Mutex<Vec<Notification>>>;

const POPUP_W: u32 = 380;
const POPUP_H: u32 = 80;

struct NotifyApp {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer: LayerSurface,
    queue: NotifQueue,
    first_configure: bool,
    last_draw: Instant,
}

impl NotifyApp {
    fn draw(&mut self) {
        // Set display_start when notification first becomes visible
        {
            let mut q = self.queue.lock().unwrap();
            if let Some(first) = q.first_mut() {
                if first.display_start.is_none() {
                    first.display_start = Some(std::time::Instant::now());
                }
            }
        }
        let notif = self.queue.lock().unwrap().first().cloned();
        let (buffer, canvas) = match self.pool.create_buffer(
            POPUP_W as i32,
            POPUP_H as i32,
            (POPUP_W * 4) as i32,
            wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(_) => return,
        };

        for p in canvas.chunks_exact_mut(4) {
            p.copy_from_slice(&[0, 0, 0, 0]);
        }

        if let Some(n) = notif {
            render::draw_notification(
                canvas,
                POPUP_W,
                POPUP_H,
                &n.app_name,
                &n.summary,
                &n.body,
                n.urgency.border_color(),
            );
        }

        self.layer
            .wl_surface()
            .attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, POPUP_W as i32, POPUP_H as i32);
        self.layer.wl_surface().commit();
        self.last_draw = Instant::now();
    }
}

impl LayerShellHandler for NotifyApp {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {}
    fn configure(
        &mut self,
        _: &Connection,
        _qh: &QueueHandle<Self>,
        _: &LayerSurface,
        _: LayerSurfaceConfigure,
        _: u32,
    ) {
        if self.first_configure {
            self.first_configure = false;
        }
        self.draw();
    }
}

impl CompositorHandler for NotifyApp {
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

impl OutputHandler for NotifyApp {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl ShmHandler for NotifyApp {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for NotifyApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_compositor!(NotifyApp);
delegate_output!(NotifyApp);
delegate_shm!(NotifyApp);
delegate_layer!(NotifyApp);
delegate_registry!(NotifyApp);

fn main() {
    eprintln!("🌲 faelight-notify v4.0.0 starting...");

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--health" || args[1] == "health") {
        println!("faelight-notify v4.0.0 — healthy");
        return;
    }

    let queue: NotifQueue = Arc::new(Mutex::new(Vec::new()));

    // D-Bus server on separate thread
    let dbus_queue = queue.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            if let Err(e) = dbus::run(dbus_queue).await {
                eprintln!("❌ D-Bus error: {}", e);
            }
        });
    });

    // Wayland setup — exact bar pattern
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let surface = compositor.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Overlay,
        Some("faelight-notify"),
        None,
    );
    layer.set_size(POPUP_W, POPUP_H);
    layer.set_anchor(Anchor::TOP | Anchor::RIGHT);
    layer.set_margin(8, 8, 0, 0);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.wl_surface().commit();

    let pool =
        SlotPool::new((POPUP_W * POPUP_H * 4 * 16) as usize, &shm).expect("Failed to create pool");

    let mut app = NotifyApp {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer,
        queue: queue.clone(),
        first_configure: true,
        last_draw: Instant::now(),
    };

    event_queue
        .roundtrip(&mut app)
        .expect("Initial roundtrip failed");
    eprintln!("✅ faelight-notify v4.0.0 running");

    loop {
        event_queue.flush().expect("Flush failed");
        let _ = event_queue.dispatch_pending(&mut app);

        // Expire old notifications
        {
            let mut q = queue.lock().unwrap();
            q.retain(|n| {
                let ms = if n.timeout > 0 {
                    n.timeout as u64
                } else {
                    n.urgency.timeout_ms()
                };
                // Expire from display_start if set, else from created
                let elapsed = n.display_start.unwrap_or(n.created).elapsed().as_millis();
                elapsed < ms as u128
            });
        }

        // Redraw at 10Hz or immediately if queue has items
        let has_notif = !app.queue.lock().unwrap().is_empty();
        if has_notif || app.last_draw.elapsed() >= Duration::from_millis(100) {
            app.draw();
            event_queue.flush().ok();
            let _ = event_queue.dispatch_pending(&mut app);
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}
