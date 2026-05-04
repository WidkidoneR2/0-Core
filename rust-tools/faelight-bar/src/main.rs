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
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::wp::fractional_scale::v1::client::{
    wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
    wp_fractional_scale_v1::{self, WpFractionalScaleV1},
};
use wayland_protocols::wp::viewporter::client::{
    wp_viewport::WpViewport, wp_viewporter::WpViewporter,
};

mod logger;
mod menu;
mod paths;
mod render;
mod widgets;

const BAR_HEIGHT: u32 = 32;
const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

#[allow(dead_code)]
enum CenterState {
    Intent,
    FridaySignal(String, Instant),
}
struct FaelightBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layer: LayerSurface,
    pool: SlotPool,
    width: u32,
    first_configure: bool,
    last_update: Instant,
    scale_120: u32, // fractional scale × 120 (e.g. 180 = 1.5x)
    viewport: Option<WpViewport>,
    center_state: CenterState,
    last_signal_check: Instant,
}

impl FaelightBar {
    fn draw(&mut self) {
        let width = self.width;
        // Fractional scaling: render at physical pixels, viewport to logical
        let scale = self.scale_120 as f64 / 120.0;
        let phys_w = ((width as f64 * scale).ceil() as u32).max(1);
        let phys_h = ((BAR_HEIGHT as f64 * scale).ceil() as u32).max(1);
        let stride = (phys_w * 4) as i32;

        let (buffer, canvas) = match self.pool.create_buffer(
            phys_w as i32,
            phys_h as i32,
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
        render::bar::render(canvas, phys_w, phys_h, self.scale_120 as f32 / 120.0);

        self.layer
            .wl_surface()
            .attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, phys_w as i32, phys_h as i32);
        // Set viewport destination to logical size — compositor handles the scale
        if let Some(ref vp) = self.viewport {
            vp.set_destination(width as i32, BAR_HEIGHT as i32);
        }
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

impl Dispatch<WpFractionalScaleManagerV1, ()> for FaelightBar {
    fn event(
        _: &mut Self,
        _: &WpFractionalScaleManagerV1,
        _: wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WpFractionalScaleV1, ()> for FaelightBar {
    fn event(
        state: &mut Self,
        _: &WpFractionalScaleV1,
        event: wp_fractional_scale_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event {
            if scale != state.scale_120 {
                state.scale_120 = scale;
                eprintln!(
                    "🔭 Fractional scale: {}/120 = {:.3}x",
                    scale,
                    scale as f64 / 120.0
                );
            }
        }
    }
}

impl Dispatch<WpViewporter, ()> for FaelightBar {
    fn event(
        _: &mut Self,
        _: &WpViewporter,
        _: wayland_protocols::wp::viewporter::client::wp_viewporter::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WpViewport, ()> for FaelightBar {
    fn event(
        _: &mut Self,
        _: &WpViewport,
        _: wayland_protocols::wp::viewporter::client::wp_viewport::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

fn main() {
    eprintln!("🌲 faelight-bar v5.0.0 starting...");

    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    // Bind fractional scale and viewport protocols
    let frac_manager = globals
        .bind::<WpFractionalScaleManagerV1, _, _>(&qh, 1..=1, ())
        .ok();
    let viewporter = globals.bind::<WpViewporter, _, _>(&qh, 1..=1, ()).ok();

    let surface = compositor.create_surface(&qh);

    // Set up fractional scale for this surface
    let frac_scale = frac_manager
        .as_ref()
        .map(|m| m.get_fractional_scale(&surface, &qh, ()));
    let viewport = viewporter
        .as_ref()
        .map(|vp| vp.get_viewport(&surface, &qh, ()));

    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("faelight-bar"), None);

    layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer.set_size(0, BAR_HEIGHT);
    layer.set_exclusive_zone(BAR_HEIGHT as i32);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.commit();

    let pool = SlotPool::new(3840 * (BAR_HEIGHT as usize * 2) * 4 * 4, &shm)
        .expect("Failed to create pool");

    let mut app = FaelightBar {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        layer,
        scale_120: 120,
        viewport,
        pool,
        width: 1920,
        first_configure: false,
        last_update: Instant::now(),
        center_state: CenterState::Intent,
        last_signal_check: Instant::now(),
    };

    let _frac_scale = frac_scale; // keep alive
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

        // Check Friday signal every 5s
        if app.first_configure && app.last_signal_check.elapsed() >= Duration::from_secs(5) {
            app.last_signal_check = Instant::now();
            if let Some(signal) = render::bar::get_friday_signal() {
                app.center_state = CenterState::FridaySignal(signal, Instant::now());
            }
        }
        // Return to intent after 10s
        let should_reset = if let CenterState::FridaySignal(_, ref since) = app.center_state {
            since.elapsed() >= Duration::from_secs(10)
        } else {
            false
        };
        if should_reset {
            app.center_state = CenterState::Intent;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
