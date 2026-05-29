//! faelight-term v3 -- Phase 4: keyboard input + scrollback
//! Goal: type into fsh, scrollback works, dirty tracking

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyboardHandler, KeyEvent, Keysym, Modifiers, RepeatInfo},
        pointer::{PointerHandler, PointerEvent, PointerEventKind, BTN_LEFT},
    },
    shell::{
        WaylandSurface,
        xdg::{
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
        },
    },
};
use wayland_client::{
    Connection, Proxy, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_seat, wl_surface},
};
use raw_window_handle::{
    HasDisplayHandle, HasWindowHandle,
    RawDisplayHandle, RawWindowHandle,
    WaylandDisplayHandle, WaylandWindowHandle,
};
use glyphon::{
    Attrs, Buffer, Color as GlyphonColor, Family, FontSystem, Metrics,
    Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer,
};
use alacritty_terminal::{
    Term,
    event::{Event as TermEvent, EventListener},
    event_loop::{EventLoop, Notifier},
    grid::Dimensions,
    index::{Column, Line, Point},
    sync::FairMutex,
    term::{Config as TermConfig, cell::Flags},
    tty::{self, Options as TtyOptions},
    vte::ansi::{Color as AnsiColor, NamedColor},
};
use std::{ptr::NonNull, sync::Arc, collections::HashMap};
use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};
use wl_clipboard_rs::copy::{MimeType as CopyMimeType, Options as CopyOptions, Source as CopySource};

// INT-324: forest typography -- JetBrainsMono Nerd Font, matching foot config
const FONT_SIZE: f32 = 14.0;      // INT-324: forest size
const LINE_HEIGHT: f32 = 18.0;    // 1.286x font size
const CELL_W: f32 = 8.4;          // JetBrainsMono at 14px
const CELL_H: f32 = 18.0;         // matches LINE_HEIGHT
const FONT_NAME: &str = "JetBrainsMono Nerd Font";
const PADDING: f32 = 4.0;

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().expect("wayland connection");
    let (globals, mut event_queue) = registry_queue_init::<AppState>(&conn).expect("registry");
    let qh: QueueHandle<AppState> = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("compositor");
    let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg_shell");
    let output_state = OutputState::new(&globals, &qh);
    let seat_state = SeatState::new(&globals, &qh);
    let registry_state = RegistryState::new(&globals);

    let surface = compositor.create_surface(&qh);
    let window = xdg_shell.create_window(
        surface.clone(),
        WindowDecorations::RequestServer,
        &qh,
    );
    window.set_title("faelight-term v3");
    window.set_app_id("faelight-term-v3");
    window.set_min_size(Some((800, 600)));
    window.commit();

    let display_ptr = conn.display().id().as_ptr() as *mut _;

    let mut state = AppState {
        registry_state,
        output_state,
        seat_state,
        compositor_state: compositor,
        xdg_shell,
        window,
        surface,
        configured: false,
        width: 800,
        height: 600,
        exit: false,
        gpu: None,
        display_ptr,
        modifiers: Modifiers::default(),
        wl_seat: None,
        scroll_accumulator: 0.0,
        selection_start: None,
        selection_end: None,
        selecting: false,
        selected_text: String::new(),
    };

    event_queue.roundtrip(&mut state).expect("roundtrip");
    event_queue.roundtrip(&mut state).expect("roundtrip 2");

    // Grab pointer from all seats after roundtrips
    let seats: Vec<_> = state.seat_state.seats().collect();
    for seat in seats {
        let _ = state.seat_state.get_pointer(&qh, &seat);
    }
    event_queue.roundtrip(&mut state).expect("roundtrip 3");

    // Initial render to break Wayland deadlock
    if state.configured {
        if let Some(ref mut gpu) = state.gpu {
            gpu.sync_terminal(state.selection_start, state.selection_end);
            gpu.render();
        }
    }
    event_queue.flush().expect("flush");

    while !state.exit {
        // Render if dirty
        if let Some(ref mut gpu) = state.gpu {
            if gpu.dirty {
                gpu.sync_terminal(state.selection_start, state.selection_end);
                gpu.render();
                gpu.dirty = false;
            }
        }
        // Read from Wayland socket then dispatch -- required for resize/configure events
        event_queue.flush().ok();
        if let Some(guard) = event_queue.prepare_read() {
            guard.read().ok(); // non-blocking: reads available events from compositor
        }
        if let Err(_) = event_queue.dispatch_pending(&mut state) { break; }
        // Poll at ~60fps -- only render when PTY signals new data
        std::thread::sleep(std::time::Duration::from_millis(16));
        // Mark dirty only when PTY has new output (check via listener needs_render)
        if let Some(ref mut gpu) = state.gpu {
            if gpu.needs_render.load(std::sync::atomic::Ordering::Relaxed) {
                gpu.dirty = true;
                gpu.needs_render.store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
}

#[derive(Clone)]
struct FaelightListener {
    pty_resp_tx: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::SyncSender<String>>>,
    needs_render: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl EventListener for FaelightListener {
    fn send_event(&self, event: TermEvent) {
        // INT-286: forward PtyWrite back to PTY -- fixes DA1/DA2, yazi TRT, app queries
        match event {
            TermEvent::PtyWrite(data) => {
                if let Ok(tx) = self.pty_resp_tx.lock() {
                    let _ = tx.try_send(data);
                }
            }
            // INT-286: OSC 52 -- app writes to Wayland clipboard
            TermEvent::ClipboardStore(_clipboard_type, text) => {
                let opts = CopyOptions::new();
                let _ = opts.copy(
                    CopySource::Bytes(text.into_bytes().into()),
                    CopyMimeType::Autodetect,
                );
            }
            // INT-286: OSC 52 -- app reads from Wayland clipboard
            TermEvent::ClipboardLoad(_clipboard_type, callback) => {
                use std::io::Read;
                if let Ok((mut reader, _)) = get_contents(
                    wl_clipboard_rs::paste::ClipboardType::Regular,
                    Seat::Unspecified,
                    MimeType::TextWithPriority("text/plain;charset=utf-8"),
                ) {
                    let mut text = String::new();
                    if reader.read_to_string(&mut text).is_ok() {
                        let response = callback(&text);
                        if let Ok(tx) = self.pty_resp_tx.lock() {
                            let _ = tx.try_send(response);
                        }
                    }
                }
            }
            TermEvent::Wakeup => {
                self.needs_render.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

#[allow(dead_code)]
struct AppState {
    registry_state: RegistryState,
    output_state: OutputState,
    seat_state: SeatState,
    compositor_state: CompositorState,
    xdg_shell: XdgShell,
    window: Window,
    surface: wl_surface::WlSurface,
    configured: bool,
    width: u32,
    height: u32,
    exit: bool,
    gpu: Option<GpuState>,
    display_ptr: *mut std::ffi::c_void,
    modifiers: Modifiers,
    wl_seat: Option<wl_seat::WlSeat>,
    scroll_accumulator: f64,
    selection_start: Option<(usize, i32)>,
    selection_end: Option<(usize, i32)>,
    selecting: bool,
    selected_text: String,
}

struct FaelightWindow {
    window_ptr: *mut std::ffi::c_void,
    display_ptr: *mut std::ffi::c_void,
}
unsafe impl Send for FaelightWindow {}
unsafe impl Sync for FaelightWindow {}

impl HasWindowHandle for FaelightWindow {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let h = WaylandWindowHandle::new(NonNull::new(self.window_ptr).unwrap());
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(RawWindowHandle::Wayland(h)) })
    }
}
impl HasDisplayHandle for FaelightWindow {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let h = WaylandDisplayHandle::new(NonNull::new(self.display_ptr).unwrap());
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(h)) })
    }
}

struct GpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    text_buffer: Buffer,
    term: Arc<FairMutex<Term<FaelightListener>>>,
    notifier: Notifier,
    pty_resp_rx: std::sync::mpsc::Receiver<String>,
    needs_render: std::sync::Arc<std::sync::atomic::AtomicBool>,
    cell_w: f32,
    cols: usize,
    rows: usize,
    dirty: bool,
    cursor_col: usize,
    cursor_row: usize,
    font_size: f32,       // INT-324: runtime font size
    line_height: f32,     // INT-324: runtime line height
}

impl GpuState {
    fn new(window: FaelightWindow, width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = instance.create_surface(window).expect("surface");
        let adapter = futures::executor::block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
        ).expect("adapter");
        let (device, queue) = futures::executor::block_on(
            adapter.request_device(&wgpu::DeviceDescriptor::default(), None)
        ).expect("device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let mut text_atlas = TextAtlas::new(&device, &queue, &cache, format);
        let text_renderer = TextRenderer::new(
            &mut text_atlas, &device,
            wgpu::MultisampleState::default(), None
        );
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
        text_buffer.set_size(&mut font_system, Some(width as f32), Some(height as f32));
        text_buffer.set_text(&mut font_system, "starting...", Attrs::new().family(Family::Name(FONT_NAME)), Shaping::Advanced);
        text_buffer.shape_until_scroll(&mut font_system, false);
        // INT-286: measure actual glyph advance width from font metrics
        let cell_w = {
            let mut m = Buffer::new(&mut font_system, Metrics::new(FONT_SIZE, LINE_HEIGHT));
            m.set_size(&mut font_system, None, None);
            m.set_text(&mut font_system, "M", Attrs::new().family(Family::Name(FONT_NAME)), Shaping::Advanced);
            m.shape_until_scroll(&mut font_system, false);
            m.layout_runs().next()
                .and_then(|r| r.glyphs.first())
                .map(|g| g.w)
                .unwrap_or(CELL_W)
        };

        let cols = ((width as f32 - PADDING * 2.0) / cell_w) as usize;
        let rows = ((height as f32 - PADDING * 2.0) / CELL_H) as usize;
        let cols = cols.max(10);
        let rows = rows.max(3);

        let term_config = TermConfig::default();
        let (pty_resp_tx, pty_resp_rx) = std::sync::mpsc::sync_channel::<String>(64);
        let pty_resp_tx = std::sync::Arc::new(std::sync::Mutex::new(pty_resp_tx));
        let needs_render = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let listener = FaelightListener { pty_resp_tx, needs_render: needs_render.clone() };
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        // INT-286: set terminal environment -- TERM=linux causes yazi/tools to fail
        let mut term_env = HashMap::new();
        term_env.insert("TERM".to_string(), "xterm-256color".to_string());
        term_env.insert("COLORTERM".to_string(), "truecolor".to_string());
        term_env.insert("TERM_PROGRAM".to_string(), "faelight-term".to_string());
        let pty_options = TtyOptions {
            shell: Some(alacritty_terminal::tty::Shell::new(shell, vec![])),
            working_directory: Some(std::path::PathBuf::from(
                std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
            )),
            env: term_env,
            hold: false,
        };
        let window_size = alacritty_terminal::event::WindowSize {
            num_cols: cols as u16,
            num_lines: rows as u16,
            cell_width: cell_w as u16,
            cell_height: CELL_H as u16,
        };

        let term = Term::new(term_config, &TermDimensions { cols, rows }, listener.clone());
        let term = Arc::new(FairMutex::new(term));
        let pty = tty::new(&pty_options, window_size, 0u64).expect("PTY");
        let event_loop = EventLoop::new(term.clone(), listener, pty, false, false).expect("event loop");
        let notifier = Notifier(event_loop.channel());
        let _thread = event_loop.spawn();

        // Set window title to reflect forest context

        Self {
            device, queue, surface, config,
            font_system, swash_cache, text_atlas, text_renderer, text_buffer,
            term, notifier, pty_resp_rx, needs_render, cell_w, cols, rows, dirty: true, cursor_col: 0, cursor_row: 0, font_size: FONT_SIZE, line_height: LINE_HEIGHT,
        }
    }

    fn write_to_pty(&mut self, data: &[u8]) {
        use alacritty_terminal::event_loop::Msg;
        let _ = self.notifier.0.send(Msg::Input(data.to_vec().into()));
        self.dirty = true;
    }

    fn resize(&mut self, width: u32, height: u32) {
        let cols = ((width as f32 - PADDING * 2.0) / self.cell_w) as usize;
        let rows = ((height as f32 - PADDING * 2.0) / CELL_H) as usize;
        let cols = cols.max(10);
        let rows = rows.max(3);
        if cols == self.cols && rows == self.rows { return; }
        // Resize wgpu surface
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        // Resize glyphon text buffer
        self.text_buffer.set_size(&mut self.font_system, Some(width as f32), Some(height as f32));
        // Resize alacritty terminal grid
        {
            let mut term = self.term.lock();
            term.resize(TermDimensions { cols, rows });
        }
        // Send SIGWINCH to PTY via notifier
        {
            use alacritty_terminal::event_loop::Msg;
            use alacritty_terminal::event::WindowSize;
            let window_size = WindowSize {
                num_cols: cols as u16,
                num_lines: rows as u16,
                cell_width: self.cell_w as u16,
                cell_height: CELL_H as u16,
            };
            let _ = self.notifier.0.send(Msg::Resize(window_size));
        }
        self.cols = cols;
        self.rows = rows;
        self.dirty = true;
    }


    /// Map alacritty AnsiColor to glyphon Color
    fn ansi_to_glyphon(color: AnsiColor, _is_bold: bool) -> glyphon::Color {
        match color {
            AnsiColor::Named(NamedColor::Black)         => glyphon::Color::rgb(0x1a, 0x1f, 0x1a),
            AnsiColor::Named(NamedColor::Red)           => glyphon::Color::rgb(0xe0, 0x6c, 0x75),
            AnsiColor::Named(NamedColor::Green)         => glyphon::Color::rgb(0x5a, 0xb0, 0x6e),
            AnsiColor::Named(NamedColor::Yellow)        => glyphon::Color::rgb(0xe5, 0xc0, 0x7b),
            AnsiColor::Named(NamedColor::Blue)          => glyphon::Color::rgb(0x61, 0xaf, 0xef),
            AnsiColor::Named(NamedColor::Magenta)       => glyphon::Color::rgb(0xc6, 0x78, 0xdd),
            AnsiColor::Named(NamedColor::Cyan)          => glyphon::Color::rgb(0x56, 0xb6, 0xc2),
            AnsiColor::Named(NamedColor::White)         => glyphon::Color::rgb(0xd7, 0xe0, 0xda),
            AnsiColor::Named(NamedColor::BrightBlack)   => glyphon::Color::rgb(0x5c, 0x63, 0x70),
            AnsiColor::Named(NamedColor::BrightRed)     => glyphon::Color::rgb(0xe0, 0x6c, 0x75),
            AnsiColor::Named(NamedColor::BrightGreen)   => glyphon::Color::rgb(0x7e, 0xc2, 0x8e),
            AnsiColor::Named(NamedColor::BrightYellow)  => glyphon::Color::rgb(0xe5, 0xc0, 0x7b),
            AnsiColor::Named(NamedColor::BrightBlue)    => glyphon::Color::rgb(0x61, 0xaf, 0xef),
            AnsiColor::Named(NamedColor::BrightMagenta) => glyphon::Color::rgb(0xc6, 0x78, 0xdd),
            AnsiColor::Named(NamedColor::BrightCyan)    => glyphon::Color::rgb(0x56, 0xb6, 0xc2),
            AnsiColor::Named(NamedColor::BrightWhite)   => glyphon::Color::rgb(0xff, 0xff, 0xff),
            AnsiColor::Named(NamedColor::Foreground)    => glyphon::Color::rgb(0xd7, 0xe0, 0xda),
            AnsiColor::Named(NamedColor::Background)    => glyphon::Color::rgb(0x11, 0x14, 0x0f),
            AnsiColor::Indexed(i) => {
                // 256-color palette -- basic implementation
                match i {
                    0  => glyphon::Color::rgb(0x1a, 0x1f, 0x1a),
                    1  => glyphon::Color::rgb(0xe0, 0x6c, 0x75),
                    2  => glyphon::Color::rgb(0x5a, 0xb0, 0x6e),
                    3  => glyphon::Color::rgb(0xe5, 0xc0, 0x7b),
                    4  => glyphon::Color::rgb(0x61, 0xaf, 0xef),
                    5  => glyphon::Color::rgb(0xc6, 0x78, 0xdd),
                    6  => glyphon::Color::rgb(0x56, 0xb6, 0xc2),
                    7  => glyphon::Color::rgb(0xd7, 0xe0, 0xda),
                    8  => glyphon::Color::rgb(0x5c, 0x63, 0x70),
                    9  => glyphon::Color::rgb(0xe0, 0x6c, 0x75),
                    10 => glyphon::Color::rgb(0x7e, 0xc2, 0x8e),
                    11 => glyphon::Color::rgb(0xe5, 0xc0, 0x7b),
                    12 => glyphon::Color::rgb(0x61, 0xaf, 0xef),
                    13 => glyphon::Color::rgb(0xc6, 0x78, 0xdd),
                    14 => glyphon::Color::rgb(0x56, 0xb6, 0xc2),
                    15 => glyphon::Color::rgb(0xff, 0xff, 0xff),
                    _ => {
                        // 6x6x6 color cube (16-231) and grayscale (232-255)
                        if i >= 232 {
                            let v = 8 + (i - 232) * 10;
                            glyphon::Color::rgb(v, v, v)
                        } else if i >= 16 {
                            let idx = i - 16;
                            let b = (idx % 6) * 51;
                            let g = ((idx / 6) % 6) * 51;
                            let r = (idx / 36) * 51;
                            glyphon::Color::rgb(r, g, b)
                        } else {
                            glyphon::Color::rgb(0xd7, 0xe0, 0xda)
                        }
                    }
                }
            }
            AnsiColor::Spec(rgb) => glyphon::Color::rgb(rgb.r, rgb.g, rgb.b),
            _ => glyphon::Color::rgb(0xd7, 0xe0, 0xda),
        }
    }

    fn sync_terminal(&mut self, sel_start: Option<(usize, i32)>, sel_end: Option<(usize, i32)>) {
        let term = self.term.lock();
        let grid = term.grid();
        // Account for scroll position -- display_offset shifts the visible window
        let display_offset = grid.display_offset() as i32;
        // Build spans with per-cell colors
        let mut spans: Vec<(String, glyphon::Attrs<'static>)> = Vec::new();
        for line in 0..self.rows {
            for col in 0..self.cols {
                // Adjust line by display_offset: positive offset = scrolled up into history
                let grid_line = line as i32 - display_offset;
                let point = Point::new(Line(grid_line), Column(col));
                let cell = &grid[point];
                let ch = if cell.c == '\0' { ' ' } else { cell.c };
                let fg = Self::ansi_to_glyphon(cell.fg, cell.flags.contains(Flags::BOLD));
                let bold = cell.flags.contains(Flags::BOLD);
                let attrs = Attrs::new()
                    .family(Family::Name(FONT_NAME))
                    .color(fg)
                    .weight(if bold { glyphon::fontdb::Weight::BOLD } else { glyphon::fontdb::Weight::NORMAL });
                spans.push((ch.to_string(), attrs));
            }
            // Newline -- add to last span
            if let Some(last) = spans.last_mut() {
                last.0.push('\n');
            } else {
                spans.push(("\n".to_string(), Attrs::new().family(Family::Name(FONT_NAME))));
            }
        }
                // Capture cursor position before drop
        self.cursor_col = grid.cursor.point.column.0;
        // Cursor position adjusted for scroll -- hide cursor when scrolled
        let cursor_line = grid.cursor.point.line.0 + display_offset;
        self.cursor_row = if cursor_line >= 0 && cursor_line < self.rows as i32 { cursor_line as usize } else { usize::MAX };
        drop(term);
        // Convert spans to glyphon AttrsList format
        // Mark cursor cell -- bright green highlight
        let cursor_idx = self.cursor_row * self.cols + self.cursor_col;
        if cursor_idx < spans.len() {
            // Replace space with block cursor character
            let cursor_char = spans[cursor_idx].0.chars().next().unwrap_or(' ');
            if cursor_char == ' ' || cursor_char == '\0' {
                spans[cursor_idx].0 = "█".to_string(); // full block
            }
            spans[cursor_idx].1 = Attrs::new()
                .family(Family::Name(FONT_NAME))
                .color(glyphon::Color::rgb(0x5a, 0xb0, 0x6e));
        }

        // Highlight selection region
        if let (Some((sc, sr)), Some((ec, er))) = (sel_start, sel_end) {
            // sr/er are global i32 line indices; convert to viewport rows for span indexing
            let (r1, c1, r2, c2) = if sr < er || (sr == er && sc <= ec) {
                (sr, sc, er, ec)
            } else {
                (er, ec, sr, sc)
            };
            for row in r1..=r2 {
                let viewport_row = row + display_offset;
                if viewport_row < 0 || viewport_row >= self.rows as i32 { continue; }
                let viewport_row = viewport_row as usize;
                let col_start = if row == r1 { c1 } else { 0 };
                let col_end = if row == r2 { c2 } else { self.cols.saturating_sub(1) };
                for col in col_start..=col_end {
                    let sel_idx = viewport_row * self.cols + col;
                    if sel_idx < spans.len() && sel_idx != cursor_idx {
                        spans[sel_idx].1 = Attrs::new()
                            .family(Family::Name(FONT_NAME))
                            .color(glyphon::Color::rgb(0x11, 0x14, 0x0f)) // dark text on highlight
                            ;
                        // We can't set background via glyphon spans directly
                        // Use a bright color to indicate selection
                        spans[sel_idx].1 = Attrs::new()
                            .family(Family::Name(FONT_NAME))
                            .color(glyphon::Color::rgb(0xff, 0xd7, 0x00)); // gold selection
                    }
                }
            }
        }

        let span_refs: Vec<(&str, glyphon::Attrs)> = spans.iter()
            .map(|(s, a)| (s.as_str(), *a))
            .collect();
        self.text_buffer.set_rich_text(
            &mut self.font_system,
            span_refs.into_iter(),
            Attrs::new().family(Family::Name(FONT_NAME)),
            Shaping::Advanced,
        );
        self.text_buffer.shape_until_scroll(&mut self.font_system, false);
    }

    fn render(&mut self) {
        // INT-286: drain PTY response buffer (DA1/DA2, kitty queries, etc.)
        use alacritty_terminal::event_loop::Msg;
        while let Ok(data) = self.pty_resp_rx.try_recv() {
            let _ = self.notifier.0.send(Msg::Input(data.into_bytes().into()));
        }
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let cache = glyphon::Cache::new(&self.device);
        let mut viewport = glyphon::Viewport::new(&self.device, &cache);
        viewport.update(&self.queue, Resolution {
            width: self.config.width,
            height: self.config.height,
        });
        let _ = self.text_renderer.prepare(
            &self.device, &self.queue,
            &mut self.font_system, &mut self.text_atlas, &viewport,
            [TextArea {
                buffer: &self.text_buffer,
                left: PADDING, top: PADDING, scale: 1.0,
                bounds: TextBounds {
                    left: 0, top: 0,
                    right: self.config.width as i32,
                    bottom: self.config.height as i32,
                },
                default_color: GlyphonColor::rgb(0xd7, 0xe0, 0xda),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        );
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("frame") }
        );
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0039, g: 0.0055, b: 0.0027, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let _ = self.text_renderer.render(&self.text_atlas, &viewport, &mut pass);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.text_atlas.trim();
    }
}

struct TermDimensions { cols: usize, rows: usize }
impl Dimensions for TermDimensions {
    fn total_lines(&self) -> usize { self.rows }
    fn screen_lines(&self) -> usize { self.rows }
    fn columns(&self) -> usize { self.cols }
    fn last_column(&self) -> Column { Column(self.cols - 1) }
}

// Keyboard handling -- wire keypresses to PTY
impl KeyboardHandler for AppState {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32,
        _: &[u32], _: &[Keysym]) {}

    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}

    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        // DEBUG INT-313: log key events -- remove after Ctrl+[ fixed
        {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .append(true).create(true)
                .open("/tmp/faelight-term-keys.log") {
                let _ = writeln!(f, "keysym={:?} ctrl={} shift={} utf8={:?}",
                    event.keysym, self.modifiers.ctrl, self.modifiers.shift, event.utf8);
            }
        }
        if let Some(ref mut gpu) = self.gpu {
            let mods = &self.modifiers;
            let ctrl = mods.ctrl;
            let shift = mods.shift;
            // Ctrl+Shift+V -- paste from Wayland clipboard (threaded to avoid deadlock)
            if ctrl && shift && (event.keysym == Keysym::v || event.keysym == Keysym::V) {
                use alacritty_terminal::event_loop::Msg;
                let notifier = gpu.notifier.0.clone();
                let term_ref = gpu.term.clone();
                std::thread::spawn(move || {
                    use std::io::Read;
                    use alacritty_terminal::term::TermMode;
                    match get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::TextWithPriority("text/plain;charset=utf-8")) {
                        Ok((mut reader, _)) => {
                            let mut text = String::new();
                            if reader.read_to_string(&mut text).is_ok() && !text.is_empty() {
                                let bracketed = term_ref.lock().mode().contains(TermMode::BRACKETED_PASTE);
                                let payload = if bracketed {
                                    format!("[200~{}[201~", text)
                                } else {
                                    text
                                };
                                let _ = notifier.send(Msg::Input(payload.into_bytes().into()));
                            }
                        }
                        Err(e) => eprintln!("[v3] paste error: {:?}", e),
                    }
                });
                return;
            }
            let ctrl = mods.ctrl;
            // INT-324: Ctrl+= increase font size, Ctrl+- decrease, Ctrl+0 reset
            let font_changed = if ctrl && (event.keysym == Keysym::equal || event.keysym == Keysym::plus) {
                gpu.font_size = (gpu.font_size + 1.0).min(32.0); true
            } else if ctrl && event.keysym == Keysym::minus {
                gpu.font_size = (gpu.font_size - 1.0).max(8.0); true
            } else if ctrl && (event.keysym == Keysym::_0 || event.keysym == Keysym::KP_0) {
                gpu.font_size = FONT_SIZE; true
            } else { false };

            if font_changed {
                // Recalculate all metrics together
                gpu.line_height = gpu.font_size * 1.286;
                // cell_w scales proportionally with font size
                gpu.cell_w = gpu.font_size * 0.6;
                let cell_h = gpu.line_height;
                gpu.text_buffer.set_metrics(&mut gpu.font_system,
                    glyphon::Metrics::new(gpu.font_size, gpu.line_height));
                // Recalculate cols/rows from window size
                let w = gpu.config.width as f32;
                let h = gpu.config.height as f32;
                let new_cols = ((w - PADDING * 2.0) / gpu.cell_w) as usize;
                let new_rows = ((h - PADDING * 2.0) / cell_h) as usize;
                gpu.cols = new_cols.max(1);
                gpu.rows = new_rows.max(1);
                gpu.dirty = true;
                return;
            }

            // INT-286: enhanced key handler — F1-F12, Ctrl/Alt/Shift modifiers
            let alt  = mods.alt;
            let bytes: Option<Vec<u8>> = match event.keysym {
                Keysym::Return | Keysym::KP_Enter => {
                    if shift { Some(b"\x1b[13;2u".to_vec()) }
                    else { Some(b"\r".to_vec()) }
                }
                Keysym::BackSpace => Some(b"\x7f".to_vec()),
                Keysym::Tab => {
                    if shift { Some(b"\x1b[Z".to_vec()) }
                    else { Some(b"\t".to_vec()) }
                }
                Keysym::Escape => Some(b"\x1b".to_vec()),
                Keysym::bracketleft => {
                    // Ctrl+[ = ESC -- check both ctrl flag and raw modifier bits
                    if ctrl || self.modifiers.ctrl {
                        Some(vec![0x1b])
                    } else {
                        Some(b"[".to_vec())
                    }
                }
                Keysym::Up    => if ctrl { Some(b"\x1b[1;5A".to_vec()) }
                                 else if shift { Some(b"\x1b[1;2A".to_vec()) }
                                 else { Some(b"\x1b[A".to_vec()) },
                Keysym::Down  => if ctrl { Some(b"\x1b[1;5B".to_vec()) }
                                 else if shift { Some(b"\x1b[1;2B".to_vec()) }
                                 else { Some(b"\x1b[B".to_vec()) },
                Keysym::Right => if ctrl { Some(b"\x1b[1;5C".to_vec()) }
                                 else if shift { Some(b"\x1b[1;2C".to_vec()) }
                                 else { Some(b"\x1b[C".to_vec()) },
                Keysym::Left  => if ctrl { Some(b"\x1b[1;5D".to_vec()) }
                                 else if shift { Some(b"\x1b[1;2D".to_vec()) }
                                 else { Some(b"\x1b[D".to_vec()) },
                Keysym::Home      => Some(b"\x1b[H".to_vec()),
                Keysym::End       => Some(b"\x1b[F".to_vec()),
                Keysym::Delete    => Some(b"\x1b[3~".to_vec()),
                Keysym::Insert    => Some(b"\x1b[2~".to_vec()),
                Keysym::Page_Up   => Some(b"\x1b[5~".to_vec()),
                Keysym::Page_Down => Some(b"\x1b[6~".to_vec()),
                Keysym::F1  => Some(b"\x1bOP".to_vec()),
                Keysym::F2  => Some(b"\x1bOQ".to_vec()),
                Keysym::F3  => Some(b"\x1bOR".to_vec()),
                Keysym::F4  => Some(b"\x1bOS".to_vec()),
                Keysym::F5  => Some(b"\x1b[15~".to_vec()),
                Keysym::F6  => Some(b"\x1b[17~".to_vec()),
                Keysym::F7  => Some(b"\x1b[18~".to_vec()),
                Keysym::F8  => Some(b"\x1b[19~".to_vec()),
                Keysym::F9  => Some(b"\x1b[20~".to_vec()),
                Keysym::F10 => Some(b"\x1b[21~".to_vec()),
                Keysym::F11 => Some(b"\x1b[23~".to_vec()),
                Keysym::F12 => Some(b"\x1b[24~".to_vec()),
                _ => {
                    if let Some(s) = event.utf8 {
                        if ctrl && s.len() == 1 {
                            let ch = s.chars().next().unwrap();
                            if ch >= 'a' && ch <= 'z' { Some(vec![ch as u8 - b'a' + 1]) }
                            else if ch >= 'A' && ch <= 'Z' { Some(vec![ch as u8 - b'A' + 1]) }
                            else if ch == '@' { Some(vec![0]) }
                            else if ch == '[' { Some(vec![0x1b]) }  // Ctrl+[ = ESC (kitty compat)
                            else if ch == '\\' { Some(vec![0x1c]) } // Ctrl+\ = FS
                            else if ch == ']' { Some(vec![0x1d]) }  // Ctrl+] = GS
                            else if ch == '^' { Some(vec![0x1e]) }  // Ctrl+^ = RS
                            else if ch == '_' { Some(vec![0x1f]) }  // Ctrl+_ = US
                            else { Some(s.into_bytes()) }
                        } else if alt && s.len() == 1 {
                            let mut v = vec![0x1b];
                            v.extend_from_slice(s.as_bytes());
                            Some(v)
                        } else { Some(s.into_bytes()) }
                    } else { None }
                }
            };
            if let Some(bytes) = bytes {
                gpu.write_to_pty(&bytes);
            }
        }
    }

    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}

    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: u32, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    fn update_repeat_info(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard, _: RepeatInfo) {}
}

impl CompositorHandler for AppState {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
}
impl OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}
impl WindowHandler for AppState {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.exit = true;
    }
    fn configure(&mut self, _: &Connection, _qh: &QueueHandle<Self>,
        _: &Window, configure: WindowConfigure, _: u32) {
        let (w, h) = configure.new_size;
        if let Some(w) = w { self.width = w.get(); }
        if let Some(h) = h { self.height = h.get(); }
        if !self.configured {
            self.configured = true;
            let window_ptr = self.surface.id().as_ptr() as *mut _;
            let fw = FaelightWindow { window_ptr, display_ptr: self.display_ptr };
            self.gpu = Some(GpuState::new(fw, self.width, self.height));
            // Create pointer now that we have a configured window
            if let Some(ref seat) = self.wl_seat.clone() {
                let _ = self.seat_state.get_pointer(_qh, seat);
            }
        } else {
            // Subsequent configure = window resize
            if let Some(ref mut gpu) = self.gpu {
                gpu.resize(self.width, self.height);
            }
        }
    }
}

impl PointerHandler for AppState {
    fn pointer_frame(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wayland_client::protocol::wl_pointer::WlPointer,
        events: &[PointerEvent]) {
        for event in events {
            match event.kind {
                PointerEventKind::Press { button, .. } if button == BTN_LEFT => {
                    // INT-286: forward click to app if mouse tracking enabled
                    if let Some(ref mut gpu) = self.gpu {
                        use alacritty_terminal::term::TermMode;
                        let mouse_enabled = gpu.term.lock().mode().intersects(
                            TermMode::MOUSE_REPORT_CLICK | TermMode::MOUSE_MOTION | TermMode::MOUSE_DRAG
                        );
                        if mouse_enabled {
                            let col = ((event.position.0 as f32 - PADDING) / gpu.cell_w) as usize + 1;
                            let row = ((event.position.1 as f32 - PADDING) / CELL_H) as usize + 1;
                            let seq = format!("\x1b[<0;{};{}M", col, row);
                            gpu.write_to_pty(seq.as_bytes());
                            return; // skip selection when app handles mouse
                        }
                    }
                    if let Some(ref gpu) = self.gpu {
                        let col = ((event.position.0 as f32 - PADDING) / gpu.cell_w) as usize;
                        let row = ((event.position.1 as f32 - PADDING) / CELL_H) as usize;
                        let col = col.min(gpu.cols.saturating_sub(1));
                        let row = row.min(gpu.rows.saturating_sub(1));
                        let display_offset = gpu.term.lock().grid().display_offset() as i32;
                        let global_line = row as i32 - display_offset;
                        self.selection_start = Some((col, global_line));
                        self.selection_end = Some((col, global_line));
                        self.selecting = true;
                    }
                }
                PointerEventKind::Release { button, .. } if button == BTN_LEFT => {
                    // INT-286: forward release to app if mouse tracking enabled
                    if let Some(ref mut gpu) = self.gpu {
                        use alacritty_terminal::term::TermMode;
                        let mouse_enabled = gpu.term.lock().mode().intersects(
                            TermMode::MOUSE_REPORT_CLICK | TermMode::MOUSE_MOTION | TermMode::MOUSE_DRAG
                        );
                        if mouse_enabled {
                            let col = ((event.position.0 as f32 - PADDING) / gpu.cell_w) as usize + 1;
                            let row = ((event.position.1 as f32 - PADDING) / CELL_H) as usize + 1;
                            let seq = format!("\x1b[<0;{};{}m", col, row);
                            gpu.write_to_pty(seq.as_bytes());
                        }
                    }
                    self.selecting = false;
                    // Build selected text from terminal grid
                    if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                        if let Some(ref gpu) = self.gpu {
                            let term = gpu.term.lock();
                            let grid = term.grid();
                            // Use global line coords -- no display_offset adjustment needed here
                            let mut text = String::new();
                            let (start_col, start_line) = start;
                            let (end_col, end_line) = end;
                            let (r1, c1, r2, c2) = if start_line < end_line || (start_line == end_line && start_col <= end_col) {
                                (start_line, start_col, end_line, end_col)
                            } else {
                                (end_line, end_col, start_line, start_col)
                            };
                            for row in r1..=r2 {
                                let col_start = if row == r1 { c1 } else { 0 };
                                let col_end = if row == r2 { c2 } else { gpu.cols.saturating_sub(1) };
                                for col in col_start..=col_end {
                                    use alacritty_terminal::index::{Column, Line, Point};
                                    // row IS the global grid line
                                    let point = Point::new(Line(row), Column(col));
                                    let cell = &grid[point];
                                    let ch = if cell.c == '\0' { ' ' } else { cell.c };
                                    text.push(ch);
                                }
                                if row < r2 { text.push('\n'); }
                            }
                            drop(term);
                            let text = text.trim_end().to_string();
                            if !text.is_empty() {
                                self.selected_text = text.clone();
                                // Copy to clipboard in background thread
                                std::thread::spawn(move || {
                                    let mut opts = CopyOptions::new();
                                    opts.foreground(true); // serve in-thread, no fork -- fixes paste to browser
                                    let _ = opts.copy(
                                        CopySource::Bytes(text.into_bytes().into()),
                                        CopyMimeType::Text,
                                    );
                                });
                            }
                        }
                    }
                }
                PointerEventKind::Motion { .. } if self.selecting => {
                    if let Some(ref gpu) = self.gpu {
                        let col = ((event.position.0 as f32 - PADDING) / gpu.cell_w) as usize;
                        let row = ((event.position.1 as f32 - PADDING) / CELL_H) as usize;
                        let col = col.min(gpu.cols.saturating_sub(1));
                        let row = row.min(gpu.rows.saturating_sub(1));
                        let display_offset = gpu.term.lock().grid().display_offset() as i32;
                        let global_line = row as i32 - display_offset;
                        self.selection_end = Some((col, global_line));
                        if let Some(ref mut gpu) = self.gpu {
                            gpu.dirty = true;
                        }
                    }
                }
                PointerEventKind::Axis { horizontal: _, vertical, .. } => {
                    // INT-286: accumulate scroll, fire at threshold to prevent too-fast scroll
                    self.scroll_accumulator += vertical.absolute;
                    const SCROLL_THRESHOLD: f64 = 15.0;
                    while self.scroll_accumulator.abs() >= SCROLL_THRESHOLD {
                        let direction = if self.scroll_accumulator < 0.0 { -1.0 } else { 1.0 };
                        self.scroll_accumulator -= SCROLL_THRESHOLD * direction;
                        if let Some(ref mut gpu) = self.gpu {
                            use alacritty_terminal::term::TermMode;
                            use alacritty_terminal::grid::Scroll;
                            let mouse_enabled = gpu.term.lock().mode().intersects(
                                TermMode::MOUSE_REPORT_CLICK | TermMode::MOUSE_MOTION | TermMode::MOUSE_DRAG
                            );
                            let col = ((event.position.0 as f32 - PADDING) / gpu.cell_w) as usize + 1;
                            let row = ((event.position.1 as f32 - PADDING) / CELL_H) as usize + 1;
                            if mouse_enabled {
                                let btn = if direction < 0.0 { 64 } else { 65 };
                                let seq = format!("\x1b[<{};{};{}M", btn, col, row);
                                gpu.write_to_pty(seq.as_bytes());
                            } else {
                                let mut term = gpu.term.lock();
                                if direction < 0.0 {
                                    term.scroll_display(Scroll::Delta(1));
                                } else {
                                    term.scroll_display(Scroll::Delta(-1));
                                }
                            }
                            gpu.dirty = true;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl SeatHandler for AppState {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, seat: wl_seat::WlSeat) {
        self.wl_seat = Some(seat);
    }
    fn new_capability(&mut self, _conn: &Connection, qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat, capability: Capability) {
        match capability {
            Capability::Keyboard => {
                self.seat_state.get_keyboard(qh, &seat, None).expect("keyboard");
            }
            Capability::Pointer => {
                // handled at startup
            }
            _ => {}
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}
impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(AppState);
delegate_output!(AppState);
delegate_seat!(AppState);
delegate_keyboard!(AppState);
delegate_pointer!(AppState);
delegate_xdg_shell!(AppState);
delegate_xdg_window!(AppState);
delegate_registry!(AppState);
