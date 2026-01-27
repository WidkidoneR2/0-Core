use std::time::{Duration, Instant};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, 
    delegate_pointer, delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler}
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
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};
use std::process::Command;
use std::fs;
use std::env;

mod state;
mod render;
mod input;
mod menu;

use state::{AppState, ModeState};
use input::{handle_key_press, KeyAction, execute_command};

const BAR_HEIGHT: u32 = 32;
const REFRESH_MS: u64 = 500;

fn handle_click(action: &str) {
    match action {
        "vpn" => {
            if let Ok(out) = Command::new("mullvad").arg("status").output() {
                let result = String::from_utf8_lossy(&out.stdout);
                if result.contains("Connected") {
                    let _ = Command::new("mullvad").arg("disconnect").spawn();
                } else {
                    let _ = Command::new("mullvad").arg("connect").spawn();
                }
            }
        }
        "volume" => {
            let _ = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .spawn();
        }
        "profile" => {
            // Cycle profiles
            let current = get_current_profile();
            let next = match current.as_str() {
                "default" => "gaming",
                "gaming" => "work",
                "work" => "low-power",
                _ => "default",
            };
            let _ = Command::new("profile").arg(next).spawn();
        }
        _ => {}
    }
}

fn get_current_profile() -> String {
    let home = env::var("HOME").unwrap_or_default();
    let path = format!("{}/.local/state/0-core/current-profile", home);
    fs::read_to_string(&path)
        .unwrap_or_else(|_| "default".to_string())
        .trim()
        .to_string()
}

fn health_check() {
    println!("🏥 faelight-bar health check");
    
    match Connection::connect_to_env() {
        Ok(_) => println!("✅ wayland: connected"),
        Err(e) => {
            eprintln!("❌ wayland: failed - {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n✅ Core checks passed!");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("faelight-bar v2.0.0");
                std::process::exit(0);
            }
            "--help" | "-h" => {
                println!("faelight-bar v2.0.0 - Hybrid Bar/Menu for Faelight Forest");
                println!();
                println!("Wayland status bar with integrated application launcher");
                println!();
                println!("USAGE: faelight-bar [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    -h, --help          Show this help");
                println!("    -v, --version       Show version");
                println!("    --health-check      Verify dependencies");
                println!();
                println!("INTERACTION:");
                println!("    Click profile       Cycle power profiles");
                println!("    Click launcher 🚀   Open application menu");
                println!("    Click VPN           Toggle connection");
                println!("    Click volume        Toggle mute");
                println!();
                println!("MENU MODE:");
                println!("      Type              Filter applications");
                println!("      Up/Down           Navigate");
                println!("      Enter             Launch selected");
                println!("      Escape            Return to bar");
                std::process::exit(0);
            }
            "--health-check" => {
                health_check();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Try faelight-bar --help");
                std::process::exit(1);
            }
        }
    }
    
    let conn = match Connection::connect_to_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to connect to Wayland: {}", e);
            eprintln!("💡 Make sure you're running under Wayland/Sway");
            std::process::exit(1);
        }
    };
    
    let (globals, mut event_queue) = match registry_queue_init(&conn) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("❌ Failed to init registry: {}", e);
            std::process::exit(1);
        }
    };
    
    let qh = event_queue.handle();
    
    let compositor = match CompositorState::bind(&globals, &qh) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ wl_compositor not available: {}", e);
            std::process::exit(1);
        }
    };
    
    let layer_shell = match LayerShell::bind(&globals, &qh) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("❌ layer shell not available: {}", e);
            eprintln!("💡 Make sure wlr-layer-shell protocol is supported");
            std::process::exit(1);
        }
    };
    
    let shm = match Shm::bind(&globals, &qh) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ wl_shm not available: {}", e);
            std::process::exit(1);
        }
    };
    
    let seat_state = SeatState::new(&globals, &qh);
    let surface = compositor.create_surface(&qh);
    
    let layer_surface = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Top,
        Some("faelight-bar"),
        None,
    );
    
    layer_surface.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer_surface.set_size(0, BAR_HEIGHT);
    layer_surface.set_exclusive_zone(BAR_HEIGHT as i32);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer_surface.commit();
    
    let pool = match SlotPool::new(4096 * 132 * 4, &shm) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Failed to create pool: {}", e);
            std::process::exit(1);
        }
    };
    
    let mut state = BarState {
        registry_state: RegistryState::new(&globals),
        seat_state,
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer_surface,
        app_state: AppState::new(),
        height: BAR_HEIGHT,
        configured: false,
        running: true,
        click_regions: Vec::new(),
        pointer_x: 0.0,
        last_draw: Instant::now(),
        keyboard: None,
    };
    
    println!("🌲 faelight-bar v2.0.0 starting (Hybrid Edition)...");
    
    while state.running {
        if let Err(e) = event_queue.blocking_dispatch(&mut state) {
            eprintln!("⚠️  Event dispatch error: {}", e);
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
    app_state: AppState,
    height: u32,
    configured: bool,
    running: bool,
    click_regions: Vec<(i32, i32, String)>,
    pointer_x: f64,
    last_draw: Instant,
    keyboard: Option<wl_keyboard::WlKeyboard>,
}

impl BarState {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        if self.app_state.width == 0 {
            return;
        }
        
        let width = self.app_state.width;
        let height = self.height;
        let stride = width as i32 * 4;
        
        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(_) => return,
        };
        
        // Render and get click regions
        self.click_regions = render::render(&self.app_state, canvas, width, height);
        
        self.layer_surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer_surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        self.layer_surface.wl_surface().frame(qh, self.layer_surface.wl_surface().clone());
        self.layer_surface.commit();
    }
    
    fn enter_menu_mode(&mut self, qh: &QueueHandle<Self>) {
        let items = menu::get_all_items();
        self.app_state.enter_menu(items, &self.layer_surface);
        self.height = self.app_state.current_height();
        self.draw(qh);
    }
    
    fn exit_menu_mode(&mut self, qh: &QueueHandle<Self>) {
        self.app_state.exit_menu(&self.layer_surface);
        self.height = self.app_state.current_height();
        self.draw(qh);
    }
}

impl CompositorHandler for BarState {
    fn scale_factor_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, 
                           _surface: &wl_surface::WlSurface, _new_factor: i32) {}
    
    fn transform_changed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, 
                        _surface: &wl_surface::WlSurface, _new_transform: wl_output::Transform) {}
    
    fn frame(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, 
             _surface: &wl_surface::WlSurface, _time: u32) {
        if self.last_draw.elapsed() >= Duration::from_millis(REFRESH_MS) {
            self.last_draw = Instant::now();
            self.draw(qh);
        } else {
            self.layer_surface.wl_surface().frame(qh, self.layer_surface.wl_surface().clone());
            self.layer_surface.wl_surface().commit();
        }
    }
    
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, 
                     _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, 
                     _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
}

impl OutputHandler for BarState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    
    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    
    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
}

impl LayerShellHandler for BarState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.running = false;
    }
    
    fn configure(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, _layer: &LayerSurface, 
                 configure: LayerSurfaceConfigure, _serial: u32) {
        if configure.new_size.0 > 0 {
            self.app_state.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }
        self.configured = true;
        self.draw(qh);
    }
}

impl ShmHandler for BarState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SeatHandler for BarState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    
    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
    
    fn new_capability(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, 
                     seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Pointer {
            let _ = self.seat_state.get_pointer(qh, &seat);
        }
        if capability == Capability::Keyboard {
            if let Ok(kbd) = self.seat_state.get_keyboard(qh, &seat, None) {
                self.keyboard = Some(kbd);
            }
        }
    }
    
    fn remove_capability(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, 
                        _seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard {
            if let Some(kbd) = self.keyboard.take() {
                kbd.release();
            }
        }
    }
    
    fn remove_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
}

impl KeyboardHandler for BarState {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard,
             _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[Keysym]) {}
    
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard,
             _: &wl_surface::WlSurface, _: u32) {}
    
    fn press_key(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, 
                 _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        let action = handle_key_press(&mut self.app_state, event);
        
        match action {
            KeyAction::None => {}
            KeyAction::Redraw => {
                self.draw(qh);
            }
            KeyAction::ExitMenu => {
                self.exit_menu_mode(qh);
            }
            KeyAction::Execute(cmd) => {
                let _ = execute_command(&cmd);
                self.exit_menu_mode(qh);
            }
        }
    }
    
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, 
                   _: &wl_keyboard::WlKeyboard, _: u32, _event: KeyEvent) {}
    
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, 
                       _: &wl_keyboard::WlKeyboard, _serial: u32, _modifiers: Modifiers, _: u32) {}
}

impl PointerHandler for BarState {
    fn pointer_frame(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, 
                    _pointer: &wl_pointer::WlPointer, events: &[PointerEvent]) {
        for event in events {
            match event.kind {
                PointerEventKind::Motion { .. } => {
                    self.pointer_x = event.position.0;
                }
                PointerEventKind::Press { button, .. } => {
                    if button == 272 {  // Left click
                        let x = self.pointer_x as i32;
                        for (start, end, action) in &self.click_regions {
                            if x >= *start && x <= *end {
                                if action == "launcher" {
                                    self.enter_menu_mode(qh);
                                } else {
                                    handle_click(action);
                                }
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl ProvidesRegistryState for BarState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
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
