//! faelight-term v3 -- Phase 2: text rendering
//! Goal: "Hello 🌲 World" renders with cosmic-text + glyphon
//! Emoji takes exactly 2 cells. Box drawing renders correctly.

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
use std::ptr::NonNull;

fn main() {
    env_logger::init();
    log::info!("faelight-term v3 -- Phase 2: text rendering");

    eprintln!("[v3] connecting to wayland...");
    let conn = Connection::connect_to_env().expect("wayland connection");
    eprintln!("[v3] wayland connected");
    let (globals, mut event_queue) = registry_queue_init::<AppState>(&conn).expect("registry");
    eprintln!("[v3] registry initialized");
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
    window.set_title("faelight-term v3 -- Phase 2: text");
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

    eprintln!("[v3] roundtrip 1...");
    event_queue.roundtrip(&mut state).expect("roundtrip");
    eprintln!("[v3] roundtrip 2...");
    event_queue.roundtrip(&mut state).expect("roundtrip 2");
    eprintln!("[v3] configured={}", state.configured);

    // Initial render to break Wayland deadlock -- Niri waits for buffer commit
    eprintln!("[v3] initial render...");
    if state.configured {
        if let Some(ref mut gpu) = state.gpu {
            gpu.render();
        }
    }
    event_queue.flush().expect("flush");
    eprintln!("[v3] entering event loop");
    let mut frame = 0u32;
    while !state.exit {
        frame += 1;
        eprintln!("[v3] loop frame {}", frame);
        match event_queue.blocking_dispatch(&mut state) {
            Ok(n) => { eprintln!("[v3] dispatch ok n={}", n); }
            Err(e) => { eprintln!("[v3] dispatch error: {:?}", e); break; }
        }
        if state.configured {
            if let Some(ref mut gpu) = state.gpu {
                gpu.render();
            }
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
}

struct FaelightWindow {
    window_ptr: *mut std::ffi::c_void,
    display_ptr: *mut std::ffi::c_void,
}
unsafe impl Send for FaelightWindow {}
unsafe impl Sync for FaelightWindow {}

impl HasWindowHandle for FaelightWindow {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let handle = WaylandWindowHandle::new(NonNull::new(self.window_ptr).unwrap());
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(RawWindowHandle::Wayland(handle)) })
    }
}
impl HasDisplayHandle for FaelightWindow {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let handle = WaylandDisplayHandle::new(NonNull::new(self.display_ptr).unwrap());
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(handle)) })
    }
}

struct GpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    // Text rendering
    font_system: FontSystem,
    swash_cache: SwashCache,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    text_buffer: Buffer,
}

impl GpuState {
    fn new(window: FaelightWindow, width: u32, height: u32) -> Self {
        // --- wgpu setup ---
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
        log::info!("wgpu: {}x{} {:?} on {:?}", width, height, format, adapter.get_info().name);

        // --- glyphon + cosmic-text setup ---
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let mut text_atlas = TextAtlas::new(&device, &queue, &cache, format);
        let text_renderer = TextRenderer::new(
            &mut text_atlas, &device,
            wgpu::MultisampleState::default(), None
        );

        // --- text buffer: the Phase 2 test string ---
        let font_size = 20.0;
        let line_height = 24.0;
        let mut text_buffer = Buffer::new(&mut font_system, Metrics::new(font_size, line_height));
        text_buffer.set_size(&mut font_system, Some(width as f32), Some(height as f32));

        let attrs = Attrs::new().family(Family::Monospace);
        text_buffer.set_text(
            &mut font_system,
            "Hello \u{1f332} World\nfaelight-term v3\nwgpu + cosmic-text + glyphon\n\u{250c}\u{2500}\u{2500}\u{2500}\u{2510}\n\u{2502} forest \u{2502}\n\u{2514}\u{2500}\u{2500}\u{2500}\u{2518}",
            attrs,
            Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

        log::info!("glyphon text buffer ready");

        let _ = viewport;

        Self {
            device, queue, surface, config,
            font_system, swash_cache, text_atlas, text_renderer,
            text_buffer,
        }
    }

    fn render(&mut self) {
        eprintln!("[v3] render called");
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(e) => { log::warn!("surface error: {:?}", e); return; }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Prepare text for rendering
        let cache = glyphon::Cache::new(&self.device);
        let mut viewport = glyphon::Viewport::new(&self.device, &cache);
        viewport.update(&self.queue, Resolution {
            width: self.config.width,
            height: self.config.height,
        });

        self.text_renderer.prepare(
            &self.device,
            &self.queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &viewport,
            [TextArea {
                buffer: &self.text_buffer,
                left: 20.0,
                top: 20.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: self.config.width as i32,
                    bottom: self.config.height as i32,
                },
                default_color: GlyphonColor::rgb(0xd7, 0xe0, 0xda), // Faelight foreground
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        ).expect("text prepare");

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
                            r: 0.067, g: 0.078, b: 0.059, a: 1.0, // #11140f forest green
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.text_renderer.render(&self.text_atlas, &viewport, &mut pass).expect("text render");
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.text_atlas.trim();
    }
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
            log::info!("GPU + text initialized: {}x{}", self.width, self.height);
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
