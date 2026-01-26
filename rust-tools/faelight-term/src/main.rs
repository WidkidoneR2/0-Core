mod pty;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use smithay_client_toolkit::{
    reexports::calloop::EventLoop,
    reexports::calloop_wayland_source::WaylandSource,
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry, 
    delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
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
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};
use swash::{
    FontRef,
    scale::{ScaleContext, Render, Source, StrikeWith},
};
use wl_clipboard_rs::copy::{MimeType, Options, Source as ClipSource};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const FONT_DATA: &[u8] = include_bytes!("../fonts/JetBrainsMonoNerdFont-Regular.ttf");
const EMOJI_DATA: &[u8] = include_bytes!("/usr/share/fonts/noto/NotoColorEmoji.ttf");

const COLORS: [[u8; 3]; 16] = [
    [0x0F, 0x14, 0x11], [0xE6, 0x7E, 0x80], [0x6B, 0xE3, 0xA3], [0xF5, 0xC1, 0x77],
    [0x5C, 0xC8, 0xFF], [0xD6, 0x99, 0xB6], [0x7F, 0xC8, 0xC8], [0xD7, 0xE0, 0xDA],
    [0x77, 0x8F, 0x7F], [0xE6, 0x7E, 0x80], [0x6B, 0xE3, 0xA3], [0xF5, 0xC1, 0x77],
    [0x5C, 0xC8, 0xFF], [0xD6, 0x99, 0xB6], [0x7F, 0xC8, 0xC8], [0xFF, 0xFF, 0xFF],
];

#[derive(Clone, Copy, Default)]
struct TextAttrs {
    bold: bool,
    italic: bool,
    underline: bool,
}

#[derive(Clone)]
struct Cell {
    ch: char,
    fg: [u8; 3],
    bg: [u8; 3],
    attrs: TextAttrs,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { 
            ch: ' ', 
            fg: COLORS[7], 
            bg: COLORS[0],
            attrs: TextAttrs::default(),
        }
    }
}


// Helper function to get character width (1 or 2 cells)
fn char_width(ch: char) -> usize {
    // Emoji and wide characters take 2 cells
    match ch {
        '\u{1F300}'..='\u{1F9FF}' | // Emoji ranges
        '\u{2600}'..='\u{26FF}' |   // Misc symbols
        '\u{2700}'..='\u{27BF}' |   // Dingbats
        '\u{FE00}'..='\u{FE0F}' |   // Variation selectors
        '\u{1F600}'..='\u{1F64F}' | // Emoticons
        '\u{1F680}'..='\u{1F6FF}' | // Transport
        '\u{1F900}'..='\u{1F9FF}' | // Supplemental
        '\u{3000}'..='\u{303F}' |   // CJK symbols
        '\u{3040}'..='\u{309F}' |   // Hiragana
        '\u{30A0}'..='\u{30FF}' |   // Katakana
        '\u{4E00}'..='\u{9FFF}' |   // CJK Unified
        '\u{AC00}'..='\u{D7AF}' => 2, // Hangul
        _ => 1
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
    current_fg: [u8; 3],
    current_bg: [u8; 3],
    current_attrs: TextAttrs,
    scroll_offset: usize,
    utf8_buffer: Vec<u8>,  // FIX 2: UTF-8 carry-over buffer
}

impl Terminal {
    fn new(rows: usize, cols: usize) -> Self {
        let grid = vec![vec![Cell::default(); cols]; rows];
        Terminal {
            rows, cols, grid, cursor_row: 0, cursor_col: 0,
            scrollback: Vec::new(), max_scrollback: 10000,
            current_fg: COLORS[7], current_bg: COLORS[0],
            current_attrs: TextAttrs::default(),
            scroll_offset: 0,
            utf8_buffer: Vec::new(),  // FIX 2: Initialize buffer
        }
    }
    
    fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = (self.scroll_offset + lines).min(self.scrollback.len());
    }
    
    fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }
    
    fn get_visible_text(&self, start_row: usize, end_row: usize, start_col: usize, end_col: usize) -> String {
        let mut text = String::new();
        
        for row in start_row..=end_row.min(self.rows - 1) {
            // FIX 7: Simplified - always read from grid when not scrolled
            let actual_row = &self.grid[row];
            
            let col_start = if row == start_row { start_col } else { 0 };
            let col_end = if row == end_row { end_col } else { self.cols - 1 };
            
            for col in col_start..=col_end.min(self.cols - 1) {
                text.push(actual_row[col].ch);
            }
            
            if row < end_row {
                text.push('\n');
            }
        }
        
        text
    }
    
    // FIX 2: Process bytes with UTF-8 carry-over handling
    fn process_bytes(&mut self, bytes: &[u8]) {
        self.utf8_buffer.extend_from_slice(bytes);
        
        let mut valid_end = 0;
        let mut chars_str = String::new();
        
        for i in 0..self.utf8_buffer.len() {
            if let Ok(s) = std::str::from_utf8(&self.utf8_buffer[..=i]) {
                valid_end = i + 1;
                chars_str = s.to_string();
            }
        }
        
        if valid_end > 0 {
            self.process_text(&chars_str);
            self.utf8_buffer.drain(..valid_end);
        }
        
        if self.utf8_buffer.len() > 4 {
            self.utf8_buffer.clear();
        }
    }
    
    fn process_text(&mut self, text: &str) {
        self.scroll_offset = 0;
        
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                if chars.peek() == Some(&'[') {
                    chars.next();
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
                // FIX 1: Backspace only moves cursor, does NOT erase
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            } else if ch == '\x7f' {
                // FIX 1: DEL is ignored (shell handles it)
            } else if ch == '\t' {
                let spaces = 8 - (self.cursor_col % 8);
                for _ in 0..spaces { self.write_char(' '); }
            } else if !ch.is_control() {
                self.write_char(ch);
            }
        }
    }
    
    fn handle_csi_sequence(&mut self, params: &str, cmd: char) {
        match cmd {
            'H' | 'f' => {
                let parts: Vec<&str> = params.split(';').collect();
                let row = parts.get(0).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).saturating_sub(1);
                let col = parts.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).saturating_sub(1);
                self.cursor_row = row.min(self.rows - 1);
                self.cursor_col = col.min(self.cols - 1);
            }
            'A' => self.cursor_row = self.cursor_row.saturating_sub(params.parse().unwrap_or(1)),
            'B' => self.cursor_row = (self.cursor_row + params.parse::<usize>().unwrap_or(1)).min(self.rows - 1),
            'C' => self.cursor_col = (self.cursor_col + params.parse::<usize>().unwrap_or(1)).min(self.cols - 1),
            'D' => self.cursor_col = self.cursor_col.saturating_sub(params.parse().unwrap_or(1)),
            'J' => {
                if params.is_empty() || params == "0" {
                    for col in self.cursor_col..self.cols {
                        self.grid[self.cursor_row][col] = Cell::default();
                    }
                    for row in (self.cursor_row + 1)..self.rows {
                        for col in 0..self.cols {
                            self.grid[row][col] = Cell::default();
                        }
                    }
                } else if params == "2" {
                    self.grid = vec![vec![Cell::default(); self.cols]; self.rows];
                    self.cursor_row = 0;
                    self.cursor_col = 0;
                }
            }
            'K' => {
                if params.is_empty() || params == "0" {
                    for col in self.cursor_col..self.cols {
                        self.grid[self.cursor_row][col] = Cell::default();
                    }
                } else if params == "2" {
                    for col in 0..self.cols {
                        self.grid[self.cursor_row][col] = Cell::default();
                    }
                }
            }
            'm' => {
                if params.is_empty() {
                    self.current_fg = COLORS[7];
                    self.current_bg = COLORS[0];
                    self.current_attrs = TextAttrs::default();
                } else {
                    let codes: Vec<&str> = params.split(';').collect();
                    let mut i = 0;
                    while i < codes.len() {
                        if let Ok(n) = codes[i].parse::<u8>() {
                            match n {
                                0 => {
                                    self.current_fg = COLORS[7];
                                    self.current_bg = COLORS[0];
                                    self.current_attrs = TextAttrs::default();
                                }
                                1 => self.current_attrs.bold = true,
                                3 => self.current_attrs.italic = true,
                                4 => self.current_attrs.underline = true,
                                22 => self.current_attrs.bold = false,
                                23 => self.current_attrs.italic = false,
                                24 => self.current_attrs.underline = false,
                                30..=37 => self.current_fg = COLORS[(n - 30) as usize],
                                38 => {
                                    if i + 4 < codes.len() && codes[i + 1] == "2" {
                                        if let (Ok(r), Ok(g), Ok(b)) = (
                                            codes[i + 2].parse::<u8>(),
                                            codes[i + 3].parse::<u8>(),
                                            codes[i + 4].parse::<u8>(),
                                        ) {
                                            self.current_fg = [r, g, b];
                                            i += 4;
                                        }
                                    }
                                }
                                40..=47 => self.current_bg = COLORS[(n - 40) as usize],
                                48 => {
                                    if i + 4 < codes.len() && codes[i + 1] == "2" {
                                        if let (Ok(r), Ok(g), Ok(b)) = (
                                            codes[i + 2].parse::<u8>(),
                                            codes[i + 3].parse::<u8>(),
                                            codes[i + 4].parse::<u8>(),
                                        ) {
                                            self.current_bg = [r, g, b];
                                            i += 4;
                                        }
                                    }
                                }
                                90..=97 => self.current_fg = COLORS[(n - 90 + 8) as usize],
                                100..=107 => self.current_bg = COLORS[(n - 100 + 8) as usize],
                                _ => {}
                            }
                        }
                        i += 1;
                    }
                }
            }
            _ => {}
        }
    }
    
    fn write_char(&mut self, ch: char) {
        let width = char_width(ch);
        if self.cursor_col + width > self.cols { self.new_line(); }
        if self.cursor_row < self.rows {
            self.grid[self.cursor_row][self.cursor_col] = Cell { 
                ch, 
                fg: self.current_fg, 
                bg: self.current_bg,
                attrs: self.current_attrs,
            };
            self.cursor_col += width;
        }
    }
    
    fn new_line(&mut self) {
        self.cursor_col = 0;
        self.cursor_row += 1;
        if self.cursor_row >= self.rows {
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

// FIX 4: Glyph cache for massive performance improvement
#[derive(Hash, Eq, PartialEq, Clone)]
struct GlyphCacheKey {
    ch: char,
    font_size: u32,
    color: [u8; 3],
}

struct CachedGlyph {
    rgba: Vec<u8>,
    width: usize,
    height: usize,
    left: i32,
    top: i32,
}

struct GlyphCache {
    cache: HashMap<GlyphCacheKey, CachedGlyph>,
}

impl GlyphCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
    
    fn get_or_render(
        &mut self,
        scale_context: &mut ScaleContext,
        main_font: &FontRef,
        emoji_font: Option<&FontRef>,
        ch: char,
        font_size: f32,
        color: [u8; 3],
    ) -> Option<&CachedGlyph> {
        let key = GlyphCacheKey {
            ch,
            font_size: (font_size * 10.0) as u32,
            color,
        };
        
        if !self.cache.contains_key(&key) {
            if let Some((rgba, width, height, left, top)) = 
                render_glyph(scale_context, main_font, emoji_font, ch, font_size, color) {
                self.cache.insert(key.clone(), CachedGlyph { rgba, width, height, left, top });
            } else {
                return None;
            }
        }
        
        self.cache.get(&key)
    }
    
    fn clear(&mut self) {
        self.cache.clear();
    }
}

// Standalone glyph rendering function
fn render_glyph(
    scale_context: &mut ScaleContext,
    main_font: &FontRef,
    emoji_font: Option<&FontRef>,
    ch: char,
    font_size: f32,
    color: [u8; 3],
) -> Option<(Vec<u8>, usize, usize, i32, i32)> {
    let glyph_id = main_font.charmap().map(ch);
    let (font, glyph_id) = if glyph_id != 0 {
        (main_font, glyph_id)
    } else if let Some(emoji_font) = emoji_font {
        let emoji_id = emoji_font.charmap().map(ch);
        if emoji_id == 0 { return None; }
        (emoji_font, emoji_id)
    } else {
        return None;
    };
    
    let mut scaler = scale_context
        .builder(*font)
        .size(font_size)
        .hint(true)
        .build();
    
    let render = Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
    ]);
    
    let image = render.render(&mut scaler, glyph_id)?;
    
    let width = image.placement.width as usize;
    let height = image.placement.height as usize;
    let left = image.placement.left;
    let top = image.placement.top;
    
    let mut rgba = vec![0u8; width * height * 4];
    
    match image.content {
        swash::scale::image::Content::Mask => {
            for (i, &alpha) in image.data.iter().enumerate() {
                let idx = i * 4;
                rgba[idx] = color[2];
                rgba[idx + 1] = color[1];
                rgba[idx + 2] = color[0];
                rgba[idx + 3] = alpha;
            }
        }
        swash::scale::image::Content::Color => {
            rgba.copy_from_slice(&image.data);
        }
        swash::scale::image::Content::SubpixelMask => {
            for i in 0..width * height {
                let idx = i * 4;
                let src_idx = i * 3;
                let alpha = image.data[src_idx];
                rgba[idx] = color[2];
                rgba[idx + 1] = color[1];
                rgba[idx + 2] = color[0];
                rgba[idx + 3] = alpha;
            }
        }
    }
    
    Some((rgba, width, height, left, top))
}

fn health_check() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏥 faelight-term v{} - Health Check", VERSION);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    print!("  Checking Wayland... ");
    match Connection::connect_to_env() {
        Ok(_) => println!("✅"),
        Err(e) => {
            println!("❌ {}", e);
            return Err(e.into());
        }
    }
    
    print!("  Checking main font... ");
    match FontRef::from_index(FONT_DATA, 0) {
        Some(_) => println!("✅"),
        None => {
            println!("❌ Failed to load font");
            return Err("Font load failed".into());
        }
    }
    
    print!("  Checking emoji font... ");
    match FontRef::from_index(EMOJI_DATA, 0) {
        Some(_) => println!("✅"),
        None => println!("⚠️  No emoji font (optional)"),
    }
    
    print!("  Checking PTY support... ");
    match pty::Pty::spawn_shell(24, 80) {
        Ok(_) => println!("✅"),
        Err(e) => {
            println!("❌ {}", e);
            return Err(e.into());
        }
    }
    
    println!("\n✅ All checks passed!");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("faelight-term v{}", VERSION);
                return Ok(());
            }
            "--help" | "-h" => {
                println!("faelight-term v{} - Terminal Emulator for Faelight Forest", VERSION);
                println!();
                println!("FEATURES:");
                println!("  • 24-bit true color support");
                println!("  • Color emoji rendering 🌲🦀🔓");
                println!("  • Bold, italic, underline text");
                println!("  • Copy/paste (Ctrl+Shift+C/V)");
                println!("  • Mouse wheel scrolling");
                println!("  • Font zoom (Ctrl +/-/0)");
                println!("  • 10,000 lines scrollback");
                println!();
                println!("KEYBOARD SHORTCUTS:");
                println!("    Ctrl + Shift + C    Copy selection");
                println!("    Ctrl + +/-/0        Font zoom");
                println!("    Shift + PageUp/Dn   Scroll");
                println!();
                println!("OPTIONS:");
                println!("    --help, --version, --health-check");
                return Ok(());
            }
            "--health-check" => {
                return health_check();
            }
            _ => {
                std::process::exit(1);
            }
        }
    }
    
    println!("🌲 faelight-term v{} starting (with emoji support)...", VERSION);
    
    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();
    
    let mut event_loop: EventLoop<App> = EventLoop::try_new()?;
    WaylandSource::new(conn.clone(), event_queue).insert(event_loop.handle())?;
    
    let compositor = CompositorState::bind(&globals, &qh)?;
    let xdg_shell = XdgShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;
    
    let surface = compositor.create_surface(&qh);
    let window = xdg_shell.create_window(surface, WindowDecorations::ServerDefault, &qh);
    
    window.set_title("🌲 faelight-term");
    window.set_app_id("faelight-term");
    window.commit();
    
    let pool = SlotPool::new(800 * 600 * 4, &shm)?;
    let pty = pty::Pty::spawn_shell(24, 80)?;
    let terminal = Terminal::new(24, 80);
    
    let main_font = FontRef::from_index(FONT_DATA, 0)
        .ok_or("Failed to load main font")?;
    let emoji_font = FontRef::from_index(EMOJI_DATA, 0);
    
    let mut app = App {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm, exit: false, first_configure: true, pool,
        width: 800, height: 600, buffer: None, window, pty, terminal,
        keyboard: None, pointer: None,
        cursor_blink_state: true, last_blink: Instant::now(),
        font_size: 17.0, 
        ctrl_pressed: false,
        shift_pressed: false,
        mouse_pressed: false,
        selection_start: None,
        selection_end: None,
        main_font,
        emoji_font,
        scale_context: ScaleContext::new(),
        glyph_cache: GlyphCache::new(),  // FIX 4: Add glyph cache
    };
    
    loop {
        let mut buf = [0u8; 4096];
        match app.pty.read(&mut buf) {
            Ok(n) if n > 0 => {
                // FIX 2: Use process_bytes for proper UTF-8 handling
                app.terminal.process_bytes(&buf[..n]);
            }
            _ => {}
        }
        
        if app.last_blink.elapsed() > Duration::from_millis(500) {
            app.cursor_blink_state = !app.cursor_blink_state;
            app.last_blink = Instant::now();
        }
        
        event_loop.dispatch(Duration::from_millis(16), &mut app)?;
        if app.exit { break; }
    }
    
    Ok(())
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
    pty: pty::Pty, 
    terminal: Terminal,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pointer: Option<wl_pointer::WlPointer>,
    cursor_blink_state: bool, 
    last_blink: Instant,
    font_size: f32, 
    ctrl_pressed: bool,
    shift_pressed: bool,
    mouse_pressed: bool,
    selection_start: Option<(usize, usize)>,
    selection_end: Option<(usize, usize)>,
    main_font: FontRef<'static>,
    emoji_font: Option<FontRef<'static>>,
    scale_context: ScaleContext,
    glyph_cache: GlyphCache,  // FIX 4: Add glyph cache
}

impl App {
    fn copy_selection(&self) {
        if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
            let (start_row, start_col) = start.min(end);
            let (end_row, end_col) = start.max(end);
            
            let text = self.terminal.get_visible_text(start_row, end_row, start_col, end_col);
            
            if !text.is_empty() {
                let len = text.len();
                // FIX: Use the clipboard correctly - pass string data
                let opts = Options::new();
                match opts.copy(
                    ClipSource::Bytes(text.as_bytes().to_vec().into()),
                    MimeType::Text
                ) {
                    Ok(_) => println!("📋 Copied {} chars", len),
                    Err(e) => eprintln!("⚠️  Copy failed: {}", e),
                }
            }
        }
    }
    
    fn paste_from_clipboard(&mut self) {
        use wl_clipboard_rs::paste::{get_contents, ClipboardType, Seat, MimeType};
        use std::io::Read;
        
        // FIX: Read from PipeReader
        match get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text) {
            Ok((mut pipe, _mime)) => {
                let mut buffer = String::new();
                if let Ok(_) = pipe.read_to_string(&mut buffer) {
                    let _ = self.pty.write(buffer.as_bytes());
                    println!("📋 Pasted {} bytes", buffer.len());
                }
            }
            Err(e) => eprintln!("⚠️  Paste failed: {}", e),
        }
    }
    
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let stride = self.width as i32 * 4;
        
        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool.create_buffer(self.width as i32, self.height as i32, stride, wl_shm::Format::Argb8888).unwrap().0
        });
        
        let canvas = match self.pool.canvas(buffer) {
            Some(canvas) => canvas,
            None => {
                let (second_buffer, canvas) = self.pool.create_buffer(
                    self.width as i32, self.height as i32, stride, wl_shm::Format::Argb8888).unwrap();
                *buffer = second_buffer;
                canvas
            }
        };
        
        // Background
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = 0x11; pixel[1] = 0x14; pixel[2] = 0x0f; pixel[3] = 0xFF;
        }
        
        let char_width = self.font_size * 0.6;
        let line_height = self.font_size * 1.45;
        
        // Determine which rows to render based on scroll offset
        let rows_to_render: Vec<&Vec<Cell>> = if self.terminal.scroll_offset > 0 {
            // Render from scrollback
            let start = self.terminal.scrollback.len().saturating_sub(self.terminal.scroll_offset);
            self.terminal.scrollback[start..].iter()
                .chain(self.terminal.grid.iter())
                .take(self.terminal.rows)
                .collect()
        } else {
            // Normal rendering
            self.terminal.grid.iter().collect()
        };
        
        for (row_idx, row) in rows_to_render.iter().enumerate() {
            let y = 15.0 + row_idx as f32 * line_height;
            
            for (col_idx, cell) in row.iter().enumerate() {
                let x = 15.0 + col_idx as f32 * char_width;
                
                let is_selected = if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                    let (min_pos, max_pos) = if start <= end { (start, end) } else { (end, start) };
                    (row_idx, col_idx) >= min_pos && (row_idx, col_idx) <= max_pos
                } else {
                    false
                };
                
                let bg = if is_selected {
                    [0x6b, 0xe3, 0xa3]
                } else {
                    cell.bg
                };
                
                // Background
                if bg != COLORS[0] {
                    for dy in 0..(line_height as usize) {
                        for dx in 0..(char_width as usize) {
                            let screen_x = x as usize + dx;
                            let screen_y = y as usize + dy;
                            if screen_x < self.width as usize && screen_y < self.height as usize {
                                let idx = (screen_y * self.width as usize + screen_x) * 4;
                                if idx + 3 < canvas.len() {
                                    canvas[idx] = bg[2];
                                    canvas[idx + 1] = bg[1];
                                    canvas[idx + 2] = bg[0];
                                }
                            }
                        }
                    }
                }
                
                // Cursor
                if row_idx == self.terminal.cursor_row && col_idx == self.terminal.cursor_col 
                    && self.cursor_blink_state && self.terminal.scroll_offset == 0 {
                    for dy in 0..(line_height as usize) {
                        for dx in 0..(char_width as usize) {
                            let screen_x = x as usize + dx;
                            let screen_y = y as usize + dy;
                            if screen_x < self.width as usize && screen_y < self.height as usize {
                                let idx = (screen_y * self.width as usize + screen_x) * 4;
                                if idx + 3 < canvas.len() {
                                    canvas[idx] = 0xA3; canvas[idx + 1] = 0xE3; canvas[idx + 2] = 0x6B;
                                }
                            }
                        }
                    }
                }
                
                // Render character - FIX 4: Use glyph cache
                if cell.ch != ' ' {
                    if let Some(glyph) = self.glyph_cache.get_or_render(
                        &mut self.scale_context,
                        &self.main_font,
                        self.emoji_font.as_ref(),
                        cell.ch,
                        self.font_size,
                        cell.fg
                    ) {
                        let glyph_x = x + glyph.left as f32;
                        let baseline_y = y + line_height * 0.8;
                        let glyph_y = baseline_y - glyph.top as f32;
                        
                        for py in 0..glyph.height {
                            for px in 0..glyph.width {
                                let src_idx = (py * glyph.width + px) * 4;
                                let alpha = glyph.rgba[src_idx + 3];
                                if alpha == 0 { continue; }
                                
                                let screen_x = (glyph_x + px as f32).round() as usize;
                                let screen_y = (glyph_y + py as f32).round() as usize;
                                
                                if screen_x >= self.width as usize || screen_y >= self.height as usize {
                                    continue;
                                }
                                
                                let idx = (screen_y * self.width as usize + screen_x) * 4;
                                if idx + 3 < canvas.len() {
                                    let alpha_f = alpha as f32 / 255.0;
                                    canvas[idx] = (glyph.rgba[src_idx] as f32 * alpha_f + canvas[idx] as f32 * (1.0 - alpha_f)) as u8;
                                    canvas[idx + 1] = (glyph.rgba[src_idx + 1] as f32 * alpha_f + canvas[idx + 1] as f32 * (1.0 - alpha_f)) as u8;
                                    canvas[idx + 2] = (glyph.rgba[src_idx + 2] as f32 * alpha_f + canvas[idx + 2] as f32 * (1.0 - alpha_f)) as u8;
                                }
                            }
                        }
                        
                        // Underline
                        if cell.attrs.underline {
                            let underline_y = (baseline_y + 2.0) as usize;
                            for ux in (x as usize)..(x as usize + char_width as usize) {
                                if ux < self.width as usize && underline_y < self.height as usize {
                                    let idx = (underline_y * self.width as usize + ux) * 4;
                                    if idx + 3 < canvas.len() {
                                        canvas[idx] = cell.fg[2];
                                        canvas[idx + 1] = cell.fg[1];
                                        canvas[idx + 2] = cell.fg[0];
                                    }
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
    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) { self.draw(qh); }
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
        if capability == Capability::Pointer && self.pointer.is_none() {
            self.pointer = self.seat_state.get_pointer(qh, &seat).ok();
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for App {
    fn pointer_frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_pointer::WlPointer, events: &[PointerEvent]) {
        for event in events {
            match event.kind {
                PointerEventKind::Axis { vertical, .. } => {
                    // Use continuous scroll (absolute) since discrete is 0 on some systems
                    if vertical.absolute < 0.0 {
                        self.terminal.scroll_up(3);
                    } else if vertical.absolute > 0.0 {
                        self.terminal.scroll_down(3);
                    }
                }
                PointerEventKind::Press { button, .. } => {
                    if button == 272 {
                        self.mouse_pressed = true;  // FIX 3: Set mouse_pressed
                        let char_width = self.font_size * 0.6;
                        let line_height = self.font_size * 1.45;
                        let col = ((event.position.0 - 15.0) / char_width as f64) as usize;
                        let row = ((event.position.1 - 15.0) / line_height as f64) as usize;
                        if row < self.terminal.rows && col < self.terminal.cols {
                            self.selection_start = Some((row, col));
                            self.selection_end = Some((row, col));
                        }
                    }
                }
                PointerEventKind::Release { button, .. } => {
                    if button == 272 { self.mouse_pressed = false; }
                }
                PointerEventKind::Motion { .. } => {
                    if self.mouse_pressed && self.selection_start.is_some() {
                        let char_width = self.font_size * 0.6;
                        let line_height = self.font_size * 1.45;
                        let col = ((event.position.0 - 15.0) / char_width as f64) as usize;
                        let row = ((event.position.1 - 15.0) / line_height as f64) as usize;
                        if row < self.terminal.rows && col < self.terminal.cols {
                            self.selection_end = Some((row, col));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl KeyboardHandler for App {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[smithay_client_toolkit::seat::keyboard::Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    
    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        // FIX 6: Handle both uppercase and lowercase for copy/paste
        if self.ctrl_pressed && self.shift_pressed && (event.keysym == Keysym::c || event.keysym == Keysym::C) {
            self.copy_selection();
            return;
        }
        
        if self.ctrl_pressed && self.shift_pressed && (event.keysym == Keysym::v || event.keysym == Keysym::V) {
            self.paste_from_clipboard();
            return;
        }
        
        // Font size controls
        if self.ctrl_pressed {
            match event.keysym {
                Keysym::plus | Keysym::equal => { 
                    self.font_size = (self.font_size + 1.0).min(30.0);
                    self.glyph_cache.clear();
                    println!("🔍 Font size: {}", self.font_size); 
                    return; 
                }
                Keysym::minus | Keysym::underscore => { 
                    self.font_size = (self.font_size - 1.0).max(6.0);
                    self.glyph_cache.clear();
                    println!("🔍 Font size: {}", self.font_size); 
                    return; 
                }
                Keysym::_0 => { 
                    self.font_size = 17.0;
                    self.glyph_cache.clear();
                    println!("🔍 Reset"); 
                    return; 
                }
                _ => {}
            }
        }
        
        // Scrolling
        if self.shift_pressed {
            match event.keysym {
                Keysym::Page_Up => {
                    self.terminal.scroll_up(10);
                    return;
                }
                Keysym::Page_Down => {
                    self.terminal.scroll_down(10);
                    return;
                }
                _ => {}
            }
        }
        
        // FIX 5: Handle Ctrl+key combinations for terminal control
        if self.ctrl_pressed && !self.shift_pressed {
            let ctrl_char = match event.keysym {
                Keysym::c => Some('\x03'),  // Ctrl+C
                Keysym::d => Some('\x04'),  // Ctrl+D
                Keysym::z => Some('\x1a'),  // Ctrl+Z
                Keysym::l => Some('\x0c'),  // Ctrl+L
                Keysym::a => Some('\x01'),  // Ctrl+A
                Keysym::e => Some('\x05'),  // Ctrl+E
                Keysym::k => Some('\x0b'),  // Ctrl+K
                Keysym::u => Some('\x15'),  // Ctrl+U
                Keysym::w => Some('\x17'),  // Ctrl+W
                _ => None,
            };
            
            if let Some(ch) = ctrl_char {
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                let _ = self.pty.write(s.as_bytes());
                return;
            }
        }
        
        // Normal key input
        if let Some(utf8) = event.utf8 { 
            let _ = self.pty.write(utf8.as_bytes()); 
        }
    }
    
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, modifiers: Modifiers, _: RawModifiers, _: u32) {
        self.ctrl_pressed = modifiers.ctrl;
        self.shift_pressed = modifiers.shift;
    }
    
    fn repeat_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
}

impl WindowHandler for App {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) { self.exit = true; }
    fn configure(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &Window, configure: WindowConfigure, _: u32) {
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
delegate_pointer!(App);
delegate_registry!(App);
