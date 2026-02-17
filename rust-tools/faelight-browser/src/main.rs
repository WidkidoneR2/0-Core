//! faelight-browser v0.1.0 - Entry point
//!
//! Phase 1: Minimal Wayland window with Faelight Forest colors

use faelight_browser::Result;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
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
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

// Import our colors
mod ui {
    pub mod colors {
        pub const BG_COLOR: [u8; 4] = [0x11, 0x14, 0x0f, 0xFF];
        pub const ACCENT_COLOR: [u8; 4] = [0xa3, 0xe3, 0x6b, 0xFF];
    }
}

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

fn main() -> Result<()> {
    println!("🌲 faelight-browser v0.1.0");
    println!("📋 Phase 1: Wayland Window");
    println!("💡 Press ESC to close");
    println!();

    // Connect to Wayland
    let conn = Connection::connect_to_env()
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;

    println!("✅ Connected to Wayland");

    let (globals, mut event_queue) = registry_queue_init(&conn)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;

    let qh = event_queue.handle();

    // Create compositor
    let compositor = CompositorState::bind(&globals, &qh)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;

    // Create layer shell
    let layer_shell = LayerShell::bind(&globals, &qh)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;

    // Create shared memory
    let shm = Shm::bind(&globals, &qh)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;

    // Create seat for keyboard input
    let seat_state = SeatState::new(&globals, &qh);

    // Create surface
    let surface = compositor.create_surface(&qh);

    // Create layer surface (browser window)
    let layer_surface =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("faelight-browser"), None);

    // Configure window (keyboard interactivity = OnDemand so ESC works)
    layer_surface.set_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    layer_surface.set_anchor(Anchor::empty()); // Floating window
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer_surface.commit();

    println!("✅ Window created ({}x{})", WINDOW_WIDTH, WINDOW_HEIGHT);

    // Create buffer pool
    let pool = SlotPool::new(WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize * 4, &shm)
        .map_err(|e| faelight_browser::error::BrowserError::Rendering(e.to_string()))?;

    // Create state
    let mut state = BrowserState {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        seat_state,
        shm,
        layer_surface,
        pool,
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        configured: false,
        running: true,
    };

    println!("✅ State initialized");
    println!();
    println!("🎨 Rendering Faelight Forest colors...");

    // Wait for configure
    event_queue
        .roundtrip(&mut state)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;

    // Main event loop
    while state.running {
        event_queue
            .blocking_dispatch(&mut state)
            .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;
    }

    println!("👋 Browser closed");

    Ok(())
}

struct BrowserState {
    registry_state: RegistryState,
    output_state: OutputState,
    seat_state: SeatState,
    shm: Shm,
    layer_surface: LayerSurface,
    pool: SlotPool,
    width: u32,
    height: u32,
    configured: bool,
    running: bool,
}

impl BrowserState {
    fn draw(&mut self, _qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to create buffer: {}", e);
                return;
            }
        };

        // Fill with background color
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&ui::colors::BG_COLOR);
        }

        // Draw accent line at top (2px)
        for x in 0..width as usize {
            for y in 0..2 {
                let idx = (y * width as usize + x) * 4;
                canvas[idx..idx + 4].copy_from_slice(&ui::colors::ACCENT_COLOR);
            }
        }

        // Attach and commit
        self.layer_surface
            .wl_surface()
            .attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer_surface
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        self.layer_surface.commit();
    }
}

impl CompositorHandler for BrowserState {
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
    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        self.draw(qh);
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

impl OutputHandler for BrowserState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for BrowserState {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.running = false;
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }
        self.configured = true;
        self.draw(qh);
    }
}

impl ShmHandler for BrowserState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SeatHandler for BrowserState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }
    fn new_capability(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        seat: wayland_client::protocol::wl_seat::WlSeat,
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
        _: wayland_client::protocol::wl_seat::WlSeat,
        _: Capability,
    ) {
    }
    fn remove_seat(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wayland_client::protocol::wl_seat::WlSeat,
    ) {
    }
}

impl KeyboardHandler for BrowserState {
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

    fn press_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        // ESC to close
        if event.keysym == Keysym::Escape {
            println!("ESC pressed - closing browser");
            self.running = false;
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
}

impl ProvidesRegistryState for BrowserState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(BrowserState);
delegate_output!(BrowserState);
delegate_layer!(BrowserState);
delegate_shm!(BrowserState);
delegate_seat!(BrowserState);
delegate_keyboard!(BrowserState);
delegate_registry!(BrowserState);
