//! faelight-term v2 -- Phase 0: Foundation
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
use cosmic_text::{
    Attrs, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache,
};
const INITIAL_COLS:   usize = 220;
const INITIAL_ROWS:   usize = 50;
const INITIAL_WIDTH:  u32   = 1760;
const INITIAL_HEIGHT: u32   = 900;
const FONT_SIZE:      f32   = 14.0;
const LINE_HEIGHT:    f32   = 20.0;
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
    let compositor     = CompositorState::bind(&globals, &qh)?;
    let xdg_shell      = XdgShell::bind(&globals, &qh)?;
    let seat_state     = SeatState::new(&globals, &qh);
    let output_state   = OutputState::new(&globals, &qh);
    let registry_state = RegistryState::new(&globals);
    let shm            = Shm::bind(&globals, &qh)?;
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
    // cosmic-text setup -- load Nerd Font explicitly
    let mut font_system = FontSystem::new();
    // Load JetBrainsMono Nerd Font from disk for full Unicode + icon coverage
    if std::path::Path::new(config::FONT_REGULAR).exists() {
        // Load Mono variant FIRST -- it has full Nerd Font icon coverage
        font_system.db_mut().load_font_file(config::FONT_MONO_REGULAR).ok();
        font_system.db_mut().load_font_file(config::FONT_REGULAR).ok();
        font_system.db_mut().load_font_file(config::FONT_BOLD).ok();
        font_system.db_mut().load_font_file(config::FONT_ITALIC).ok();
        // Load NotoColorEmoji for tree and other emoji
        if std::path::Path::new(config::FONT_EMOJI).exists() {
            font_system.db_mut().load_font_file(config::FONT_EMOJI).ok();
        }
        eprintln!("faelight-term: Nerd Font loaded from {}", config::FONT_REGULAR);


    } else {
        eprintln!("faelight-term: WARNING -- Nerd Font not found, falling back to system fonts");
    }

    let swash_cache = SwashCache::new();
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
        font_system,
        swash_cache,
        width:      INITIAL_WIDTH,
        height:     INITIAL_HEIGHT,
        cell_w:     10u32, // will be measured from font metrics on first render
        cell_h:     LINE_HEIGHT as u32,
        configured: false,
        running:    true,
        keyboard:   None,
        pointer:    None,
    };
    let mut event_loop: EventLoop<App> = EventLoop::try_new()?;
    WaylandSource::new(conn, event_queue).insert(event_loop.handle())?;
    while app.running {
        event_loop.dispatch(Some(std::time::Duration::from_millis(8)), &mut app)?;
        let mut dirty = false;
        loop {
            let mut buf = [0u8; 4096];
            match app.pty.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => { app.terminal.feed(&buf[..n]); dirty = true; }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => { eprintln!("PTY error: {}", e); app.running = false; break; }
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
    font_system:    FontSystem,
    swash_cache:    SwashCache,
    width:          u32,
    height:         u32,
    cell_w:         u32,
    cell_h:         u32,
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
        if let Ok((buffer, canvas)) = self.pool.create_buffer(
            width as i32, height as i32, stride as i32,
            wl_shm::Format::Xrgb8888,
        ) {
            // Fill background
            for pixel in canvas.chunks_exact_mut(4) {
                pixel[0] = 0x11; pixel[1] = 0x14;
                pixel[2] = 0x0f; pixel[3] = 0xff;
            }
            // Draw terminal cells
            let rows = self.terminal.rows;
            let cols = self.terminal.cols;
            let cell_w = self.cell_w;
            let cell_h = self.cell_h;
            for row in 0..rows {
                for col in 0..cols {
                    let cell = self.terminal.grid[row][col];
                    if cell.ch == ' ' || cell.ch == '\0' { continue; }
                    let cell_x = (col as u32 * cell_w) as i32;
                    let cell_y = (row as u32 * cell_h) as i32;
                    if cell_x + cell_w as i32 > width as i32 { continue; }
                    if cell_y + cell_h as i32 > height as i32 { continue; }
                    // Get fg color from palette
                    let fg_idx = cell.fg as usize % 16;
                    let fg = self.config.colors[fg_idx];
                    let fg_r = (fg[0] * 255.0) as u8;
                    let fg_g = (fg[1] * 255.0) as u8;
                    let fg_b = (fg[2] * 255.0) as u8;
                    // Shape and rasterize with cosmic-text
                    let mut text_buf = Buffer::new(
                        &mut self.font_system,
                        Metrics::new(FONT_SIZE, LINE_HEIGHT),
                    );
                    text_buf.set_size(&mut self.font_system,
                        Some(cell_w as f32), Some(cell_h as f32));
                    let attrs = Attrs::new().family(cosmic_text::Family::Name("JetBrainsMono Nerd Font Mono"));
                    let text = cell.ch.to_string();
                    text_buf.set_text(&mut self.font_system, &text, attrs, Shaping::Basic);
                    text_buf.shape_until_scroll(&mut self.font_system, false);
                    // Rasterize glyphs using layout_runs + physical + with_pixels
                    let base_color = Color::rgb(fg_r, fg_g, fg_b);
                    for run in text_buf.layout_runs() {
                        for glyph in run.glyphs.iter() {
                            let phys = glyph.physical((0.0, 0.0), 1.0);
                            let gx = cell_x + phys.x;
                            let gy = cell_y + run.line_y as i32 + phys.y;
                            self.swash_cache.with_pixels(
                                &mut self.font_system,
                                phys.cache_key,
                                base_color,
                                |px_off, py_off, color| {
                                    let px = gx + px_off;
                                    let py = gy + py_off;
                                    if px < 0 || py < 0 { return; }
                                    let px = px as u32;
                                    let py = py as u32;
                                    if px >= width || py >= height { return; }
                                    let alpha = color.a();
                                    if alpha == 0 { return; }
                                    let offset = (py * stride + px * 4) as usize;
                                    if offset + 3 >= canvas.len() { return; }
                                    if alpha == 255 {
                                        // Fully opaque -- write directly (color emoji, solid glyphs)
                                        canvas[offset]     = color.b();
                                        canvas[offset + 1] = color.g();
                                        canvas[offset + 2] = color.r();
                                        canvas[offset + 3] = 0xff;
                                    } else {
                                        // Alpha blend
                                        let a = alpha as u32;
                                        let inv = 255 - a;
                                        canvas[offset]     = ((canvas[offset]     as u32 * inv + color.b() as u32 * a) / 255) as u8;
                                        canvas[offset + 1] = ((canvas[offset + 1] as u32 * inv + color.g() as u32 * a) / 255) as u8;
                                        canvas[offset + 2] = ((canvas[offset + 2] as u32 * inv + color.r() as u32 * a) / 255) as u8;
                                        canvas[offset + 3] = 0xff;
                                    }
                                }
                            );
                        }
                    }
                }
            }
            // Draw cursor
            let cx = (self.terminal.cursor_x as u32 * cell_w) as usize;
            let cy = (self.terminal.cursor_y as u32 * cell_h) as usize;
            for dy in 0..cell_h as usize {
                for dx in 0..2usize {
                    let px = cx + dx;
                    let py = cy + dy;
                    if px < width as usize && py < height as usize {
                        let offset = (py * stride as usize + px * 4) as usize;
                        if offset + 3 < canvas.len() {
                            canvas[offset]     = 0xa3;
                            canvas[offset + 1] = 0xe3;
                            canvas[offset + 2] = 0x6b;
                            canvas[offset + 3] = 0xff;
                        }
                    }
                }
            }
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
            // Resize pool
            let needed = (self.width * self.height * 4) as usize;
            if let Err(e) = self.pool.resize(needed) {
                eprintln!("pool resize error: {}", e);
            }
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
