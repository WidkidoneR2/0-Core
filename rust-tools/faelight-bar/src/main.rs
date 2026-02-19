//! faelight-bar - Building piece by piece
//! Step 2: Add profile icon only

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::process::Command;
use std::time::{Duration, Instant};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

mod render;
use render::colors;

const BAR_HEIGHT: u32 = 32;
const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

fn get_profile_icon() -> String {
    let output = Command::new("profile").arg("status").output().ok();
    if let Some(out) = output {
        let result = String::from_utf8_lossy(&out.stdout);
        for line in result.lines() {
            if line.contains("Current:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let profile_name = parts[parts.len() - 1];
                    return match profile_name {
                        "default" => "DEF".to_string(),
                        "work" => "WORK".to_string(),
                        "gaming" => "GAME".to_string(),
                        "low-power" => "BATT".to_string(),
                        _ => "DEF".to_string(),
                    };
                }
            }
        }
    }
    "🏠".to_string()
}

fn get_zone() -> String {
    let output = Command::new("faelight-zone").arg("--label").output().ok();
    if let Some(out) = output {
        String::from_utf8_lossy(&out.stdout).trim().to_uppercase()
    } else {
        "WORK".to_string()
    }
}

fn get_lock_status() -> (String, u32) {
    // Check actual immutable bit like the prompt does
    let output = Command::new("lsattr")
        .args(["-d", "/home/christian/0-core"])
        .output()
        .ok();

    if let Some(out) = output {
        let result = String::from_utf8_lossy(&out.stdout);
        if result.contains("----i") {
            ("LOCKED".to_string(), colors::ERROR)
        } else {
            ("UNLOCKED".to_string(), colors::SUCCESS)
        }
    } else {
        ("UNLOCKED".to_string(), colors::SUCCESS)
    }
}

fn get_health() -> (String, u32) {
    let output = Command::new("dot-doctor").output().ok();
    if let Some(out) = output {
        let result = String::from_utf8_lossy(&out.stdout);
        // Parse "Health:   94%" from output
        for line in result.lines() {
            if line.contains("Health:") && line.contains("%") {
                if let Some(pct_str) = line
                    .split('%')
                    .next()
                    .and_then(|s| s.split_whitespace().last())
                {
                    if let Ok(pct) = pct_str.parse::<i32>() {
                        let color = if pct == 100 {
                            colors::SUCCESS // Green for 100%
                        } else if pct >= 80 {
                            colors::WARNING // Yellow for 80-99%
                        } else {
                            colors::ERROR // Red for <80%
                        };
                        return (format!("HP:{}%", pct), color);
                    }
                }
            }
        }
    }
    ("HP:??".to_string(), colors::FG)
}

fn get_wifi() -> (String, u32) {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
        .ok();
    if let Some(out) = output {
        let result = String::from_utf8_lossy(&out.stdout);
        for line in result.lines() {
            if line.starts_with("yes:") {
                return ("WIFI".to_string(), colors::SUCCESS);
            }
        }
    }
    ("WIFI-OFF".to_string(), colors::ERROR)
}

fn get_vpn() -> (String, u32) {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "type,state", "con", "show", "--active"])
        .output()
        .ok();
    if let Some(out) = output {
        let result = String::from_utf8_lossy(&out.stdout);
        if result.contains("vpn:activated") || result.contains("wireguard:activated") {
            return ("VPN".to_string(), colors::SUCCESS);
        }
    }
    ("VPN-OFF".to_string(), colors::ERROR)
}

fn get_volume() -> (String, u32) {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .ok();
    if let Some(out) = output {
        let vol = String::from_utf8_lossy(&out.stdout);
        if vol.contains("MUTED") {
            ("VOL:MUTE".to_string(), colors::ERROR)
        } else if let Some(num) = vol.split_whitespace().nth(1) {
            if let Ok(val) = num.parse::<f32>() {
                (format!("VOL:{}%", (val * 100.0) as i32), colors::SUCCESS)
            } else {
                ("VOL:??".to_string(), colors::FG)
            }
        } else {
            ("VOL:??".to_string(), colors::FG)
        }
    } else {
        ("VOL:??".to_string(), colors::FG)
    }
}

fn get_battery() -> (String, u32) {
    let output = Command::new("cat")
        .arg("/sys/class/power_supply/BAT1/capacity")
        .output()
        .ok();
    if let Some(out) = output {
        let pct = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if let Ok(num) = pct.parse::<i32>() {
            let color = if num > 50 {
                colors::SUCCESS // Green for >50%
            } else if num > 20 {
                colors::WARNING // Yellow for 21-50%
            } else {
                colors::ERROR // Red for <=20%
            };
            return (format!("BAT:{}%", num), color);
        }
    }
    ("BAT:??".to_string(), colors::FG)
}

fn get_time() -> (String, u32) {
    let output = Command::new("date").arg("+%H:%M").output().ok();
    if let Some(out) = output {
        (
            String::from_utf8_lossy(&out.stdout).trim().to_string(),
            colors::ACCENT_BLUE,
        )
    } else {
        ("??:??".to_string(), colors::FG)
    }
}

struct FaelightBar {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layer: LayerSurface,
    pool: SlotPool,
    width: u32,
    first_configure: bool,
    last_update: Instant,
}

impl FaelightBar {
    fn draw(&mut self) {
        eprintln!("🔄 Drawing bar...");
        let width = self.width;
        let stride = (width * 4) as i32;

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            BAR_HEIGHT as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(b) => b,
            Err(_) => return,
        };

        // Dark background
        for pixel in canvas.chunks_exact_mut(4) {
            pixel.copy_from_slice(&colors::BG.to_le_bytes());
        }

        // Get status
        let profile = get_profile_icon();
        let zone = get_zone();
        let (lock_status, lock_color) = get_lock_status();
        let (health, health_color) = get_health();

        let y = 8;
        let mut x = 10;

        // Draw profile
        x = render::text::draw_text(canvas, stride, x, y, &profile, colors::ACCENT);
        x += 20;

        // Draw zone
        let zone_text = format!("Z: {}", zone);
        x = render::text::draw_text(canvas, stride, x, y, &zone_text, colors::ACCENT_BLUE);
        x += 20;

        // Draw lock status (color changes based on locked/unlocked)
        x = render::text::draw_text(canvas, stride, x, y, &lock_status, lock_color);
        x += 20;

        // Draw health (color changes: green=100%, yellow=80-99%, red=<80%)
        render::text::draw_text(canvas, stride, x, y, &health, health_color);

        // RIGHT SIDE - calculate all widths first, then position
        let (wifi, wifi_color) = get_wifi();
        let (vpn, vpn_color) = get_vpn();
        let (volume, vol_color) = get_volume();
        let (battery, bat_color) = get_battery();
        let (time, time_color) = get_time();

        // Calculate widths
        let time_w = render::text::text_width(&time);
        let bat_w = render::text::text_width(&battery);
        let vol_w = render::text::text_width(&volume);
        let vpn_w = render::text::text_width(&vpn);
        let wifi_w = render::text::text_width(&wifi);

        // Position from right to left with 30px gaps
        let time_x = (width as i32) - time_w - 10;
        let bat_x = time_x - 30 - bat_w;
        let vol_x = bat_x - 30 - vol_w;
        let vpn_x = vol_x - 30 - vpn_w;
        let wifi_x = vpn_x - 30 - wifi_w;

        // Draw right side with colors
        render::text::draw_text(canvas, stride, time_x, y, &time, time_color);
        render::text::draw_text(canvas, stride, bat_x, y, &battery, bat_color);
        render::text::draw_text(canvas, stride, vol_x, y, &volume, vol_color);
        render::text::draw_text(canvas, stride, vpn_x, y, &vpn, vpn_color);
        render::text::draw_text(canvas, stride, wifi_x, y, &wifi, wifi_color);

        self.layer
            .wl_surface()
            .attach(Some(buffer.wl_buffer()), 0, 0);
        self.layer.wl_surface().commit();
    }
}

impl LayerShellHandler for FaelightBar {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        std::process::exit(0);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let (width, height) = configure.new_size;
        if width > 0 && height > 0 {
            self.width = width;
        }

        if !self.first_configure {
            self.first_configure = true;
            self.draw();
        }
    }
}

delegate_compositor!(FaelightBar);
delegate_output!(FaelightBar);
delegate_shm!(FaelightBar);
delegate_layer!(FaelightBar);
delegate_registry!(FaelightBar);

impl OutputHandler for FaelightBar {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl ShmHandler for FaelightBar {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl CompositorHandler for FaelightBar {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }
    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }
    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl ProvidesRegistryState for FaelightBar {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

fn main() {
    let conn = Connection::connect_to_env().expect("Failed to connect to Wayland");
    let (globals, mut event_queue) = registry_queue_init(&conn).expect("Failed to init registry");
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("faelight-bar"), None);

    layer.set_anchor(Anchor::TOP | Anchor::LEFT | Anchor::RIGHT);
    layer.set_size(0, BAR_HEIGHT);
    layer.set_exclusive_zone(BAR_HEIGHT as i32);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.commit();

    let pool = SlotPool::new(1920 * BAR_HEIGHT as usize * 4, &shm).expect("Failed to create pool");

    let mut app = FaelightBar {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        layer,
        pool,
        width: 1920,
        first_configure: false,
        last_update: Instant::now(),
    };

    loop {
        // Dispatch Wayland events
        event_queue.flush().unwrap();

        // Redraw every 2 seconds
        if app.first_configure && app.last_update.elapsed() >= UPDATE_INTERVAL {
            app.last_update = Instant::now();
            app.draw();
        }

        // Dispatch pending events without blocking
        let _ = event_queue.dispatch_pending(&mut app);

        // Sleep briefly to avoid busy-waiting
        std::thread::sleep(Duration::from_millis(100));
    }
}
