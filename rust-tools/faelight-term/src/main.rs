mod pty;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_output, delegate_registry, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        xdg::{
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell, XdgSurface,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 BUFFER TEST");
    
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();
    
    let mut app = App {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        compositor_state: CompositorState::bind(&globals, &qh)?,
        shm_state: Shm::bind(&globals, &qh)?,
        xdg_shell_state: XdgShell::bind(&globals, &qh)?,
        exit: false,
        window: None,
        pool: None,
    };
    
    let surface = app.compositor_state.create_surface(&qh);
    let window = app.xdg_shell_state.create_window(surface, WindowDecorations::ServerDefault, &qh);
    
    window.set_title("BUFFER TEST");
    window.commit();
    
    app.window = Some(window);
    
    event_queue.blocking_dispatch(&mut app)?;
    
    println!("✅ Entering loop");
    
    loop {
        if app.exit {
            break;
        }
        event_queue.dispatch_pending(&mut app)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    Ok(())
}

struct App {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor_state: CompositorState,
    shm_state: Shm,
    xdg_shell_state: XdgShell,
    exit: bool,
    window: Option<Window>,
    pool: Option<SlotPool>,
}

impl CompositorHandler for App {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState { &mut self.output_state }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl WindowHandler for App {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.exit = true;
    }
    
    fn configure(&mut self, conn: &Connection, _qh: &QueueHandle<Self>, window: &Window, configure: WindowConfigure, serial: u32) {
        let width = configure.new_size.0.map(|w| w.get()).unwrap_or(800);
        let height = configure.new_size.1.map(|h| h.get()).unwrap_or(600);
        
        println!("⚙️  Configure {}x{}", width, height);
        
        // CREATE BUFFER POOL
        let pool = SlotPool::new((width * height * 4) as usize, &self.shm_state).unwrap();
        self.pool = Some(pool);
        
        // ACK
        window.xdg_surface().ack_configure(serial);
        
        // DRAW RED PIXELS
        let pool = self.pool.as_mut().unwrap();
        let (buffer, canvas) = pool.create_buffer(
            width as i32,
            height as i32,
            width as i32 * 4,
            wl_shm::Format::Argb8888,
        ).unwrap();
        
        // BRIGHT RED
        for pixel in canvas.chunks_exact_mut(4) {
            pixel[0] = 0x00; // B
            pixel[1] = 0x00; // G
            pixel[2] = 0xFF; // R
            pixel[3] = 0xFF; // A
        }
        
        // ATTACH BUFFER!
        window.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        window.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        
        // COMMIT WITH BUFFER
        window.wl_surface().commit();
        conn.flush().unwrap();
        
        println!("🎨 RED BUFFER ATTACHED AND COMMITTED!");
    }
}

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm { &mut self.shm_state }
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState { &mut self.registry_state }
    registry_handlers![OutputState];
}

delegate_compositor!(App);
delegate_output!(App);
delegate_shm!(App);
delegate_xdg_shell!(App);
delegate_xdg_window!(App);
delegate_registry!(App);
