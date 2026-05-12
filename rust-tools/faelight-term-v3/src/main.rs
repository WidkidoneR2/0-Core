//! faelight-term v3 -- Phase 1 spike
//! Goal: wgpu surface renders in a Wayland window via sctk
//! No text, no PTY -- just proof the GPU pipeline works on Wayland

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
use std::ptr::NonNull;

fn main() {
    env_logger::init();
    log::info!("faelight-term v3 -- Phase 1 spike starting");

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
        wgpu_state: None,
        display_ptr,
    };

    event_queue.roundtrip(&mut state).expect("roundtrip");
    event_queue.roundtrip(&mut state).expect("roundtrip 2");

    log::info!("window configured: {}", state.configured);

    while !state.exit {
        event_queue.blocking_dispatch(&mut state).expect("dispatch");
        if state.configured {
            if let Some(ref mut wgpu) = state.wgpu_state {
                wgpu.render();
            }
        }
    }

    log::info!("faelight-term v3 spike complete");
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
    wgpu_state: Option<WgpuState>,
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
        let handle = WaylandWindowHandle::new(
            NonNull::new(self.window_ptr).expect("non-null surface")
        );
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(RawWindowHandle::Wayland(handle)) })
    }
}

impl HasDisplayHandle for FaelightWindow {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let handle = WaylandDisplayHandle::new(
            NonNull::new(self.display_ptr).expect("non-null display")
        );
        Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(RawDisplayHandle::Wayland(handle)) })
    }
}

#[allow(dead_code)]
struct WgpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl WgpuState {
    fn new(window: FaelightWindow, width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).expect("wgpu surface");

        let adapter = futures::executor::block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
        ).expect("adapter");

        log::info!("wgpu adapter: {:?}", adapter.get_info());

        let (device, queue) = futures::executor::block_on(
            adapter.request_device(&wgpu::DeviceDescriptor::default(), None)
        ).expect("device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

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

        log::info!("wgpu surface configured: {}x{} {:?}", width, height, format);

        Self { device, queue, surface, config }
    }

    fn render(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(e) => {
                log::warn!("surface error: {:?}", e);
                return;
            }
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("frame") }
        );
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            // Faelight Forest green: #11140f
                            r: 0.067,
                            g: 0.078,
                            b: 0.059,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl CompositorHandler for AppState {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface, _: u32) {}
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
            let display_ptr = self.display_ptr;
            let fw = FaelightWindow { window_ptr, display_ptr };
            self.wgpu_state = Some(WgpuState::new(fw, self.width, self.height));
            log::info!("wgpu initialized -- {}x{}", self.width, self.height);
        }
    }
}

impl SeatHandler for AppState {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, _: &QueueHandle<Self>,
        _: wl_seat::WlSeat, _: Capability) {}
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
delegate_xdg_shell!(AppState);
delegate_xdg_window!(AppState);
delegate_registry!(AppState);
