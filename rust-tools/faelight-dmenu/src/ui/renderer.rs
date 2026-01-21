//! Wayland renderer for dmenu

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
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler,
            LayerSurface, LayerSurfaceConfigure,
        },
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_seat, wl_shm, wl_surface},
    Connection, EventQueue, QueueHandle,
};
use fontdue::{Font, FontSettings};

use super::SharedState;

const MAX_WIDTH: u32 = 450;
const HEIGHT: u32 = 50;
const BORDER_RADIUS: usize = 12;

// 🌲 EXACT Faelight Bar Colors!
const BG_COLOR: u32 = 0xDD0F1411;      // Dark forest green (transparent)
const TEXT_COLOR: u32 = 0xFFD7E0DA;    // Light sage
const ACCENT_COLOR: u32 = 0xFF6BE3A3;  // Bright mint green
const SELECTED_COLOR: u32 = 0xFFF5C177; // Peachy amber

pub struct DmenuApp {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm_state: Shm,
    layer_shell: LayerShell,

    pub exit: bool,
    surface: Option<wl_surface::WlSurface>,
    layer: Option<LayerSurface>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pool: Option<SlotPool>,
    
    font: Font,
    state: SharedState,
}

impl DmenuApp {
    pub fn new(state: SharedState) -> (Self, Connection, EventQueue<Self>) {
        let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
        let (globals, event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
        let qh = event_queue.handle();

        let registry_state = RegistryState::new(&globals);
        let seat_state = SeatState::new(&globals, &qh);
        let output_state = OutputState::new(&globals, &qh);
        let compositor_state = CompositorState::bind(&globals, &qh)
            .expect("wl_compositor not available");
        let shm_state = Shm::bind(&globals, &qh).expect("wl_shm not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell not available");

        // Load font
        let font_data = include_bytes!("/usr/share/fonts/liberation/LiberationMono-Regular.ttf");
        let font = Font::from_bytes(font_data as &[u8], FontSettings::default())
            .expect("Failed to load font");

        let app = Self {
            registry_state,
            seat_state,
            output_state,
            compositor_state,
            shm_state,
            layer_shell,
            exit: false,
            surface: None,
            layer: None,
            keyboard: None,
            pool: None,
            font,
            state,
        };

        (app, conn, event_queue)
    }

    pub fn init_surface(&mut self, qh: &QueueHandle<Self>) {
        let surface = self.compositor_state.create_surface(qh);
        
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface.clone(),
            Layer::Overlay,
            Some("faelight-dmenu"),
            None,
        );

        layer.set_anchor(Anchor::empty());
        layer.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        layer.set_size(MAX_WIDTH, HEIGHT);
        
        surface.commit();

        self.surface = Some(surface);
        self.layer = Some(layer);
    }

    pub fn draw(&mut self, _qh: &QueueHandle<Self>) {
        if self.surface.is_none() {
            return;
        }

        let pool = self.pool.get_or_insert_with(|| {
            SlotPool::new(MAX_WIDTH as usize * HEIGHT as usize * 4, &self.shm_state)
                .expect("Failed to create pool")
        });

        let (buffer, canvas) = pool
            .create_buffer(
                MAX_WIDTH as i32,
                HEIGHT as i32,
                MAX_WIDTH as i32 * 4,
                wl_shm::Format::Argb8888,
            )
            .expect("Failed to create buffer");

        // Clear to transparent
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&0x00000000u32.to_le_bytes());
        }

        // Draw rounded rectangle background
        draw_rounded_rect(canvas, 0, 0, MAX_WIDTH as usize, HEIGHT as usize, BORDER_RADIUS, BG_COLOR);

        // Get state snapshot
        let (query, filtered, selected) = {
            let state = self.state.lock().unwrap();
            (state.query.clone(), state.filtered.clone(), state.selected)
        };

        // Draw prompt and query
        let query_text = format!("> {}", query);
        draw_text_simple(&self.font, canvas, 15, 10, &query_text, ACCENT_COLOR);

        // Draw selected item
        if !filtered.is_empty() {
            let item = filtered.get(selected).unwrap();
            let count_text = format!("[{}/{}]", selected + 1, filtered.len());
            
            draw_text_simple(&self.font, canvas, 15, 28, &count_text, TEXT_COLOR);
            draw_text_simple(&self.font, canvas, 80, 28, item, SELECTED_COLOR);
        } else if !query.is_empty() {
            draw_text_simple(&self.font, canvas, 15, 28, "No matches", TEXT_COLOR);
        }

        // Attach buffer and commit
        if let Some(surface) = &self.surface {
            buffer.attach_to(surface).expect("Failed to attach buffer");
            surface.damage_buffer(0, 0, MAX_WIDTH as i32, HEIGHT as i32);
            surface.commit();
        }
    }
}

// Draw rounded rectangle
fn draw_rounded_rect(canvas: &mut [u8], x: usize, y: usize, width: usize, height: usize, radius: usize, color: u32) {
    for py in 0..height {
        for px in 0..width {
            let in_rect = if px < radius && py < radius {
                let dx = radius as i32 - px as i32;
                let dy = radius as i32 - py as i32;
                (dx * dx + dy * dy) <= (radius * radius) as i32
            } else if px >= width - radius && py < radius {
                let dx = px as i32 - (width - radius - 1) as i32;
                let dy = radius as i32 - py as i32;
                (dx * dx + dy * dy) <= (radius * radius) as i32
            } else if px < radius && py >= height - radius {
                let dx = radius as i32 - px as i32;
                let dy = py as i32 - (height - radius - 1) as i32;
                (dx * dx + dy * dy) <= (radius * radius) as i32
            } else if px >= width - radius && py >= height - radius {
                let dx = px as i32 - (width - radius - 1) as i32;
                let dy = py as i32 - (height - radius - 1) as i32;
                (dx * dx + dy * dy) <= (radius * radius) as i32
            } else {
                true
            };

            if in_rect {
                let screen_x = x + px;
                let screen_y = y + py;
                if screen_x < MAX_WIDTH as usize && screen_y < HEIGHT as usize {
                    let offset = (screen_y * MAX_WIDTH as usize + screen_x) * 4;
                    if offset + 3 < canvas.len() {
                        canvas[offset..offset + 4].copy_from_slice(&color.to_le_bytes());
                    }
                }
            }
        }
    }
}

// Simple text rendering
fn draw_text_simple(font: &Font, canvas: &mut [u8], x: usize, y: usize, text: &str, color: u32) {
    let mut offset_x = x;
    let font_size = 14.0;
    
    for ch in text.chars() {
        let (metrics, bitmap) = font.rasterize(ch, font_size);
        
        if metrics.width == 0 || metrics.height == 0 {
            offset_x += metrics.advance_width as usize;
            continue;
        }
        
        for py in 0..metrics.height {
            for px in 0..metrics.width {
                let bitmap_idx = py * metrics.width + px;
                if bitmap_idx >= bitmap.len() {
                    continue;
                }
                
                let alpha = bitmap[bitmap_idx];
                if alpha < 10 {
                    continue;
                }
                
                let screen_x = offset_x + px;
                let screen_y = y + py;
                
                if screen_x < MAX_WIDTH as usize && screen_y < HEIGHT as usize {
                    let pixel_offset = (screen_y * MAX_WIDTH as usize + screen_x) * 4;
                    if pixel_offset + 3 < canvas.len() {
                        let alpha_f = alpha as f32 / 255.0;
                        
                        let r = ((color >> 16) & 0xFF) as u8;
                        let g = ((color >> 8) & 0xFF) as u8;
                        let b = (color & 0xFF) as u8;
                        
                        let bg_b = canvas[pixel_offset];
                        let bg_g = canvas[pixel_offset + 1];
                        let bg_r = canvas[pixel_offset + 2];
                        
                        let final_r = (r as f32 * alpha_f + bg_r as f32 * (1.0 - alpha_f)) as u8;
                        let final_g = (g as f32 * alpha_f + bg_g as f32 * (1.0 - alpha_f)) as u8;
                        let final_b = (b as f32 * alpha_f + bg_b as f32 * (1.0 - alpha_f)) as u8;
                        
                        canvas[pixel_offset] = final_b;
                        canvas[pixel_offset + 1] = final_g;
                        canvas[pixel_offset + 2] = final_r;
                        canvas[pixel_offset + 3] = 0xFF;
                    }
                }
            }
        }
        
        offset_x += metrics.advance_width as usize;
    }
}

// Handler implementations (unchanged)
impl CompositorHandler for DmenuApp {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        self.draw(qh);
    }
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
}

impl OutputHandler for DmenuApp {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for DmenuApp {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &LayerSurface,
        _configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        if let Some(surface) = &self.surface {
            surface.commit();
        }
        self.draw(qh);
    }
}

impl SeatHandler for DmenuApp {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            self.keyboard = self.seat_state.get_keyboard(qh, &seat, None).ok();
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard {
            if let Some(keyboard) = self.keyboard.take() {
                keyboard.release();
            }
        }
    }
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for DmenuApp {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}
    fn press_key(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        let mut state = self.state.lock().unwrap();
        
        match event.keysym {
            Keysym::Escape => {
                state.cancel();
                self.exit = true;
            }
            Keysym::Return => {
                state.confirm();
                self.exit = true;
            }
            Keysym::Down => {
                state.select_next();
                drop(state);
                self.draw(qh);
            }
            Keysym::Up => {
                state.select_prev();
                drop(state);
                self.draw(qh);
            }
            Keysym::BackSpace => {
                if !state.query.is_empty() {
                    state.query.pop();
                    let query_clone = state.query.clone();
                    state.update_query(query_clone);
                    drop(state);
                    self.draw(qh);
                }
            }
            _ => {
                if let Some(ch) = event.utf8.and_then(|s| s.chars().next()) {
                    if ch.is_alphanumeric() || ch.is_whitespace() || ch.is_ascii_punctuation() {
                        state.query.push(ch);
                        let query_clone = state.query.clone();
                        state.update_query(query_clone);
                        drop(state);
                        self.draw(qh);
                    }
                }
            }
        }
    }
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: Modifiers, _: u32) {}
}

impl ShmHandler for DmenuApp {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm_state
    }
}

impl ProvidesRegistryState for DmenuApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

delegate_compositor!(DmenuApp);
delegate_output!(DmenuApp);
delegate_shm!(DmenuApp);
delegate_seat!(DmenuApp);
delegate_keyboard!(DmenuApp);
delegate_layer!(DmenuApp);
delegate_registry!(DmenuApp);
