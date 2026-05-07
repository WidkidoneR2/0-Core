//! faelight-lock v2.0.0 -- Native Rust Wayland Screen Locker
//! ext-session-lock-v1 protocol. No wrapper. The forest locks itself.
use fontdue::{Font, FontSettings};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_registry, delegate_seat,
    delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers, RepeatInfo},
        Capability, SeatHandler, SeatState,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols::ext::session_lock::v1::client::{
    ext_session_lock_manager_v1::ExtSessionLockManagerV1,
    ext_session_lock_surface_v1::{self, ExtSessionLockSurfaceV1},
    ext_session_lock_v1::{self, ExtSessionLockV1},
};
// Forest palette (BGRA bytes for Argb8888 format)
const BG: [u8; 4]    = [0x0f, 0x14, 0x11, 0xff];
const FG: [u8; 4]    = [0xd8, 0xe0, 0xd7, 0xff];
const GREEN: [u8; 4] = [0x6b, 0xe3, 0xa3, 0xff];
const RED: [u8; 4]   = [0x6b, 0x6b, 0xff, 0xff];
const DIM: [u8; 4]   = [0x50, 0x60, 0x55, 0xff];
#[allow(dead_code)]
struct LockSurface {
    surface:      wl_surface::WlSurface,
    lock_surface: ExtSessionLockSurfaceV1,
    width:        u32,
    height:       u32,
    configured:   bool,
}
#[allow(dead_code)]
struct LockApp {
    running:        bool,
    registry_state: RegistryState,
    compositor:     CompositorState,
    shm:            Shm,
    output_state:   OutputState,
    seat_state:     SeatState,
    pool:           Option<SlotPool>,
    lock_manager:   Option<ExtSessionLockManagerV1>,
    lock:           Option<ExtSessionLockV1>,
    surfaces:       Vec<LockSurface>,
    keyboards:      Vec<wl_keyboard::WlKeyboard>,
    password:       String,
    error:          Option<String>,
    error_since:    Option<std::time::Instant>,
    username:       String,
    locked:         bool,
    needs_redraw:   bool,
    font:           Font,
}
impl LockApp {
    fn auth_helper_path() -> String {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{}/0-core/scripts/faelight-lock-auth", home)
    }
    fn try_unlock(&mut self, _qh: &QueueHandle<Self>) {
        if self.password.is_empty() { return; }
        let path = Self::auth_helper_path();
        let input = format!("{}\n{}\n", self.username, self.password);
        self.password.clear();
        let result = std::process::Command::new(&path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = stdin.write_all(input.as_bytes());
                }
                child.wait_with_output()
            });
        match result {
            Ok(out) if String::from_utf8_lossy(&out.stdout).trim() == "OK" => {
                if let Some(lock) = &self.lock {
                    lock.unlock_and_destroy();
                }
                self.running = false;
            }
            _ => {
                self.error = Some("Incorrect password".to_string());
                self.error_since = Some(std::time::Instant::now());
                self.needs_redraw = true;
            }
        }
    }
    fn render_all(&mut self, _qh: &QueueHandle<Self>) {
        if let Some(since) = self.error_since {
            if since.elapsed().as_secs() >= 3 {
                self.error = None;
                self.error_since = None;
            }
        }
        let pass_dots = "\u{2022}".repeat(self.password.len());
        let error = self.error.clone();
        let version = std::fs::read_to_string("/etc/faelight/VERSION")
            .unwrap_or_else(|_| "13.0.0".to_string());
        let commits = std::fs::read_to_string("/etc/faelight/COMMITS")
            .unwrap_or_else(|_| "?".to_string());
        let w_list: Vec<(u32, u32)> = self.surfaces.iter()
            .map(|s| (s.width, s.height))
            .collect();
        for (i, &(w, h)) in w_list.iter().enumerate() {
            if !self.surfaces[i].configured || w == 0 || h == 0 { continue; }
            let pool = match &mut self.pool {
                Some(p) => p,
                None => continue,
            };
            let (buffer, canvas) = match pool.create_buffer(
                w as i32, h as i32, (w * 4) as i32,
                wl_shm::Format::Argb8888,
            ) {
                Ok(b) => b,
                Err(_) => continue,
            };
            // Background
            for px in canvas.chunks_exact_mut(4) { px.copy_from_slice(&BG); }
            // Content
            let cx = (w / 2) as i32;
            let cy = (h / 2) as i32;
            // Title
            draw_text(canvas, w, cx - 130, cy - 100,
                "Faelight Forest", 32.0, &GREEN, &self.font);
            // Version + commits
            let info = format!("{}  \u{00b7}  {} commits",
                version.trim(), commits.trim());
            draw_text(canvas, w, cx - 100, cy - 58,
                &info, 16.0, &DIM, &self.font);
            // Password field
            let (pwd_text, pwd_color) = if pass_dots.is_empty() {
                ("Enter password", &DIM)
            } else {
                (pass_dots.as_str(), &FG as &[u8; 4])
            };
            draw_text(canvas, w, cx - 130, cy - 5,
                pwd_text, 24.0, pwd_color, &self.font);
            // Underline
            draw_hline(canvas, w, cx - 130, cy + 28, 260, &DIM);
            // Error
            if let Some(ref err) = error {
                draw_text(canvas, w, cx - 100, cy + 60,
                    err, 16.0, &RED, &self.font);
            }
            // Hint
            draw_text(canvas, w, cx - 130, cy + 100,
                "Enter  unlock     Esc  clear", 13.0, &DIM, &self.font);
            self.surfaces[i].surface.attach(Some(buffer.wl_buffer()), 0, 0);
            self.surfaces[i].surface.damage_buffer(0, 0, w as i32, h as i32);
            self.surfaces[i].surface.commit();
        }
        self.needs_redraw = false;
    }
}
fn draw_text(canvas: &mut [u8], w: u32, x: i32, y: i32,
             text: &str, size: f32, color: &[u8; 4], font: &Font) {
    use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: 0.0, y: 0.0,
        max_width: Some(w as f32),
        max_height: Some(200.0),
        ..LayoutSettings::default()
    });
    layout.append(&[font], &TextStyle::new(text, size, 0));
    let stride = w as i32;
    for glyph in layout.glyphs() {
        let (metrics, bitmap) = font.rasterize_config(glyph.key);
        for (i, &alpha) in bitmap.iter().enumerate() {
            if alpha < 10 { continue; }
            let gx = i % metrics.width;
            let gy = i / metrics.width;
            let px = x + glyph.x as i32 + gx as i32;
            let py = y + glyph.y as i32 + gy as i32 - 4;
            if px < 0 || py < 0 || px >= w as i32 { continue; }
            let off = (py * stride + px) as usize * 4;
            if off + 3 >= canvas.len() { continue; }
            let alpha_f = (alpha as f32 / 255.0).powf(1.0 / 2.2);
            let inv = 1.0 - alpha_f;
            canvas[off]   = (color[0] as f32 * alpha_f + canvas[off]   as f32 * inv) as u8;
            canvas[off+1] = (color[1] as f32 * alpha_f + canvas[off+1] as f32 * inv) as u8;
            canvas[off+2] = (color[2] as f32 * alpha_f + canvas[off+2] as f32 * inv) as u8;
            canvas[off+3] = 0xff;
        }
    }
}
fn draw_hline(canvas: &mut [u8], w: u32, x: i32, y: i32, len: i32, color: &[u8; 4]) {
    if y < 0 { return; }
    for i in 0..len {
        let px = x + i;
        if px < 0 || px >= w as i32 { continue; }
        let off = (y as u32 * w + px as u32) as usize * 4;
        if off + 3 < canvas.len() { canvas[off..off+4].copy_from_slice(color); }
    }
}
// Dispatch for ext-session-lock-v1 protocol
impl Dispatch<ExtSessionLockManagerV1, ()> for LockApp {
    fn event(_: &mut Self, _: &ExtSessionLockManagerV1,
             _: <ExtSessionLockManagerV1 as wayland_client::Proxy>::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}
impl Dispatch<ExtSessionLockV1, ()> for LockApp {
    fn event(state: &mut Self, _: &ExtSessionLockV1,
             event: ext_session_lock_v1::Event,
             _: &(), _: &Connection, _: &QueueHandle<Self>) {
        match event {
            ext_session_lock_v1::Event::Locked => {
                state.locked = true;
                state.needs_redraw = true;
            }
            ext_session_lock_v1::Event::Finished => {
                eprintln!("faelight-lock: compositor rejected lock");
                state.running = false;
            }
            _ => {}
        }
    }
}
impl Dispatch<ExtSessionLockSurfaceV1, usize> for LockApp {
    fn event(state: &mut Self, surf: &ExtSessionLockSurfaceV1,
             event: ext_session_lock_surface_v1::Event,
             idx: &usize, _: &Connection, _: &QueueHandle<Self>) {
        if let ext_session_lock_surface_v1::Event::Configure { serial, width, height } = event {
            surf.ack_configure(serial);
            if let Some(s) = state.surfaces.get_mut(*idx) {
                s.width = width;
                s.height = height;
                s.configured = true;
            }
            state.needs_redraw = true;
        }
    }
}
// Keyboard handler -- password input
impl KeyboardHandler for LockApp {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>,
             _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface,
             _: u32, _: &[u32], _: &[Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>,
             _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    fn press_key(&mut self, _: &Connection, qh: &QueueHandle<Self>,
                 _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        match event.keysym {
            Keysym::Return | Keysym::KP_Enter => { self.try_unlock(qh); }
            Keysym::BackSpace => {
                self.password.pop();
                self.needs_redraw = true;
            }
            Keysym::Escape => {
                self.password.clear();
                self.error = None;
                self.needs_redraw = true;
            }
            _ => {
                if let Some(s) = event.utf8 {
                    for ch in s.chars() {
                        if !ch.is_control() { self.password.push(ch); }
                    }
                    self.needs_redraw = true;
                }
            }
        }
    }
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>,
                   _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>,
                        _: &wl_keyboard::WlKeyboard, _: u32,
                        _: Modifiers, _: RawModifiers, _: u32) {}
    fn update_repeat_info(&mut self, _: &Connection, _: &QueueHandle<Self>,
                          _: &wl_keyboard::WlKeyboard, _: RepeatInfo) {}
    fn repeat_key(&mut self, _: &Connection, _: &QueueHandle<Self>,
                  _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
}
impl SeatHandler for LockApp {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>,
                      seat: wl_seat::WlSeat, cap: Capability) {
        if cap == Capability::Keyboard {
            if let Ok(kb) = self.seat_state.get_keyboard(qh, &seat, None) {
                self.keyboards.push(kb);
            }
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>,
                         _: wl_seat::WlSeat, _: Capability) {}
}
impl CompositorHandler for LockApp {
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
impl OutputHandler for LockApp {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}
impl ShmHandler for LockApp {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm }
}
impl ProvidesRegistryState for LockApp {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState, SeatState];
}
delegate_compositor!(LockApp);
delegate_output!(LockApp);
delegate_shm!(LockApp);
delegate_seat!(LockApp);
delegate_keyboard!(LockApp);
delegate_registry!(LockApp);
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|a| a == "--health") {
        println!("faelight-lock v2.0.0 -- healthy");
        return Ok(());
    }
    let username = std::env::var("USER").unwrap_or_else(|_| "christian".to_string());
    let font = Font::from_bytes(
        include_bytes!("/usr/share/fonts/TTF/HackNerdFont-Regular.ttf") as &[u8],
        FontSettings::default(),
    ).expect("Failed to load font");
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init::<LockApp>(&conn)?;
    let qh = event_queue.handle();
    let compositor = CompositorState::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    let output_state = OutputState::new(&globals, &qh);
    let seat_state = SeatState::new(&globals, &qh);
    let registry_state = RegistryState::new(&globals);
    // Bind ext_session_lock_manager_v1
    let lock_manager: ExtSessionLockManagerV1 = globals.bind(&qh, 1..=1, ())?;
    // Pool: 4K * 4 outputs * BGRA
    let pool = SlotPool::new(3840 * 2160 * 4 * 4, &shm)?;
    let mut app = LockApp {
        running: true,
        registry_state,
        compositor,
        shm,
        output_state,
        seat_state,
        pool: Some(pool),
        lock_manager: Some(lock_manager.clone()),
        lock: None,
        surfaces: Vec::new(),
        keyboards: Vec::new(),
        password: String::new(),
        error: None,
        error_since: None,
        username,
        locked: false,
        needs_redraw: false,
        font,
    };
    // Roundtrip to discover outputs and seats
    event_queue.roundtrip(&mut app)?;
    // Lock the session
    let lock = lock_manager.lock(&qh, ());
    app.lock = Some(lock.clone());
    // Create lock surface for each output
    let outputs: Vec<wl_output::WlOutput> = app.output_state.outputs().collect();
    for (idx, output) in outputs.iter().enumerate() {
        let surface = app.compositor.create_surface(&qh);
        let lock_surface = lock.get_lock_surface(&surface, output, &qh, idx);
        app.surfaces.push(LockSurface {
            surface,
            lock_surface,
            width: 0,
            height: 0,
            configured: false,
        });
    }
    // Event loop
    while app.running {
        event_queue.blocking_dispatch(&mut app)?;
        if app.needs_redraw && app.locked {
            let qh = event_queue.handle();
            app.render_all(&qh);
        }
    }
    // Flush pending requests (sends unlock_and_destroy to compositor)
    let _ = event_queue.flush();
    Ok(())
}
