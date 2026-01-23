mod pty;
use std::time::Duration;
use smithay_client_toolkit::{
    reexports::calloop::EventLoop,
    reexports::calloop_wayland_source::WaylandSource,
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_registry, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
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
    protocol::{wl_output, wl_shm, wl_surface},
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
    println!("🦀 faelight-term v1.0");
    
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
    
    println!("✅ Shell spawned!");
    
    let mut app = App {
        registry_state: RegistryState::new(&globals),
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
    };
    
    loop {
        // Read from PTY
        let mut buf = [0u8; 4096];
        if let Ok(n) = app.pty.read(&mut buf) {
            if n > 0 {
                if let Ok(text) = std::str::from_utf8(&buf[..n]) {
                    let clean = strip_ansi(text);
                    app.output_text.push_str(&clean);
                }
            }
        }
        
        event_loop.dispatch(Duration::from_millis(16), &mut app).unwrap();
        if app.exit { break; }
    }
}

struct App {
    registry_state: RegistryState,
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
        
        // Background: dark green
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = 0x11; // B
            pixel[1] = 0x14; // G
            pixel[2] = 0x0f; // R
            pixel[3] = 0xFF; // A
        }
        
        // Draw text (last 1000 chars)
        let text = self.output_text.chars().rev().take(1000).collect::<String>();
        let text: String = text.chars().rev().collect();
        
        let font_size = 16.0;
        let mut x = 10.0;
        let mut y = 20.0;
        
        for ch in text.chars() {
            if ch == '\n' {
                y += font_size + 4.0;
                x = 10.0;
                continue;
            }
            
            if ch == '\r' {
                continue; // Skip carriage return
            }
            
            let (metrics, bitmap) = self.font.rasterize(ch, font_size);
            
            // Skip zero-width characters
            if metrics.width == 0 || bitmap.is_empty() {
                x += metrics.advance_width;
                continue;
            }
            
            for (py, row) in bitmap.chunks(metrics.width).enumerate() {
                for (px, &alpha) in row.iter().enumerate() {
                    if alpha == 0 { continue; }
                    
                    let screen_x = (x as usize + px).saturating_sub(metrics.xmin as usize);
                    let screen_y = (y as usize + py).saturating_sub(metrics.ymin as usize);
                    
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
    registry_handlers![OutputState];
}

delegate_compositor!(App);
delegate_output!(App);
delegate_shm!(App);
delegate_xdg_shell!(App);
delegate_xdg_window!(App);
delegate_registry!(App);
