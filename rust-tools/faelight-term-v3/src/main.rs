//! faelight-term v3 -- Phase 3: PTY + grid
//! Goal: fsh runs in the terminal, output appears correctly
//! alacritty_terminal handles PTY, VTE parsing, and grid state

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_registry, delegate_seat,
    delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState},
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
    protocol::{wl_output, wl_seat, wl_surface},
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
    term::Config as TermConfig,
    tty::{self, Options as TtyOptions},
};
use std::{
    ptr::NonNull,
    sync::Arc,
};

// Cell size in pixels
const CELL_W: f32 = 10.0;
const CELL_H: f32 = 20.0;
const FONT_SIZE: f32 = 16.0;
const LINE_HEIGHT: f32 = 20.0;
const PADDING: f32 = 4.0;

fn main() {
    env_logger::init();
    eprintln!("[v3] Phase 3: PTY + grid starting");

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
    };

    event_queue.roundtrip(&mut state).expect("roundtrip");
    event_queue.roundtrip(&mut state).expect("roundtrip 2");

    eprintln!("[v3] configured={}", state.configured);

    // Initial render to break Wayland deadlock
    if state.configured {
        if let Some(ref mut gpu) = state.gpu {
            gpu.render();
        }
    }
    event_queue.flush().expect("flush");

    eprintln!("[v3] entering event loop");
    while !state.exit {
        // Sync terminal output to GPU text buffer
        if let Some(ref mut gpu) = state.gpu {
            gpu.sync_terminal();
        }

        match event_queue.blocking_dispatch(&mut state) {
            Ok(_) => {}
            Err(e) => { eprintln!("[v3] dispatch error: {:?}", e); break; }
        }

        if state.configured {
            if let Some(ref mut gpu) = state.gpu {
                gpu.render();
            }
        }
    }

    eprintln!("[v3] exiting");
}

// Event listener for alacritty_terminal
#[derive(Clone)]
struct FaelightListener;
impl EventListener for FaelightListener {
    fn send_event(&self, _event: TermEvent) {}
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
    // Text
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    text_buffer: Buffer,
    // Terminal
    term: Arc<FairMutex<Term<FaelightListener>>>,
    _notifier: Notifier,
    cols: usize,
    rows: usize,
}

impl GpuState {
    fn new(window: FaelightWindow, width: u32, height: u32) -> Self {
        // wgpu setup
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
        eprintln!("[v3] wgpu: {}x{} {:?}", width, height, format);

        // glyphon setup
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
        text_buffer.set_text(&mut font_system, "faelight-term v3 starting...", Attrs::new().family(Family::Monospace), Shaping::Advanced);
        text_buffer.shape_until_scroll(&mut font_system, false);

        // Terminal setup
        let cols = ((width as f32 - PADDING * 2.0) / CELL_W) as usize;
        let rows = ((height as f32 - PADDING * 2.0) / CELL_H) as usize;
        let cols = cols.max(10);
        let rows = rows.max(3);

        eprintln!("[v3] terminal grid: {}x{} (cols x rows)", cols, rows);

        let term_config = TermConfig::default();
        let listener = FaelightListener;

        // Spawn PTY with fsh
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        eprintln!("[v3] spawning shell: {}", shell);

        let pty_options = TtyOptions {
            shell: Some(alacritty_terminal::tty::Shell::new(shell, vec![])),
            working_directory: Some(std::path::PathBuf::from(
                std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
            )),
            env: std::collections::HashMap::new(),
            hold: false,
        };

        let window_size = alacritty_terminal::event::WindowSize {
            num_cols: cols as u16,
            num_lines: rows as u16,
            cell_width: CELL_W as u16,
            cell_height: CELL_H as u16,
        };

        let term = Term::new(term_config, &TermDimensions { cols, rows }, listener.clone());
        let term = Arc::new(FairMutex::new(term));

        let pty = tty::new(&pty_options, window_size, 0u64).expect("PTY");
        let event_loop = EventLoop::new(term.clone(), listener, pty, false, false).expect("event loop");
        let notifier = Notifier(event_loop.channel());
        let _event_loop_thread = event_loop.spawn();

        eprintln!("[v3] PTY spawned, shell running");

        Self {
            device, queue, surface, config,
            font_system, swash_cache, text_atlas, text_renderer, text_buffer,
            term,
            _notifier: notifier,
            cols,
            rows,
        }
    }

    fn sync_terminal(&mut self) {
        let term = self.term.lock();
        let grid = term.grid();
        let mut text = String::new();

        for line in 0..self.rows {
            for col in 0..self.cols {
                let point = Point::new(Line(line as i32), Column(col));
                let cell = &grid[point];
                let ch = cell.c;
                text.push(if ch == '\0' { ' ' } else { ch });
            }
            text.push('\n');
        }
        drop(term);

        let attrs = Attrs::new().family(Family::Monospace);
        self.text_buffer.set_text(&mut self.font_system, &text, attrs, Shaping::Advanced);
        self.text_buffer.shape_until_scroll(&mut self.font_system, false);
    }

    fn render(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(e) => { eprintln!("[v3] surface error: {:?}", e); return; }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let cache = glyphon::Cache::new(&self.device);
        let mut viewport = glyphon::Viewport::new(&self.device, &cache);
        viewport.update(&self.queue, Resolution {
            width: self.config.width,
            height: self.config.height,
        });

        if let Err(e) = self.text_renderer.prepare(
            &self.device, &self.queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &viewport,
            [TextArea {
                buffer: &self.text_buffer,
                left: PADDING,
                top: PADDING,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0, top: 0,
                    right: self.config.width as i32,
                    bottom: self.config.height as i32,
                },
                default_color: GlyphonColor::rgb(0xd7, 0xe0, 0xda),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        ) {
            eprintln!("[v3] text prepare error: {:?}", e);
            return;
        }

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("frame") }
        );
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.067, g: 0.078, b: 0.059, a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let Err(e) = self.text_renderer.render(&self.text_atlas, &viewport, &mut pass) {
                eprintln!("[v3] text render error: {:?}", e);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.text_atlas.trim();
    }
}

// Terminal dimensions adapter
struct TermDimensions { cols: usize, rows: usize }
impl Dimensions for TermDimensions {
    fn total_lines(&self) -> usize { self.rows }
    fn screen_lines(&self) -> usize { self.rows }
    fn columns(&self) -> usize { self.cols }
    fn last_column(&self) -> Column { Column(self.cols - 1) }
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
    fn configure(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &Window, configure: WindowConfigure, _: u32) {
        let (w, h) = configure.new_size;
        if let Some(w) = w { self.width = w.get(); }
        if let Some(h) = h { self.height = h.get(); }
        if !self.configured {
            self.configured = true;
            let window_ptr = self.surface.id().as_ptr() as *mut _;
            let fw = FaelightWindow { window_ptr, display_ptr: self.display_ptr };
            self.gpu = Some(GpuState::new(fw, self.width, self.height));
        }
    }
}
impl SeatHandler for AppState {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}
impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(AppState);
delegate_output!(AppState);
delegate_seat!(AppState);
delegate_xdg_shell!(AppState);
delegate_xdg_window!(AppState);
delegate_registry!(AppState);
