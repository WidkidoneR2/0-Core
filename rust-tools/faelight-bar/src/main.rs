//! faelight-bar v5.0.0 - Wired to render/bar.rs with widget system

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
use std::time::{Duration, Instant};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

mod logger;
mod menu;
mod paths;
mod render;
mod widgets;

const BAR_HEIGHT: u32 = 32;
const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

struct FaelightBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layer: LayerSurface,
    pool: SlotPool,
    width: u32,
    first_configure: bool,
    last_update: Instant,
}

impl FaelightBar {
    fn draw(&mut self) {
        let width = self.width;
        let stride = (width * 4) as i32;

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            BAR_HEIGHT as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("❌ Buffer exhausted: {}", e);
                return;
            }
        };

        // Fill background
        let bg = render::colors::BG.to_le_bytes();
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&bg);
        }

        // Delegate all drawing to render/bar.rs
        render::bar::render(canvas, width, BAR_HEIGHT);

        self.layer
            .wl_surface()
            .attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, BAR_HEIGHT as i32);
        self.layer.wl_surface().commit();
    }
}

impl LayerShellHandler for FaelightBar {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        std::process::exit(0);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        eprintln!("⚙️  Configure: {:?}", configure.new_size);
        let (width, _height) = configure.new_size;
        if width > 0 {
            self.width = width;
        }
        self.first_configure = true;
        self.draw();
    }
}

delegate_compositor!(FaelightBar);
delegate_output!(FaelightBar);
delegate_shm!(FaelightBar);
delegate_layer!(FaelightBar);
delegate_registry!(FaelightBar);

impl OutputHandler for FaelightBar {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl ShmHandler for FaelightBar {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl CompositorHandler for FaelightBar {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }
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

impl ProvidesRegistryState for FaelightBar {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

fn main() {
    eprintln!("🌲 faelight-bar v5.0.0 starting...");

    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("faelight-bar"), None);

    layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer.set_size(0, BAR_HEIGHT);
    layer.set_exclusive_zone(BAR_HEIGHT as i32);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.commit();

    let pool =
        SlotPool::new(1920 * BAR_HEIGHT as usize * 4 * 4, &shm).expect("Failed to create pool");

    let mut app = FaelightBar {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        layer,
        pool,
        width: 1920,
        first_configure: false,
        last_update: Instant::now(),
    };

    // Block until we get the initial configure from the compositor
    event_queue
        .roundtrip(&mut app)
        .expect("Initial roundtrip failed");
    eprintln!("✅ Initial configure received, bar visible");

    loop {
        // Flush outgoing requests
        event_queue.flush().expect("Flush failed");

        // Read and dispatch incoming events (buffer releases, configure, etc.)
        let _ = event_queue.dispatch_pending(&mut app);

        // Redraw on interval
        if app.first_configure && app.last_update.elapsed() >= UPDATE_INTERVAL {
            app.last_update = Instant::now();
            app.draw();
            // Extra dispatch after draw to process buffer release immediately
            event_queue.flush().ok();
            let _ = event_queue.dispatch_pending(&mut app);
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}
