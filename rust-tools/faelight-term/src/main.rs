//! faelight-term v2 -- Phase 0: Foundation
#![allow(dead_code, unused_imports, unused_variables)]
mod config;
mod renderer;
mod terminal;
mod input;
mod pty;
use config::Config;
use terminal::Terminal;
use pty::Pty;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::calloop::EventLoop,
    reexports::calloop_wayland_source::WaylandSource,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers, RepeatInfo},
        pointer::{PointerEvent, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        xdg::{
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
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
const INITIAL_COLS: usize = 220;
const INITIAL_ROWS: usize = 50;
const INITIAL_WIDTH: u32  = 1760;
const INITIAL_HEIGHT: u32 = 900;
fn main() {
    let config = Config::load();
    eprintln!("faelight-term v2 -- starting");
    if let Err(e) = run(config) {
        eprintln!("fatal: {}", e);
        std::process::exit(1);
    }
}
fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();
    let compositor    = CompositorState::bind(&globals, &qh)?;
    let xdg_shell     = XdgShell::bind(&globals, &qh)?;
    let seat_state    = SeatState::new(&globals, &qh);
    let output_state  = OutputState::new(&globals, &qh);
    let registry_state = RegistryState::new(&globals);
    let shm           = Shm::bind(&globals, &qh)?;
    let surface = compositor.create_surface(&qh);
    let window  = xdg_shell.create_window(
        surface.clone(), WindowDecorations::RequestServer, &qh,
    );
    window.set_title("faelight-term");
    window.set_app_id("faelight-term");
    window.commit();
    let pty      = Pty::spawn(&config.shell, INITIAL_COLS as u16, INITIAL_ROWS as u16)?;
    let terminal = Terminal::new(INITIAL_COLS, INITIAL_ROWS);
    let pool     = SlotPool::new((INITIAL_WIDTH * INITIAL_HEIGHT * 4) as usize, &shm)?;
    let mut app = App {
        config,
        compositor,
        xdg_shell,
        seat_state,
        output_state,
        registry_state,
        shm,
        window,
        surface,
        pool,
        terminal,
        pty,
        width:      INITIAL_WIDTH,
        height:     INITIAL_HEIGHT,
        configured: false,
        running:    true,
        keyboard:   None,
        pointer:    None,
    };
    let mut event_loop: EventLoop<App> = EventLoop::try_new()?;
    WaylandSource::new(conn, event_queue).insert(event_loop.handle())?;
    while app.running {
        event_loop.dispatch(Some(std::time::Duration::from_millis(8)), &mut app)?;
        // Drain all available PTY output
        let mut dirty = false;
        loop {
            let mut buf = [0u8; 4096];
            match app.pty.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    app.terminal.feed(&buf[..n]);
                    dirty = true;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => { eprintln!("PTY read error: {:?}", e); app.running = false; break; }
            }
        }
        if dirty && app.configured {
            app.render();
        }
    }
    Ok(())
}
struct App {
    config:         Config,
    compositor:     CompositorState,
    xdg_shell:      XdgShell,
    seat_state:     SeatState,
    output_state:   OutputState,
    registry_state: RegistryState,
    shm:            Shm,
    window:         Window,
    surface:        wl_surface::WlSurface,
    pool:           SlotPool,
    terminal:       Terminal,
    pty:            Pty,
    width:          u32,
    height:         u32,
    configured:     bool,
    running:        bool,
    keyboard:       Option<wl_keyboard::WlKeyboard>,
    pointer:        Option<wl_pointer::WlPointer>,
}
impl App {
    fn render(&mut self) {
        let width  = self.width;
        let height = self.height;
        let stride = width * 4;
        let _size  = (stride * height) as usize;
        if let Ok((buffer, canvas)) = self.pool.create_buffer(
            width as i32, height as i32, stride as i32,
            wl_shm::Format::Xrgb8888,
        ) {
            // Fill background
            for pixel in canvas.chunks_exact_mut(4) {
                pixel[0] = 0x11; // B
                pixel[1] = 0x14; // G
                pixel[2] = 0x0f; // R
                pixel[3] = 0xff; // X
            }
            // Phase 0: background only.
            // Cell rendering (cosmic-text glyph atlas) comes in next gate.
            self.surface.attach(Some(buffer.wl_buffer()), 0, 0);
            self.surface.damage_buffer(0, 0, width as i32, height as i32);
            self.surface.commit();
        }
    }
}
impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm }
}
impl CompositorHandler for App {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: u32) {}
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
}
impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}
impl WindowHandler for App {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.running = false;
    }
    fn configure(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &Window, configure: WindowConfigure, _: u32)
    {
        if let (Some(w), Some(h)) = configure.new_size {
            self.width  = w.get() as u32;
            self.height = h.get() as u32;
        }
        self.configured = true;
        self.render();
    }
}
impl SeatHandler for App {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat, cap: Capability)
    {
        if cap == Capability::Keyboard && self.keyboard.is_none() {
            self.keyboard = Some(self.seat_state.get_keyboard(qh, &seat, None).unwrap());
        }
        if cap == Capability::Pointer && self.pointer.is_none() {
            self.pointer = Some(self.seat_state.get_pointer(qh, &seat).unwrap());
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}
impl KeyboardHandler for App {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32,
        _: &[u32], _: &[Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent)
    {
        if let Some(bytes) = keysym_to_bytes(event.keysym, &event.utf8) {
            self.pty.write(&bytes).ok();
        }
    }
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, _: Modifiers, _: RawModifiers, _: u32) {}
    fn update_repeat_info(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: RepeatInfo) {}
    fn repeat_key(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
}
impl PointerHandler for App {
    fn pointer_frame(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_pointer::WlPointer, _: &[PointerEvent]) {}
}
impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState, SeatState];
}
fn keysym_to_bytes(keysym: Keysym, utf8: &Option<String>) -> Option<Vec<u8>> {
    if let Some(t) = utf8 {
        if !t.is_empty() { return Some(t.as_bytes().to_vec()); }
    }
    match keysym {
        Keysym::Return    => Some(b"\r".to_vec()),
        Keysym::BackSpace => Some(b"\x7f".to_vec()),
        Keysym::Tab       => Some(b"\t".to_vec()),
        Keysym::Escape    => Some(b"\x1b".to_vec()),
        Keysym::Up        => Some(b"\x1b[A".to_vec()),
        Keysym::Down      => Some(b"\x1b[B".to_vec()),
        Keysym::Right     => Some(b"\x1b[C".to_vec()),
        Keysym::Left      => Some(b"\x1b[D".to_vec()),
        Keysym::Home      => Some(b"\x1b[H".to_vec()),
        Keysym::End       => Some(b"\x1b[F".to_vec()),
        Keysym::Delete    => Some(b"\x1b[3~".to_vec()),
        Keysym::Page_Up   => Some(b"\x1b[5~".to_vec()),
        Keysym::Page_Down => Some(b"\x1b[6~".to_vec()),
        _ => None,
    }
}
smithay_client_toolkit::delegate_shm!(App);
delegate_compositor!(App);
delegate_output!(App);
delegate_xdg_shell!(App);
delegate_xdg_window!(App);
delegate_seat!(App);
delegate_keyboard!(App);
delegate_pointer!(App);
delegate_registry!(App);
