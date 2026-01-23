mod pty;
use std::time::Duration;
use smithay_client_toolkit::{
    reexports::calloop::EventLoop,
    reexports::calloop_wayland_source::WaylandSource,
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_registry, delegate_seat,
    delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Modifiers, RawModifiers},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        xdg::{
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
        },
        WaylandSurface,
    },
    shm::{slot::{Buffer, SlotPool}, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};
use fontdue::Font;

const FONT_DATA: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");

fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if let Some('[') = chars.next() {
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() { break; }
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

fn main() {
    println!("🦀 faelight-term - INTERACTIVE!");
    
    let conn = Connection::connect_to_env().unwrap();
    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();
    
    let mut event_loop: EventLoop<App> = EventLoop::try_new().unwrap();
    WaylandSource::new(conn.clone(), event_queue).insert(event_loop.handle()).unwrap();
    
    let compositor = CompositorState::bind(&globals, &qh).unwrap();
    let xdg_shell = XdgShell::bind(&globals, &qh).unwrap();
    let shm = Shm::bind(&globals, &qh).unwrap();
    
    let surface = compositor.create_surface(&qh);
    let window = xdg_shell.create_window(surface, WindowDecorations::ServerDefault, &qh);
    
    window.set_title("faelight-term");
    window.set_app_id("faelight-term");
    window.commit();
    
    let pool = SlotPool::new(800 * 600 * 4, &shm).unwrap();
    let font = Font::from_bytes(FONT_DATA, fontdue::FontSettings::default()).unwrap();
    let pty = pty::Pty::spawn_shell(24, 80).unwrap();
    
    println!("✅ Type commands and press Enter!");
    
    let mut app = App {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        exit: false,
        first_configure: true,
        pool,
        width: 800,
        height: 600,
        buffer: None,
        window,
        font,
        pty,
        output_text: String::new(),
        keyboard: None,
    };
    
    loop {
        let mut buf = [0u8; 4096];
        match app.pty.read(&mut buf) {
            Ok(n) if n > 0 => {
                if let Ok(text) = std::str::from_utf8(&buf[..n]) {
                    let clean = strip_ansi(text);
                    app.output_text.push_str(&clean);
                    if app.output_text.len() > 5000 {
                        app.output_text = app.output_text.chars().skip(1000).collect();
                    }
                }
            }
            _ => {}
        }
        
        event_loop.dispatch(Duration::from_millis(16), &mut app).unwrap();
        if app.exit { break; }
    }
}

struct App {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    exit: bool,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    buffer: Option<Buffer>,
    window: Window,
    font: Font,
    pty: pty::Pty,
    output_text: String,
    keyboard: Option<wl_keyboard::WlKeyboard>,
}

impl App {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let stride = self.width as i32 * 4;
        
        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool.create_buffer(
                self.width as i32,
                self.height as i32,
                stride,
                wl_shm::Format::Argb8888,
            ).unwrap().0
        });
        
        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                let (second_buffer, canvas) = self.pool.create_buffer(
                    self.width as i32,
                    self.height as i32,
                    stride,
                    wl_shm::Format::Argb8888,
                ).unwrap();
                *buffer = second_buffer;
                canvas
            }
        };
        
        // Background: #0f1411
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = 0x11;
            pixel[1] = 0x14;
            pixel[2] = 0x0f;
            pixel[3] = 0xFF;
        }
        
        let font_size = 16.0;
        let line_height = 20.0;
        let max_lines = (self.height as f32 / line_height) as usize;
        
        let lines: Vec<&str> = self.output_text.lines().collect();
        let start_line = lines.len().saturating_sub(max_lines);
        
        let mut y = 10.0;
        
        for line in lines.iter().skip(start_line) {
            let mut x = 10.0;
            
            for ch in line.chars() {
                if x > self.width as f32 - 20.0 { break; }
                
                let (metrics, bitmap) = self.font.rasterize(ch, font_size);
                
                if metrics.width == 0 || bitmap.is_empty() {
                    x += metrics.advance_width;
                    continue;
                }
                
                for (py, row) in bitmap.chunks(metrics.width).enumerate() {
                    for (px, &alpha) in row.iter().enumerate() {
                        if alpha == 0 { continue; }
                        
                        let screen_x = (x as i32 + px as i32 + metrics.xmin) as usize;
                        let screen_y = (y as i32 + py as i32 + metrics.ymin) as usize;
                        
                        if screen_x >= self.width as usize || screen_y >= self.height as usize {
                            continue;
                        }
                        
                        let idx = (screen_y * self.width as usize + screen_x) * 4;
                        if idx + 3 < canvas.len() {
                            let alpha = alpha as f32 / 255.0;
                            canvas[idx + 0] = (0xDA as f32 * alpha + canvas[idx + 0] as f32 * (1.0 - alpha)) as u8;
                            canvas[idx + 1] = (0xE0 as f32 * alpha + canvas[idx + 1] as f32 * (1.0 - alpha)) as u8;
                            canvas[idx + 2] = (0xD7 as f32 * alpha + canvas[idx + 2] as f32 * (1.0 - alpha)) as u8;
                        }
                    }
                }
                
                x += metrics.advance_width;
            }
            
            y += line_height;
            if y > self.height as f32 - 20.0 { break; }
        }
        
        self.window.wl_surface().damage_buffer(0, 0, self.width as i32, self.height as i32);
        self.window.wl_surface().frame(qh, self.window.wl_surface().clone());
        buffer.attach_to(self.window.wl_surface()).unwrap();
        self.window.commit();
    }
}

impl CompositorHandler for App {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        self.draw(qh);
    }
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl SeatHandler for App {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            self.keyboard = self.seat_state.get_keyboard(qh, &seat, None).ok();
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for App {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[smithay_client_toolkit::seat::keyboard::Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    
    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        if let Some(utf8) = event.utf8 {
            let _ = self.pty.write(utf8.as_bytes());
        }
    }
    
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: Modifiers, _: RawModifiers, _: u32) {}
    
    fn repeat_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
}

impl WindowHandler for App {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.exit = true;
    }
    
    fn configure(&mut self, _: &Connection, qh: &QueueHandle<Self>, _window: &Window, configure: WindowConfigure, _serial: u32) {
        self.buffer = None;
        self.width = configure.new_size.0.map(|v| v.get()).unwrap_or(800);
        self.height = configure.new_size.1.map(|v| v.get()).unwrap_or(600);
        
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm }
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(App);
delegate_output!(App);
delegate_shm!(App);
delegate_xdg_shell!(App);
delegate_xdg_window!(App);
delegate_seat!(App);
delegate_keyboard!(App);
delegate_registry!(App);
