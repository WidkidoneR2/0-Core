//! faelight-bar v4.0.0 - Rock-solid Wayland status bar
//! Phase 1: Minimal foundation with clock widget

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
        pointer::{PointerEvent, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::time::Instant;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

mod logger;
mod render;
mod widgets;

use widgets::{
    BatteryWidget, ClockWidget, DateWidget, HealthWidget, LockWidget, NetworkWidget, ProfileWidget,
    RenderContext, SearchWidget, VolumeWidget, VpnWidget, Widget, ZoneWidget,
};

const BAR_HEIGHT: u32 = 64;

fn main() {
    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("faelight-bar v4.0.0");
                std::process::exit(0);
            }
            "--help" | "-h" => {
                println!("faelight-bar v4.0.0 - Faelight Forest Status Bar");
                println!();
                println!("USAGE: faelight-bar [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    -h, --help       Show this help");
                println!("    -v, --version    Show version");
                println!();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                std::process::exit(1);
            }
        }
    }

    // Initialize logger
    logger::init();
    logger::log_info("Starting faelight-bar v4.0.0");

    // Connect to Wayland
    let conn = match Connection::connect_to_env() {
        Ok(c) => c,
        Err(e) => {
            logger::log_error(&format!("Failed to connect to Wayland: {}", e));
            eprintln!("❌ Failed to connect to Wayland: {}", e);
            std::process::exit(1);
        }
    };

    let (globals, mut event_queue) = match registry_queue_init(&conn) {
        Ok(r) => r,
        Err(e) => {
            logger::log_error(&format!("Failed to init registry: {}", e));
            std::process::exit(1);
        }
    };

    let qh = event_queue.handle();

    let compositor = match CompositorState::bind(&globals, &qh) {
        Ok(c) => c,
        Err(e) => {
            logger::log_error(&format!("wl_compositor not available: {}", e));
            std::process::exit(1);
        }
    };

    let layer_shell = match LayerShell::bind(&globals, &qh) {
        Ok(l) => l,
        Err(e) => {
            logger::log_error(&format!("layer shell not available: {}", e));
            std::process::exit(1);
        }
    };

    let shm = match Shm::bind(&globals, &qh) {
        Ok(s) => s,
        Err(e) => {
            logger::log_error(&format!("wl_shm not available: {}", e));
            std::process::exit(1);
        }
    };

    let seat_state = SeatState::new(&globals, &qh);
    let surface = compositor.create_surface(&qh);

    let layer_surface =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("faelight-bar"), None);

    layer_surface.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer_surface.set_size(0, BAR_HEIGHT);
    layer_surface.set_exclusive_zone(BAR_HEIGHT as i32);
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer_surface.commit();

    let pool = match SlotPool::new(4096 * 132 * 4, &shm) {
        Ok(p) => p,
        Err(e) => {
            logger::log_error(&format!("Failed to create pool: {}", e));
            std::process::exit(1);
        }
    };

    // Create widgets
    // Create widgets
    let widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(ClockWidget::new()),
        Box::new(VolumeWidget::new()),
        Box::new(VpnWidget::new()),
        Box::new(ProfileWidget::new()),
        Box::new(BatteryWidget::new()),
        Box::new(NetworkWidget::new()),
        Box::new(DateWidget::new()),
        Box::new(LockWidget::new()),
        Box::new(ZoneWidget::new()),
        Box::new(HealthWidget::new()),
        Box::new(SearchWidget::new()),
    ];

    let mut state = BarState {
        registry_state: RegistryState::new(&globals),
        seat_state,
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool,
        layer_surface,
        widgets,
        width: 0,
        height: BAR_HEIGHT,
        configured: false,
        running: true,
        last_update: Instant::now(),
        pointer_position: None,
        click_regions: Vec::new(),
    };

    logger::log_info("Bar initialized, waiting for configure");

    // Wait for initial configure
    event_queue
        .roundtrip(&mut state)
        .expect("Failed initial roundtrip");

    if !state.configured {
        logger::log_error("Never received configure event!");
        std::process::exit(1);
    }

    logger::log_info("Bar configured, entering event loop");

    // Main event loop
    while state.running {
        if let Err(e) = event_queue.blocking_dispatch(&mut state) {
            logger::log_warn(&format!("Event dispatch error: {}", e));
        }
    }

    logger::log_info("Bar shutting down");
}

struct BarState {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    pool: SlotPool,
    layer_surface: LayerSurface,
    widgets: Vec<Box<dyn Widget>>,
    width: u32,
    height: u32,
    configured: bool,
    running: bool,
    last_update: Instant,
    pointer_position: Option<(i32, i32)>,
    click_regions: Vec<(i32, i32, usize)>,
}

impl BarState {
    fn update_widgets(&mut self) {
        for widget in &mut self.widgets {
            if let Err(e) = widget.update() {
                logger::log_warn(&format!("Widget {} update failed: {}", widget.name(), e));
            }
        }
    }

    fn draw(&mut self, qh: &QueueHandle<Self>) {
        if self.width == 0 {
            return;
        }

        // Update widgets every second
        if self.last_update.elapsed().as_secs() >= 1 {
            self.update_widgets();
            self.last_update = Instant::now();
        }

        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(e) => {
                logger::log_warn(&format!("Failed to create buffer: {}", e));
                return;
            }
        };

        // Clear background
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&render::BG.to_le_bytes());
        }

        // Render widgets with proper spacing
        let y = 24;
        self.click_regions.clear();

        // Update all widgets
        for widget in &mut self.widgets {
            let _ = widget.update();
        }

        // Profile (teal) - widget[3]
        if let Ok(output) = self.widgets[3].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = 30;
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
            if output.clickable {
                self.click_regions
                    .push((x, output.text.len() as i32 * 8, 3));
            }
        }

        // VPN - widget[2]
        if let Ok(output) = self.widgets[2].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = (width as i32) - 555; // VPN
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
            if output.clickable {
                self.click_regions
                    .push((x, output.text.len() as i32 * 8, 2));
            }
        }

        // Volume - widget[1]
        if let Ok(output) = self.widgets[1].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = (width as i32) - 440; // Volume
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
            if output.clickable {
                self.click_regions
                    .push((x, output.text.len() as i32 * 8, 1));
            }
        }

        // Clock - widget[0]

        // Battery - widget[4]
        if let Ok(output) = self.widgets[4].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = (width as i32) - 360; // Battery
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
        }

        // Network - widget[5]
        if let Ok(output) = self.widgets[5].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = (width as i32) - 640; // Network
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
        }

        // Lock - widget[7]
        if let Ok(output) = self.widgets[7].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = 400;
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
            if output.clickable {
                self.click_regions
                    .push((x, output.text.len() as i32 * 8, 7));
            }
        }

        // Zone - widget[8]
        if let Ok(output) = self.widgets[8].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = 250;
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
        }

        // Health - widget[9]
        if let Ok(output) = self.widgets[9].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = 630;
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
        }

        // Search - widget[10]
        if let Ok(output) = self.widgets[10].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = 800;
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
            if output.clickable {
                self.click_regions
                    .push((x, output.text.len() as i32 * 8, 10));
            }
        }
        if let Ok(output) = self.widgets[0].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let clock_x = (width as i32) - 130;
            render::draw_text(canvas, stride, clock_x, y, &output.text, output.color);
        }

        // Date - widget[6] (next to clock)
        if let Ok(output) = self.widgets[6].render(&RenderContext {
            width,
            height,
            x_offset: 0,
        }) {
            let x = (width as i32) - 240;
            render::draw_text(canvas, stride, x, y, &output.text, output.color);
        }

        self.layer_surface
            .wl_surface()
            .attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer_surface
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        self.layer_surface
            .wl_surface()
            .frame(qh, self.layer_surface.wl_surface().clone());
        self.layer_surface.commit();
    }

    fn handle_click(&mut self, x: i32, _y: i32, qh: &QueueHandle<Self>) {
        logger::log_info(&format!("Click at x={}", x));

        for (widget_x, widget_width, widget_idx) in &self.click_regions {
            if x >= *widget_x && x < (*widget_x + *widget_width) {
                logger::log_info(&format!("Clicked widget {}", widget_idx));
                if let Some(widget) = self.widgets.get_mut(*widget_idx) {
                    if let Err(e) = widget.on_click() {
                        logger::log_error(&format!("Click error: {}", e));
                    } else {
                        logger::log_info("Widget clicked successfully!");
                        self.draw(qh);
                    }
                }
                break;
            }
        }
    }
}

impl CompositorHandler for BarState {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        // Only redraw if we need to update (every second)
        if self.last_update.elapsed().as_secs() >= 1 {
            self.draw(qh);
        } else {
            // Request next frame for smooth updates
            self.layer_surface
                .wl_surface()
                .frame(qh, self.layer_surface.wl_surface().clone());
            self.layer_surface.commit();
        }
    }

    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for BarState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for BarState {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.running = false;
        logger::log_info("Layer surface closed");
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }
        self.configured = true;
        logger::log_info(&format!("Configured: {}x{}", self.width, self.height));
        self.draw(qh);
    }
}

impl ShmHandler for BarState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SeatHandler for BarState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            logger::log_info("Pointer capability available - creating pointer");
            self.seat_state.get_pointer(qh, &seat).ok();
        }
    }
    fn remove_capability(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _: Capability,
    ) {
    }
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for BarState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _: &[Keysym],
    ) {
    }
    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: &wl_surface::WlSurface,
        _: u32,
    ) {
    }
    fn press_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }
    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: KeyEvent,
    ) {
    }
    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _: Modifiers,
        _: u32,
    ) {
    }
}

impl PointerHandler for BarState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use smithay_client_toolkit::seat::pointer::PointerEventKind;

        for event in events {
            match event.kind {
                PointerEventKind::Enter { .. } => {}
                PointerEventKind::Leave { .. } => {
                    self.pointer_position = None;
                }
                PointerEventKind::Motion { .. } => {
                    let (x, y) = event.position;
                    self.pointer_position = Some((x as i32, y as i32));
                }
                PointerEventKind::Press { button, .. } => {
                    if button == 272 {
                        if let Some((x, y)) = self.pointer_position {
                            self.handle_click(x, y, qh);
                        }
                    }
                }
                PointerEventKind::Release { .. } => {}
                _ => {}
            }
        }
    }
}

impl ProvidesRegistryState for BarState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(BarState);
delegate_output!(BarState);
delegate_layer!(BarState);
delegate_shm!(BarState);
delegate_registry!(BarState);
delegate_seat!(BarState);
delegate_keyboard!(BarState);
delegate_pointer!(BarState);
