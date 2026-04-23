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
    // Session memory -- restore last working directory
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let db_path = format!("{}/0-core/runtime/state.db", home);
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let last_dir: Option<String> = conn.query_row(
                "SELECT value FROM session_state WHERE key = 'last_dir'",
                [], |r| r.get(0)
            ).ok();
            if let Some(dir) = last_dir {
                if std::path::Path::new(&dir).exists() {
                    let cmd = format!("cd {}\n", dir);
                    pty.write(cmd.as_bytes()).ok();
                }
            }
        }
    }
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
        cell_w:     9u32, // JetBrains Mono 14pt = 9px confirmed
        cell_h:     LINE_HEIGHT as u32,
        configured: false,
        running:    true,
        keyboard:   None,
        pointer:    None,
        modifiers:  Modifiers::default(),
        sel_start:  None,
        sel_end:    None,
        show_status:   false,
        show_friday:   false,
        scroll_offset: 0,
        mouse_down:    false,
        mouse_pos:     (0.0, 0.0),
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
                Ok(n) => {
                    let data = &buf[..n];
                    // Build error highlighting -- inject ANSI around error[E codes
                    if let Ok(s) = std::str::from_utf8(data) {
                        // JSON auto pretty-print
                        let s_trim = s.trim();
                        let json_handled = if (s_trim.starts_with('{') || s_trim.starts_with('[')) && s_trim.len() > 10 {
                            if let Some(pretty) = pretty_json(s_trim) {
                                app.terminal.feed(pretty.as_bytes());
                                dirty = true;
                                true
                            } else { false }
                        } else { false };
                        if !json_handled {
                        if s.contains("error[E") || s.contains("error: ") {
                            // Replace error[EXXXX with red highlight
                            let highlighted = s
                                .replace("error[E", "\x1b[1;31merror\x1b[0m[E")
                                .replace("error: aborting", "\x1b[1;31merror\x1b[0m: aborting")
                                .replace("error: could not", "\x1b[1;31merror\x1b[0m: could not");
                            app.terminal.feed(highlighted.as_bytes());
                        } else {
                            app.terminal.feed(data);
                        }
                    } // end if !json_handled
                    } else {
                        app.terminal.feed(data);
                    }
                    dirty = true;
                }
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
    modifiers:      Modifiers,
    sel_start:      Option<(usize, usize)>,
    sel_end:        Option<(usize, usize)>,
    show_status:    bool,
    show_friday:    bool,
    scroll_offset:  usize,
    mouse_down:     bool,
    mouse_pos:      (f64, f64),
}

fn pretty_json(s: &str) -> Option<String> {
    let t = s.trim();
    if !(t.starts_with('{') || t.starts_with('[')) || t.len() < 10 { return None; }
    let mut out = String::new();
    let mut depth: usize = 0;
    let mut in_str = false;
    let mut prev = ' ';
    for ch in t.chars() {
        if in_str {
            if ch == '"' && prev != '\\' { in_str = false; out.push_str("\x1b[0m"); }
            out.push(ch);
        } else {
            match ch {
                '{' | '[' => { depth+=1; out.push(ch); out.push_str("\r\n"); out.push_str(&"  ".repeat(depth)); }
                '}' | ']' => { if depth>0{depth-=1;} out.push_str("\r\n"); out.push_str(&"  ".repeat(depth)); out.push(ch); }
                ',' => { out.push(ch); out.push_str("\r\n"); out.push_str(&"  ".repeat(depth)); }
                ':' => { out.push_str("\x1b[36m:\x1b[0m "); }
                '"' => { in_str=true; out.push_str("\x1b[32m\""); }
                ' ' => {}
                _ => { out.push(ch); }
            }
        }
        prev = ch;
    }
    out.push_str("\r\n");
    Some(out)
}

impl App {
    fn build_friday_data(&self) -> Vec<(String, String, f64)> {
        // Returns vec of (domain, fact, confidence)
        let home = std::env::var("HOME").unwrap_or_default();
        let db_path = format!("{}/0-core/runtime/state.db", home);
        let mut entries: Vec<(String, String, f64)> = Vec::new();
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            // Recent knowledge entries excluding abstraction noise
            let mut stmt = conn.prepare(
                "SELECT domain, fact, confidence FROM friday_knowledge WHERE domain NOT IN ('abstraction','cross_intent') ORDER BY CASE domain WHEN 'rust' THEN 1 WHEN 'wayland' THEN 2 WHEN 'shell' THEN 3 WHEN 'workflow' THEN 4 WHEN 'philosophy' THEN 5 ELSE 6 END, confidence DESC LIMIT 6"
            ).unwrap_or_else(|_| conn.prepare("SELECT 1,1,1").unwrap());
            let rows = stmt.query_map([], |r| {
                Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,f64>(2)?))
            });
            if let Ok(rows) = rows {
                for row in rows.flatten() { entries.push(row); }
            }
        }
        entries
    }

    fn build_status_text(&self) -> String {
        let home = std::env::var("HOME").unwrap_or_default();
        let db_path = format!("{}/0-core/runtime/state.db", home);
        let lock = if std::path::Path::new("/etc/0-core/.locked").exists() { "LOCKED" } else { "UNLOCKED" };
        let health_cache = format!("{}/.cache/faelight/last-health", home);
        let health = std::fs::read_to_string(&health_cache).unwrap_or_else(|_| "100".to_string());
        let health = health.trim().to_string();
        let (intent, obs) = if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            let intent: String = conn.query_row(
                "SELECT value FROM forest_state WHERE key = 'active_intent' LIMIT 1",
                [], |r| r.get(0)).unwrap_or_else(|_| "none".to_string());
            let facts: i64 = conn.query_row(
                "SELECT COUNT(*) FROM friday_knowledge", [], |r| r.get(0)).unwrap_or(0);
            (intent, facts)
        } else { ("none".to_string(), 0) };
        let intent_str = if intent == "none" || intent.is_empty() {
            "No active intent".to_string()
        } else { format!("INT-{}", intent) };
        // Format: text|r,g,b sections
        format!("{}|107,227,163|  Health: {}%|245,193,119|  {}|92,200,255|  Friday: {} observations|180,140,220",
            lock, health, intent_str, obs)
    }

    fn get_selection_text(&self) -> Option<String> {
        let (sr, sc) = self.sel_start?;
        let (er, ec) = self.sel_end?;
        let (r0, c0, r1, c1) = if (sr, sc) <= (er, ec) {
            (sr, sc, er, ec)
        } else {
            (er, ec, sr, sc)
        };
        let mut text = String::new();
        for row in r0..=r1 {
            if row >= self.terminal.rows { break; }
            let col_start = if row == r0 { c0 } else { 0 };
            let col_end   = if row == r1 { c1 } else { self.terminal.cols };
            for col in col_start..col_end.min(self.terminal.cols) {
                let ch = self.terminal.grid[row][col].ch;
                if ch != ' ' && ch != ' ' || col + 1 < col_end { text.push(if ch == ' ' { ' ' } else { ch }); }
            }
            if row < r1 { text.push('\n'); }
        }
        if text.trim().is_empty() { None } else { Some(text) }
    }
    fn render(&mut self) {
        let width  = self.width;
        let height = self.height;
        let stride = width * 4;
        let status_str_cache = if self.show_status { self.build_status_text() } else { String::new() };
        let friday_data_cache = if self.show_friday { self.build_friday_data() } else { Vec::new() };
        if let Ok((buffer, canvas)) = self.pool.create_buffer(
            width as i32, height as i32, stride as i32,
            wl_shm::Format::Xrgb8888,
        ) {
            // Fill background
            for pixel in canvas.chunks_exact_mut(4) {
                pixel[0] = 0x11; pixel[1] = 0x14;
                pixel[2] = 0x0f; pixel[3] = 0xff;
            }
            // Draw terminal cells -- per-cell rendering (correct cell alignment)
            let rows = self.terminal.rows;
            let cols = self.terminal.cols;
            let cell_w = self.cell_w;
            let cell_h = self.cell_h;
            let sb_len = self.terminal.scrollback.len();
            let scroll_off = self.scroll_offset.min(sb_len);
            for row in 0..rows {
                for col in 0..cols {
                    let cell = if scroll_off > 0 {
                        let sb_row = sb_len.saturating_sub(scroll_off) + row;
                        if sb_row < sb_len {
                            self.terminal.scrollback.get(sb_row)
                                .and_then(|r| r.get(col).copied())
                                .unwrap_or_default()
                        } else {
                            let grid_row = sb_row - sb_len;
                            if grid_row < self.terminal.rows { self.terminal.grid[grid_row][col] }
                            else { continue; }
                        }
                    } else {
                        self.terminal.grid[row][col]
                    };
                    if cell.ch == ' ' || cell.ch == '\0' { continue; }
                    let cell_x = (col as u32 * cell_w) as i32;
                    let cell_y = (row as u32 * cell_h) as i32;
                    if cell_x + cell_w as i32 > width as i32 { continue; }
                    if cell_y + cell_h as i32 > height as i32 { continue; }
                    let cp = cell.ch as u32;
                    let is_emoji = cp != 0x276F && matches!(cp,
                        0x1F300..=0x1FAFF | 0x1F000..=0x1FFFF |
                        0x2300..=0x23FF | 0x2700..=0x27BF | 0x2600..=0x26FF
                    );
                    let base_family = if is_emoji {
                        cosmic_text::Family::Name("Noto Color Emoji")
                    } else {
                        cosmic_text::Family::Name("JetBrainsMono Nerd Font Mono")
                    };
                    let weight = if cell.attrs.bold { cosmic_text::Weight::BOLD } else { cosmic_text::Weight::NORMAL };
                    let style  = if cell.attrs.italic { cosmic_text::Style::Italic } else { cosmic_text::Style::Normal };
                    let mut fr = cell.fg.r; let mut fg_c = cell.fg.g; let mut fb = cell.fg.b;
                    if cell.attrs.dim {
                        fr = (fr as u32 * 6 / 10) as u8;
                        fg_c = (fg_c as u32 * 6 / 10) as u8;
                        fb = (fb as u32 * 6 / 10) as u8;
                    }
                    let attrs = Attrs::new().family(base_family).weight(weight).style(style);
                    let text = cell.ch.to_string();
                    let mut text_buf = Buffer::new(&mut self.font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
                    text_buf.set_size(&mut self.font_system, Some(cell_w as f32), Some(cell_h as f32));
                    text_buf.set_text(&mut self.font_system, &text, attrs, Shaping::Basic);
                    text_buf.shape_until_scroll(&mut self.font_system, false);
                    let base_color = Color::rgb(fr, fg_c, fb);
                    for run in text_buf.layout_runs() {
                        for glyph in run.glyphs.iter() {
                            let phys = glyph.physical((0.0, 0.0), 1.0);
                            let gx = cell_x + phys.x;
                            let gy = cell_y + run.line_y as i32 + phys.y;
                            self.swash_cache.with_pixels(&mut self.font_system, phys.cache_key, base_color,
                                |px_off, py_off, color| {
                                    let px = gx + px_off; let py = gy + py_off;
                                    if px < 0 || py < 0 { return; }
                                    let px = px as u32; let py = py as u32;
                                    if px >= width || py >= height { return; }
                                    let alpha = color.a(); if alpha == 0 { return; }
                                    let offset = (py * stride + px * 4) as usize;
                                    if offset + 3 >= canvas.len() { return; }
                                    if alpha == 255 {
                                        canvas[offset] = color.b(); canvas[offset+1] = color.g();
                                        canvas[offset+2] = color.r(); canvas[offset+3] = 0xff;
                                    } else {
                                        let a = alpha as u32; let inv = 255 - a;
                                        canvas[offset]   = ((canvas[offset]   as u32*inv + color.b() as u32*a) / 255) as u8;
                                        canvas[offset+1] = ((canvas[offset+1] as u32*inv + color.g() as u32*a) / 255) as u8;
                                        canvas[offset+2] = ((canvas[offset+2] as u32*inv + color.r() as u32*a) / 255) as u8;
                                        canvas[offset+3] = 0xff;
                                    }
                                }
                            );
                        }
                    }
                }
            }

            // Draw selection highlight
            if let (Some((sr, sc)), Some((er, ec))) = (self.sel_start, self.sel_end) {
                let (r0, c0, r1, c1) = if (sr, sc) <= (er, ec) { (sr, sc, er, ec) } else { (er, ec, sr, sc) };
                for row in r0..=r1.min(rows-1) {
                    let col_start = if row == r0 { c0 } else { 0 };
                    let col_end   = if row == r1 { c1 } else { cols };
                    let real_end = (col_start..col_end.min(cols)).rev()
                        .find(|&c| { let ch = self.terminal.grid[row][c].ch; ch != ' ' && ch != '\0' })
                        .map(|c| c + 1).unwrap_or(col_start);
                    for col in col_start..real_end {
                        let hx = (col as u32 * cell_w) as usize;
                        let hy = (row as u32 * cell_h) as usize;
                        for dy in 0..cell_h as usize {
                            for dx in 0..cell_w as usize {
                                let px = hx + dx; let py = hy + dy;
                                if px < width as usize && py < height as usize {
                                    let off = py * stride as usize + px * 4;
                                    if off + 3 < canvas.len() {
                                        // Faelight selection: dark teal, 35% opacity blend
                                        canvas[off]   = (canvas[off]   as u32 * 65 / 100 + 0x1a * 35 / 100) as u8;
                                        canvas[off+1] = (canvas[off+1] as u32 * 65 / 100 + 0x4a * 35 / 100) as u8;
                                        canvas[off+2] = (canvas[off+2] as u32 * 65 / 100 + 0x4a * 35 / 100) as u8;
                                    }
                                }
                            }
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
                        let offset = (py * stride as usize + px * 4);
                        if offset + 3 < canvas.len() {
                            canvas[offset]     = 0xa3;
                            canvas[offset + 1] = 0xe3;
                            canvas[offset + 2] = 0x6b;
                            canvas[offset + 3] = 0xff;
                        }
                    }
                }
            }
            // Friday panel -- slides in from right, 35% width
            if self.show_friday {
                let panel_w = (width * 35 / 100).max(200);
                let panel_x = width - panel_w;
                // Panel background -- deep forest dark
                for py in 0..height {
                    for px in panel_x..width {
                        let o = (py * stride + px * 4) as usize;
                        if o + 3 < canvas.len() {
                            canvas[o] = 0x08; canvas[o+1] = 0x18; canvas[o+2] = 0x10; canvas[o+3] = 0xff;
                        }
                    }
                }
                // Panel border -- teal left edge
                for py in 0..height {
                    let o = (py * stride + panel_x * 4) as usize;
                    if o + 3 < canvas.len() {
                        canvas[o] = 0x7f; canvas[o+1] = 0xc8; canvas[o+2] = 0xc8; canvas[o+3] = 0xff;
                    }
                }
                // Render Friday panel content
                let mut py_off = self.cell_h as i32 / 2;
                let px_off = panel_x as i32 + 10;
                let panel_render_w = panel_w.saturating_sub(20) as f32;
                // Title
                // Title: "FRIDAY" in green + " // Knowledge" in cyan on one line
                let title_str = "FRIDAY  //  Knowledge";
                let mut tb = Buffer::new(&mut self.font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
                tb.set_size(&mut self.font_system, Some(panel_render_w), Some(LINE_HEIGHT));
                let ta = Attrs::new().family(cosmic_text::Family::Name("JetBrainsMono Nerd Font Mono")).weight(cosmic_text::Weight::BOLD);
                tb.set_text(&mut self.font_system, title_str, ta, Shaping::Basic);
                tb.shape_until_scroll(&mut self.font_system, false);
                let sc = Color::rgb(107, 227, 163);
                for run in tb.layout_runs() {
                    for g in run.glyphs.iter() {
                        let phys = g.physical((0.0,0.0),1.0);
                        let gx = px_off + phys.x;
                        let gy = py_off + run.line_y as i32 + phys.y;
                        self.swash_cache.with_pixels(&mut self.font_system, phys.cache_key, sc, |dx,dy,color| {
                            let (px2,py2) = ((gx+dx) as u32, (gy+dy) as u32);
                            if px2 >= width || py2 >= height { return; }
                            let al = color.a(); if al == 0 { return; }
                            let poff = (py2*stride+px2*4) as usize;
                            if poff+3 >= canvas.len() { return; }
                            canvas[poff]=color.b(); canvas[poff+1]=color.g(); canvas[poff+2]=color.r(); canvas[poff+3]=0xff;
                        });
                    }
                }
                py_off += self.cell_h as i32 * 2;
                // Knowledge entries
                for (domain, fact, confidence) in &friday_data_cache {
                    if py_off + self.cell_h as i32 * 2 > height as i32 { break; }
                    // Domain label color
                    let dom_col = match domain.as_str() {
                        "rust"          => [245, 193, 119],
                        "patterns"      => [107, 227, 163],
                        "cross_intent"  => [92, 200, 255],
                        "shell"         => [180, 140, 220],
                        "wayland"       => [230, 126, 128],
                        _               => [200, 200, 200],
                    };
                    let lines: &[(&str, [u8;3])] = &[
                        (domain.as_str(), dom_col),
                    ];
                    for (txt, col) in lines {
                        let mut tb = Buffer::new(&mut self.font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
                        tb.set_size(&mut self.font_system, Some(panel_render_w), Some(LINE_HEIGHT));
                        let ta = Attrs::new().family(cosmic_text::Family::Name("JetBrainsMono Nerd Font Mono")).weight(cosmic_text::Weight::BOLD);
                        tb.set_text(&mut self.font_system, txt, ta, Shaping::Basic);
                        tb.shape_until_scroll(&mut self.font_system, false);
                        let sc = Color::rgb(col[0], col[1], col[2]);
                        for run in tb.layout_runs() {
                            for g in run.glyphs.iter() {
                                let phys = g.physical((0.0,0.0),1.0);
                                let gx = px_off + phys.x;
                                let gy = py_off + run.line_y as i32 + phys.y;
                                self.swash_cache.with_pixels(&mut self.font_system, phys.cache_key, sc, |dx,dy,color| {
                                    let (px2,py2) = ((gx+dx) as u32, (gy+dy) as u32);
                                    if px2 >= width || py2 >= height { return; }
                                    let al = color.a(); if al == 0 { return; }
                                    let poff = (py2*stride+px2*4) as usize;
                                    if poff+3 >= canvas.len() { return; }
                                    canvas[poff]=color.b(); canvas[poff+1]=color.g(); canvas[poff+2]=color.r(); canvas[poff+3]=0xff;
                                });
                            }
                        }
                    }
                    py_off += self.cell_h as i32;
                    // Fact text -- truncated to fit panel
                    let fact_short: String = fact.chars().take(38).collect();
                    let conf_str = format!("{:.0}%  {}", confidence * 100.0, fact_short);
                    let mut tb = Buffer::new(&mut self.font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
                    tb.set_size(&mut self.font_system, Some(panel_render_w), Some(LINE_HEIGHT));
                    let ta = Attrs::new().family(cosmic_text::Family::Name("JetBrainsMono Nerd Font Mono")).weight(cosmic_text::Weight(300));
                    tb.set_text(&mut self.font_system, &conf_str, ta, Shaping::Basic);
                    tb.shape_until_scroll(&mut self.font_system, false);
                    let sc = Color::rgb(0xb0, 0xc8, 0xb8);
                    for run in tb.layout_runs() {
                        for g in run.glyphs.iter() {
                            let phys = g.physical((0.0,0.0),1.0);
                            let gx = px_off + phys.x;
                            let gy = py_off + run.line_y as i32 + phys.y;
                            self.swash_cache.with_pixels(&mut self.font_system, phys.cache_key, sc, |dx,dy,color| {
                                let (px2,py2) = ((gx+dx) as u32, (gy+dy) as u32);
                                if px2 >= width || py2 >= height { return; }
                                let al = color.a(); if al == 0 { return; }
                                let poff = (py2*stride+px2*4) as usize;
                                if poff+3 >= canvas.len() { return; }
                                canvas[poff]=color.b(); canvas[poff+1]=color.g(); canvas[poff+2]=color.r(); canvas[poff+3]=0xff;
                            });
                        }
                    }
                    py_off += self.cell_h as i32 + 8;
                }
            }
            if self.show_status {
                let strip_h = self.cell_h;
                let strip_y = height.saturating_sub(strip_h);
                for dy in 0..strip_h {
                    for dx in 0..width {
                        let o = ((strip_y+dy)*stride+dx*4) as usize;
                        if o+3 < canvas.len() {
                            canvas[o]=0x0a; canvas[o+1]=0x22; canvas[o+2]=0x18; canvas[o+3]=0xff;
                        }
                    }
                }
                let status_str = status_str_cache.clone();
                let parts: Vec<&str> = status_str.split('|').collect();
                let mut xoff = 12i32;
                let mut si = 0usize;
                while si + 1 < parts.len() {
                    let txt = parts[si];
                    let cstr = parts[si+1];
                    let rgb: Vec<u8> = cstr.split(',').filter_map(|s| s.trim().parse().ok()).collect();
                    let sc = if rgb.len()==3 { Color::rgb(rgb[0],rgb[1],rgb[2]) } else { Color::rgb(0xd7,0xe0,0xda) };
                    let mut sb = Buffer::new(&mut self.font_system, Metrics::new(FONT_SIZE, strip_h as f32));
                    sb.set_size(&mut self.font_system, Some(width as f32), Some(strip_h as f32));
                    let sa = Attrs::new().family(cosmic_text::Family::Name("JetBrainsMono Nerd Font Mono")).weight(cosmic_text::Weight(300));
                    sb.set_text(&mut self.font_system, txt, sa, Shaping::Basic);
                    sb.shape_until_scroll(&mut self.font_system, false);
                    for run in sb.layout_runs() {
                        for g in run.glyphs.iter() {
                            let phys = g.physical((0.0,0.0),1.0);
                            let gx = xoff+phys.x; let gy = strip_y as i32+run.line_y as i32+phys.y;
                            self.swash_cache.with_pixels(&mut self.font_system, phys.cache_key, sc, |dx,dy,color| {
                                let px=gx+dx; let py=gy+dy;
                                if px<0||py<0 { return; }
                                let (px,py)=(px as u32,py as u32);
                                if px>=width||py>=height { return; }
                                let al=color.a(); if al==0 { return; }
                                let poff=(py*stride+px*4) as usize;
                                if poff+3>=canvas.len() { return; }
                                canvas[poff]=color.b(); canvas[poff+1]=color.g();
                                canvas[poff+2]=color.r(); canvas[poff+3]=0xff;
                            });
                        }
                    }
                    xoff += (txt.chars().count() as i32 * self.cell_w as i32) + 20;
                    si += 2;
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
            self.width  = w.get();
            self.height = h.get();
            // Resize pool
            let needed = (self.width * self.height * 4) as usize;
            if let Err(e) = self.pool.resize(needed) {
                eprintln!("pool resize error: {}", e);
            }
            // Recalculate terminal grid dimensions
            let cell_w = self.cell_w.max(1);
            let cell_h = self.cell_h.max(1);
            let new_cols = (self.width / cell_w).max(1) as usize;
            let new_rows = (self.height / cell_h).max(1) as usize;
            if new_cols != self.terminal.cols || new_rows != self.terminal.rows {
                self.terminal.resize(new_cols, new_rows);
                self.pty.resize(new_cols as u16, new_rows as u16);
                // Trigger shell to redraw by sending SIGWINCH to PTY process group
                unsafe {
                    nix::libc::killpg(
                        nix::libc::tcgetpgrp(self.pty.master),
                        nix::libc::SIGWINCH
                    );
                }
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
        let ctrl  = self.modifiers.ctrl;
        let shift = self.modifiers.shift;

        if ctrl && shift && (event.keysym == Keysym::f || event.keysym == Keysym::F) {
            self.show_friday = !self.show_friday;
            self.render();
            return;
        }

        if ctrl && shift && (event.keysym == Keysym::s || event.keysym == Keysym::S) {
            self.show_status = !self.show_status;
            self.render();
            return;
        }

        if ctrl && shift && (event.keysym == Keysym::c || event.keysym == Keysym::C) {
            if let Some(text) = self.get_selection_text() {
                let mut child = std::process::Command::new("wl-copy")
                    .stdin(std::process::Stdio::piped())
                    .spawn();
                if let Ok(ref mut child) = child {
                    if let Some(ref mut stdin) = child.stdin {
                        use std::io::Write;
                        stdin.write_all(text.as_bytes()).ok();
                    }
                }
            }
            return;
        }

        if ctrl && shift && (event.keysym == Keysym::v || event.keysym == Keysym::V) {
            if let Ok(output) = std::process::Command::new("wl-paste")
                .arg("--no-newline")
                .output()
            {
                if !output.stdout.is_empty() {
                    self.pty.write(&output.stdout).ok();
                }
            }
            return;
        }

        if let Some(bytes) = keysym_to_bytes(event.keysym, &event.utf8) {
            self.pty.write(&bytes).ok();
        }
    }
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, modifiers: Modifiers, _: RawModifiers, _: u32) {
        self.modifiers = modifiers;
    }
    fn update_repeat_info(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: RepeatInfo) {}
    fn repeat_key(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
}
impl PointerHandler for App {
    fn pointer_frame(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_pointer::WlPointer, events: &[PointerEvent])
    {
        use smithay_client_toolkit::seat::pointer::PointerEventKind;
        for event in events {
            match event.kind {
                PointerEventKind::Axis { vertical, .. } => {
                    let amount = 3usize;
                    if vertical.absolute < 0.0 || vertical.discrete < 0 {
                        let max = self.terminal.scrollback.len();
                        self.scroll_offset = (self.scroll_offset + amount).min(max);
                    } else if vertical.absolute > 0.0 || vertical.discrete > 0 {
                        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
                    }
                    self.render();
                }
                PointerEventKind::Press { button, .. } if button == 0x110 => {
                    self.mouse_down = true;
                    self.mouse_pos = event.position;
                    let col = (event.position.0 / self.cell_w as f64) as usize;
                    let row = (event.position.1 / self.cell_h as f64) as usize;
                    self.sel_start = Some((row.min(self.terminal.rows-1), col.min(self.terminal.cols-1)));
                    self.sel_end = None;
                }
                PointerEventKind::Motion { .. } if self.mouse_down => {
                    self.mouse_pos = event.position;
                    let col = (event.position.0 / self.cell_w as f64) as usize;
                    let row = (event.position.1 / self.cell_h as f64) as usize;
                    self.sel_end = Some((row.min(self.terminal.rows-1), col.min(self.terminal.cols-1)));
                    self.render();
                }
                PointerEventKind::Release { button, .. } if button == 0x110 => {
                    self.mouse_down = false;
                    // Auto-copy selection to clipboard
                    if let Some(text) = self.get_selection_text() {
                        let mut child = std::process::Command::new("wl-copy")
                            .stdin(std::process::Stdio::piped())
                            .spawn();
                        if let Ok(ref mut child) = child {
                            if let Some(ref mut stdin) = child.stdin {
                                use std::io::Write;
                                stdin.write_all(text.as_bytes()).ok();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
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
