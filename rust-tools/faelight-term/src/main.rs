mod pty;
use std::time::{Duration, Instant};
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

#[derive(Clone)]
struct Cell {
    ch: char,
    fg: [u8; 3],
    bg: [u8; 3],
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            ch: ' ',
            fg: [0xD7, 0xE0, 0xDA], // Fog White
            bg: [0x0F, 0x14, 0x11], // Forest Night
        }
    }
}

struct Terminal {
    rows: usize,
    cols: usize,
    grid: Vec<Vec<Cell>>,
    cursor_row: usize,
    cursor_col: usize,
    scrollback: Vec<Vec<Cell>>,
    max_scrollback: usize,
}

impl Terminal {
    fn new(rows: usize, cols: usize) -> Self {
        let grid = vec![vec![Cell::default(); cols]; rows];
        Terminal {
            rows,
            cols,
            grid,
            cursor_row: 0,
            cursor_col: 0,
            scrollback: Vec::new(),
            max_scrollback: 1000,
        }
    }

    fn process_text(&mut self, text: &str) {
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // ANSI escape sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    let mut seq = String::new();
                    
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphabetic() {
                            let cmd = chars.next().unwrap();
                            self.handle_csi_sequence(&seq, cmd);
                            break;
                        } else {
                            seq.push(chars.next().unwrap());
                        }
                    }
                }
            } else if ch == '\r' {
                self.cursor_col = 0;
            } else if ch == '\n' {
                self.new_line();
            } else if ch == '\x08' {
                // Backspace
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            } else if ch == '\t' {
                // Tab (8 spaces)
                let spaces = 8 - (self.cursor_col % 8);
                for _ in 0..spaces {
                    self.write_char(' ');
                }
            } else {
                self.write_char(ch);
            }
        }
    }

    fn handle_csi_sequence(&mut self, params: &str, cmd: char) {
        match cmd {
            'H' | 'f' => {
                // Cursor position
                let parts: Vec<&str> = params.split(';').collect();
                let row = parts.get(0).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).saturating_sub(1);
                let col = parts.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).saturating_sub(1);
                self.cursor_row = row.min(self.rows - 1);
                self.cursor_col = col.min(self.cols - 1);
            }
            'A' => {
                // Cursor up
                let n = params.parse::<usize>().unwrap_or(1);
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            'B' => {
                // Cursor down
                let n = params.parse::<usize>().unwrap_or(1);
                self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
            }
            'C' => {
                // Cursor forward
                let n = params.parse::<usize>().unwrap_or(1);
                self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
            }
            'D' => {
                // Cursor back
                let n = params.parse::<usize>().unwrap_or(1);
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            'J' => {
                // Clear screen
                if params.is_empty() || params == "0" {
                    // Clear from cursor to end
                    for col in self.cursor_col..self.cols {
                        self.grid[self.cursor_row][col] = Cell::default();
                    }
                    for row in (self.cursor_row + 1)..self.rows {
                        for col in 0..self.cols {
                            self.grid[row][col] = Cell::default();
                        }
                    }
                } else if params == "2" {
                    // Clear entire screen
                    self.grid = vec![vec![Cell::default(); self.cols]; self.rows];
                    self.cursor_row = 0;
                    self.cursor_col = 0;
                }
            }
            'K' => {
                // Clear line
                if params.is_empty() || params == "0" {
                    // Clear from cursor to end of line
                    for col in self.cursor_col..self.cols {
                        self.grid[self.cursor_row][col] = Cell::default();
                    }
                } else if params == "2" {
                    // Clear entire line
                    for col in 0..self.cols {
                        self.grid[self.cursor_row][col] = Cell::default();
                    }
                }
            }
            _ => {}
        }
    }

    fn write_char(&mut self, ch: char) {
        if self.cursor_col >= self.cols {
            self.new_line();
        }
        
        if self.cursor_row < self.rows {
            self.grid[self.cursor_row][self.cursor_col].ch = ch;
            self.cursor_col += 1;
        }
    }

    fn new_line(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;
        
        if self.cursor_row >= self.rows {
            // Scroll up
            let old_line = self.grid.remove(0);
            self.scrollback.push(old_line);
            if self.scrollback.len() > self.max_scrollback {
                self.scrollback.remove(0);
            }
            self.grid.push(vec![Cell::default(); self.cols]);
            self.cursor_row = self.rows - 1;
        }
    }
}

fn main() {
    println!("🦀 faelight-term v8.0.0 - MIND BLOWING!");
    
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
    
    window.set_title("🌲 faelight-term");
    window.set_app_id("faelight-term");
    window.commit();
    
    let pool = SlotPool::new(800 * 600 * 4, &shm).unwrap();
    let font = Font::from_bytes(FONT_DATA, fontdue::FontSettings::default()).unwrap();
    let pty = pty::Pty::spawn_shell(24, 80).unwrap();
    let terminal = Terminal::new(24, 80);
    
    println!("✅ Terminal ready! Type away!");
    
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
        terminal,
        keyboard: None,
        cursor_blink_state: true,
        last_blink: Instant::now(),
    };
    
    loop {
        let mut buf = [0u8; 4096];
        match app.pty.read(&mut buf) {
            Ok(n) if n > 0 => {
                if let Ok(text) = std::str::from_utf8(&buf[..n]) {
                    app.terminal.process_text(text);
                }
            }
            _ => {}
        }
        
        // Cursor blink (500ms)
        if app.last_blink.elapsed() > Duration::from_millis(500) {
            app.cursor_blink_state = !app.cursor_blink_state;
            app.last_blink = Instant::now();
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
    terminal: Terminal,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    cursor_blink_state: bool,
    last_blink: Instant,
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
        
        // Background: Forest Night
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = 0x11;
            pixel[1] = 0x14;
            pixel[2] = 0x0f;
            pixel[3] = 0xFF;
        }
        
        let font_size = 16.0;
        let char_width = 10.0;  // Monospace width
        let line_height = 20.0;
        
        // Render terminal grid
        for (row_idx, row) in self.terminal.grid.iter().enumerate() {
            let y = 10.0 + row_idx as f32 * line_height;
            
            for (col_idx, cell) in row.iter().enumerate() {
                let x = 10.0 + col_idx as f32 * char_width;
                
                // Render cursor
                if row_idx == self.terminal.cursor_row 
                    && col_idx == self.terminal.cursor_col 
                    && self.cursor_blink_state 
                {
                    // Draw cursor block
                    let cursor_x = x as usize;
                    let cursor_y = y as usize;
                    
                    for dy in 0..line_height as usize {
                        for dx in 0..char_width as usize {
                            let screen_x = cursor_x + dx;
                            let screen_y = cursor_y + dy;
                            
                            if screen_x < self.width as usize && screen_y < self.height as usize {
                                let idx = (screen_y * self.width as usize + screen_x) * 4;
                                if idx + 3 < canvas.len() {
                                    // Faelight Green cursor
                                    canvas[idx + 0] = 0xA3;
                                    canvas[idx + 1] = 0xE3;
                                    canvas[idx + 2] = 0x6B;
                                    canvas[idx + 3] = 0xFF;
                                }
                            }
                        }
                    }
                }
                
                // Render character
                if cell.ch != ' ' {
                    let (metrics, bitmap) = self.font.rasterize(cell.ch, font_size);
                    
                    if metrics.width > 0 && !bitmap.is_empty() {
                        for (py, row_data) in bitmap.chunks(metrics.width).enumerate() {
                            for (px, &alpha) in row_data.iter().enumerate() {
                                if alpha == 0 { continue; }
                                
                                let screen_x = (x as i32 + px as i32 + metrics.xmin) as usize;
                                let screen_y = (y as i32 + py as i32 + metrics.ymin) as usize;
                                
                                if screen_x >= self.width as usize || screen_y >= self.height as usize {
                                    continue;
                                }
                                
                                let idx = (screen_y * self.width as usize + screen_x) * 4;
                                if idx + 3 < canvas.len() {
                                    let alpha = alpha as f32 / 255.0;
                                    canvas[idx + 0] = (cell.fg[2] as f32 * alpha + canvas[idx + 0] as f32 * (1.0 - alpha)) as u8;
                                    canvas[idx + 1] = (cell.fg[1] as f32 * alpha + canvas[idx + 1] as f32 * (1.0 - alpha)) as u8;
                                    canvas[idx + 2] = (cell.fg[0] as f32 * alpha + canvas[idx + 2] as f32 * (1.0 - alpha)) as u8;
                                }
                            }
                        }
                    }
                }
            }
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
