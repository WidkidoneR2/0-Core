//! faelight-browser v0.1.0 - Graphical Wayland version
//! Same dual-pane layout as TUI, but graphical

use faelight_browser::{Result, storage::BookmarkStore, security::SecurityStatus};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
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
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

mod render;
mod ui;

use render::{draw_gradient_separator, draw_text};
use ui::colors::{ACCENT_BLUE, ACCENT_GREEN, BG_COLOR, TEXT_BRIGHT, TEXT_DIM};

#[allow(dead_code)]
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

// Layout dimensions
const LEFT_PANE_WIDTH: u32 = 576;  // 30% of 1920
const SEPARATOR_WIDTH: u32 = 2;
#[allow(dead_code)]
const RIGHT_PANE_WIDTH: u32 = WINDOW_WIDTH - LEFT_PANE_WIDTH - SEPARATOR_WIDTH;

const TAB_LIST_HEIGHT: u32 = 600;  // 60% of left pane
#[allow(dead_code)]
const BOOKMARK_LIST_HEIGHT: u32 = WINDOW_HEIGHT - TAB_LIST_HEIGHT - 50;

struct Tab {
    title: String,
    url: String,
    security: SecurityStatus,
}

fn main() -> Result<()> {
    println!("🌲 faelight-browser (Wayland)");
    println!("💡 Dual-pane graphical version");
    println!("💡 Press ESC to close");

    let conn = Connection::connect_to_env()
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;

    let (globals, mut event_queue) = registry_queue_init(&conn)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;

    let qh = event_queue.handle();
    let compositor = CompositorState::bind(&globals, &qh)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;
    let layer_shell = LayerShell::bind(&globals, &qh)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;
    let shm = Shm::bind(&globals, &qh)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;
    let seat_state = SeatState::new(&globals, &qh);

    let surface = compositor.create_surface(&qh);
    let layer_surface = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Top,
        Some("faelight-browser"),
        None,
    );

    layer_surface.set_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    layer_surface.set_anchor(Anchor::empty());
    layer_surface.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
    layer_surface.commit();

    let pool = SlotPool::new(WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize * 4, &shm)
        .map_err(|e| faelight_browser::error::BrowserError::Rendering(e.to_string()))?;

    let bookmark_store = BookmarkStore::new().unwrap_or_default();
    
    let tabs = vec![
        Tab {
            title: "Home".to_string(),
            url: "about:home".to_string(),
            security: SecurityStatus::LocalFile,
        },
        Tab {
            title: "Example".to_string(),
            url: "https://example.com".to_string(),
            security: SecurityStatus::Secure,
        },
    ];

    let mut state = BrowserState {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        seat_state,
        shm,
        layer_surface,
        pool,
        bookmark_store,
        tabs,
        active_tab: 0,
        active_bookmark: 0,
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        configured: false,
        running: true,
    };

    event_queue.roundtrip(&mut state)
        .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;

    while state.running {
        event_queue.blocking_dispatch(&mut state)
            .map_err(|e| faelight_browser::error::BrowserError::WaylandConnection(e.to_string()))?;
    }

    Ok(())
}

struct BrowserState {
    registry_state: RegistryState,
    output_state: OutputState,
    seat_state: SeatState,
    shm: Shm,
    layer_surface: LayerSurface,
    pool: SlotPool,
    bookmark_store: BookmarkStore,
    tabs: Vec<Tab>,
    active_tab: usize,
    active_bookmark: usize,
    width: u32,
    height: u32,
    configured: bool,
    running: bool,
}

impl BrowserState {
    fn draw(&mut self, _qh: &QueueHandle<Self>) {
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
                eprintln!("Failed to create buffer: {}", e);
                return;
            }
        };

        // Fill background
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&BG_COLOR);
        }

        // LEFT PANE - Tabs
        let tab_y_start = 20;
        let mut tab_y = tab_y_start;
        
        draw_text(canvas, width, "📑 Tabs", 20, 10, ACCENT_BLUE);
        
        for (i, tab) in self.tabs.iter().enumerate() {
            let color = if i == self.active_tab {
                ACCENT_BLUE
            } else {
                TEXT_DIM
            };
            
            let security_icon = tab.security.icon();
            let text = format!("{} {}", security_icon, tab.title);
            draw_text(canvas, width, &text, 30, tab_y, color);
            tab_y += 30;
        }

        // Separator between tabs and bookmarks
        let bookmark_y_start = TAB_LIST_HEIGHT as i32;
        for y in bookmark_y_start..bookmark_y_start + 2 {
            for x in 0..LEFT_PANE_WIDTH {
                let idx = (y as usize * width as usize + x as usize) * 4;
                if idx + 3 < canvas.len() {
                    canvas[idx..idx + 4].copy_from_slice(&TEXT_DIM);
                }
            }
        }

        // LEFT PANE - Bookmarks
        let mut bm_y = bookmark_y_start + 20;
        draw_text(canvas, width, "🔖 Bookmarks", 20, bookmark_y_start + 10, ACCENT_GREEN);
        
        for (i, bookmark) in self.bookmark_store.list().iter().enumerate() {
            let color = if i == self.active_bookmark {
                ACCENT_GREEN
            } else {
                TEXT_DIM
            };
            
            draw_text(canvas, width, &format!("⭐ {}", bookmark.name), 30, bm_y, color);
            bm_y += 30;
        }

        // VERTICAL SEPARATOR
        let sep_x = LEFT_PANE_WIDTH as i32;
        for _y in 0..height {
            draw_gradient_separator(canvas, width, height, sep_x, TEXT_DIM);
            draw_gradient_separator(canvas, width, height, sep_x + 1, TEXT_DIM);
        }

        // RIGHT PANE - Content
        let content_x = LEFT_PANE_WIDTH as i32 + SEPARATOR_WIDTH as i32 + 20;
        
        if let Some(tab) = self.tabs.get(self.active_tab) {
            // Title with security indicator
            let security_color = match tab.security {
                SecurityStatus::Secure => ACCENT_GREEN,
                SecurityStatus::Insecure => [0x70, 0x87, 0xd0, 0xFF], // RED
                _ => ACCENT_BLUE,
            };
            
            let title_text = format!("{} {} - {}", tab.security.icon(), tab.title, tab.url);
            draw_text(canvas, width, &title_text, content_x, 10, security_color);
            
            // Content
            let mut content_y = 50;
            let content_lines = vec![
                "Welcome to Faelight Browser",
                "",
                "Graphical dual-pane layout",
                "Same style as TUI version",
                "",
                "🔒 Secure by default",
                "📝 Flat-file bookmarks",
                "🎨 Faelight Forest colors",
            ];
            
            for line in content_lines {
                draw_text(canvas, width, line, content_x, content_y, TEXT_BRIGHT);
                content_y += 25;
            }
        }

        self.layer_surface.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer_surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        self.layer_surface.commit();
    }
}

impl CompositorHandler for BrowserState {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        self.draw(qh);
    }
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
}

impl OutputHandler for BrowserState {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for BrowserState {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.running = false;
    }
    fn configure(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &LayerSurface, configure: LayerSurfaceConfigure, _: u32) {
        if configure.new_size.0 > 0 { self.width = configure.new_size.0; }
        if configure.new_size.1 > 0 { self.height = configure.new_size.1; }
        self.configured = true;
        self.draw(qh);
    }
}

impl ShmHandler for BrowserState {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm }
}

impl SeatHandler for BrowserState {
    fn seat_state(&mut self) -> &mut SeatState { &mut self.seat_state }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wayland_client::protocol::wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wayland_client::protocol::wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard {
            self.seat_state.get_keyboard(qh, &seat, None).ok();
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wayland_client::protocol::wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wayland_client::protocol::wl_seat::WlSeat) {}
}

impl KeyboardHandler for BrowserState {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        if event.keysym == Keysym::Escape {
            self.running = false;
        }
    }
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: Modifiers, _: u32) {}
}

impl ProvidesRegistryState for BrowserState {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(BrowserState);
delegate_output!(BrowserState);
delegate_layer!(BrowserState);
delegate_shm!(BrowserState);
delegate_seat!(BrowserState);
delegate_keyboard!(BrowserState);
delegate_registry!(BrowserState);
