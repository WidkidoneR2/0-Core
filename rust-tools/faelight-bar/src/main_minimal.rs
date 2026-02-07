// faelight-bar v3.0 - Minimal Cache-Based Bar
use std::time::{Duration, Instant};
use std::process::Command;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, 
    delegate_pointer, delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState, keyboard::KeyboardHandler, pointer::PointerHandler},
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, 
                    LayerSurface, LayerSurfaceConfigure},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

const BAR_HEIGHT: u32 = 32;
const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone)]
struct StatusData {
    profile: String,
    health: String,
    updates: String,
    lock: String,
    time: String,
}

impl Default for StatusData {
    fn default() -> Self {
        Self {
            profile: "".to_string(),
            health: "".to_string(),
            updates: "".to_string(),
            lock: "".to_string(),
            time: "00:00".to_string(),
        }
    }
}

impl StatusData {
    fn update(&mut self) {
        if let Ok(output) = Command::new("/home/christian/0-core/status-blocks/profile").output() {
            self.profile = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
        if let Ok(output) = Command::new("/home/christian/0-core/status-blocks/health").output() {
            self.health = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
        if let Ok(output) = Command::new("/home/christian/0-core/status-blocks/updates").output() {
            self.updates = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
        if let Ok(output) = Command::new("/home/christian/0-core/status-blocks/lock").output() {
            self.lock = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
        if let Ok(output) = Command::new("/home/christian/0-core/status-blocks/time").output() {
            self.time = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
}

struct BarState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer_surface: LayerSurface,
    width: u32,
    height: u32,
    configured: bool,
    running: bool,
    status: StatusData,
    last_update: Instant,
}

impl BarState {
    fn update_status(&mut self) {
        if self.last_update.elapsed() >= UPDATE_INTERVAL {
            self.status.update();
            self.last_update = Instant::now();
        }
    }
    
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        self.update_status();
        
        if self.width == 0 { return; }
        
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;
        
        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32, height as i32, stride, wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(_) => return,
        };
        
        // Dark green background
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0x0f, 0x14, 0x11, 0xFF]);
        }
        
        self.layer_surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer_surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        self.layer_surface.commit();
    }
}

impl CompositorHandler for BarState {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) { self.draw(qh); }
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
}

impl OutputHandler for BarState {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for BarState {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) { self.running = false; }
    fn configure(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &LayerSurface, configure: LayerSurfaceConfigure, _: u32) {
        if configure.new_size.0 > 0 { self.width = configure.new_size.0; }
        if configure.new_size.1 > 0 { self.height = configure.new_size.1; }
        self.configured = true;
        self.draw(qh);
    }
}

impl ShmHandler for BarState {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm }
}

impl SeatHandler for BarState {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for BarState {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[smithay_client_toolkit::seat::keyboard::Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: smithay_client_toolkit::seat::keyboard::KeyEvent) {}
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: smithay_client_toolkit::seat::keyboard::KeyEvent) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: smithay_client_toolkit::seat::keyboard::Modifiers, _: u32) {}
}

impl PointerHandler for BarState {
    fn pointer_frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_pointer::WlPointer, _: &[smithay_client_toolkit::seat::pointer::PointerEvent]) {}
}

impl ProvidesRegistryState for BarState {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(BarState);
delegate_output!(BarState);
delegate_layer!(BarState);
delegate_shm!(BarState);
delegate_registry!(BarState);
delegate_seat!(BarState);
delegate_keyboard!(BarState);
delegate_pointer!(BarState);

fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
    let qh = event_queue.handle();
    
    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");
    let seat_state = SeatState::new(&globals, &qh);
    let surface = compositor.create_surface(&qh);
    
    let layer_surface = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("faelight-bar"), None);
    layer_surface.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer_surface.set_size(0, BAR_HEIGHT);
    layer_surface.set_exclusive_zone(BAR_HEIGHT as i32);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer_surface.commit();
    
    let pool = SlotPool::new(4096 * 132 * 4, &shm).expect("Failed to create pool");
    
    let mut state = BarState {
        registry_state: RegistryState::new(&globals),
        seat_state,
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer_surface,
        width: 0,
        height: BAR_HEIGHT,
        configured: false,
        running: true,
        status: StatusData::default(),
        last_update: Instant::now() - UPDATE_INTERVAL,
    };
    
    eprintln!("🌲 faelight-bar v3.0 minimal starting...");
    event_queue.roundtrip(&mut state).expect("Failed initial roundtrip");
    
    if !state.configured {
        eprintln!("❌ Never received configure event!");
        std::process::exit(1);
    }
    
    eprintln!("✅ Bar started (status updates every {}s)", UPDATE_INTERVAL.as_secs());
    
    while state.running {
        event_queue.blocking_dispatch(&mut state).ok();
    }
}
