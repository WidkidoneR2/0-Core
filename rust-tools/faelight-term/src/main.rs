use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_registry, delegate_seat,
    delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{keyboard::KeyboardHandler, Capability, SeatHandler, SeatState},
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
    protocol::{wl_keyboard, wl_output, wl_seat, wl_surface},
    Connection, QueueHandle,
};
use fontdue::Font;

const FONT_DATA: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Starting faelight-term...");
    
    let font = Font::from_bytes(FONT_DATA, fontdue::FontSettings::default())?;
    println!("✅ Font loaded: JetBrains Mono");
    
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let mut app = App {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        compositor_state: CompositorState::bind(&globals, &qh)?,
        shm_state: Shm::bind(&globals, &qh)?,
        xdg_shell_state: XdgShell::bind(&globals, &qh)?,
        
        exit: false,
        window: None,
        keyboard: None,
        pool: None,
        font,
    };

    let surface = app.compositor_state.create_surface(&qh);
    let window = app.xdg_shell_state.create_window(
        surface,
        WindowDecorations::ServerDefault,
        &qh,
    );
    window.set_title("faelight-term");
    window.set_app_id("faelight-term");
    window.commit();
    
    app.window = Some(window);

    println!("✅ Window created! Press ESC to exit");

    while !app.exit {
        event_queue.blocking_dispatch(&mut app)?;
    }

    println!("👋 Goodbye!");
    Ok(())
}

struct App {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm_state: Shm,
    xdg_shell_state: XdgShell,
    
    exit: bool,
    window: Option<Window>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pool: Option<SlotPool>,
    font: Font,
}

impl CompositorHandler for App {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
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

    fn configure(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, window: &Window, configure: WindowConfigure, _serial: u32) {
        let width = configure.new_size.0.map(|w| w.get()).unwrap_or(800);
        let height = configure.new_size.1.map(|h| h.get()).unwrap_or(600);
        
        if self.pool.is_none() {
            self.pool = Some(SlotPool::new((width * height * 4) as usize, &self.shm_state).unwrap());
        }
        
        if let Some(pool) = &mut self.pool {
            let stride = width as i32 * 4;
            
            let (buffer, canvas) = pool.create_buffer(
                width as i32,
                height as i32,
                stride,
                wayland_client::protocol::wl_shm::Format::Argb8888,
            ).unwrap();
            
            // Fill with black
            for pixel in canvas.chunks_exact_mut(4) {
                pixel[0] = 0x00; // B
                pixel[1] = 0x00; // G
                pixel[2] = 0x00; // R
                pixel[3] = 0xFF; // A
            }
            
            // Render text - pass font explicitly to avoid borrow issues
            render_text(&self.font, canvas, width as usize, height as usize);
            
            window.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
            window.wl_surface().commit();
        }
    }
}

// Free function to avoid borrow checker issues
fn render_text(font: &Font, canvas: &mut [u8], width: usize, height: usize) {
    let text = "Hello, faelight-term!";
    let font_size = 24.0;
    
    let mut x = 20.0;
    let y = 40.0;
    
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        
        // Skip zero-width characters (spaces, commas, etc)
        if metrics.width == 0 {
            x += metrics.advance_width;
            continue;
        }
        
        // Draw each pixel of the glyph
        for (glyph_y, row) in bitmap.chunks(metrics.width).enumerate() {
            for (glyph_x, &coverage) in row.iter().enumerate() {
                if coverage > 0 {
                    let px = (x + glyph_x as f32) as usize;
                    let py = (y + glyph_y as f32) as usize;
                    
                    if px < width && py < height {
                        let offset = (py * width + px) * 4;
                        // White text (blend with coverage)
                        canvas[offset + 0] = coverage; // B
                        canvas[offset + 1] = coverage; // G
                        canvas[offset + 2] = coverage; // R
                        canvas[offset + 3] = 0xFF;     // A
                    }
                }
            }
        }
        
        x += metrics.advance_width;
    }
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
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _keysyms: &[smithay_client_toolkit::seat::keyboard::Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: smithay_client_toolkit::seat::keyboard::KeyEvent) {
        if event.raw_code == 1 {
            self.exit = true;
        }
    }
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: smithay_client_toolkit::seat::keyboard::KeyEvent) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _serial: u32, _modifiers: smithay_client_toolkit::seat::keyboard::Modifiers) {}
}

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm_state }
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(App);
delegate_output!(App);
delegate_shm!(App);
delegate_seat!(App);
delegate_keyboard!(App);
delegate_xdg_shell!(App);
delegate_xdg_window!(App);
delegate_registry!(App);
