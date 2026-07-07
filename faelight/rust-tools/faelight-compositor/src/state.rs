use std::{ffi::OsString, sync::Arc};

use rusqlite::Connection;
use smithay::{
    desktop::{layer_map_for_output, PopupManager, Space, Window},
    input::{Seat, SeatState},
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, Mode, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_output::WlOutput,
            Display, DisplayHandle,
        },
    },
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        cursor_shape::CursorShapeManagerState,
        fractional_scale::FractionalScaleManagerState,
        output::OutputManagerState,
        selection::{data_device::DataDeviceState, primary_selection::PrimarySelectionState},
        shell::wlr_layer::{Layer, LayerSurface, WlrLayerShellHandler, WlrLayerShellState},
        shell::xdg::{decoration::XdgDecorationState, XdgShellState},
        shm::ShmState,
        socket::ListeningSocketSource,
        xdg_activation::XdgActivationState,
    },
};

pub struct FaelightCompositor {
    pub session: Option<smithay::backend::session::libseat::LibSeatSession>,
    pub drm_device: Option<smithay::backend::drm::DrmDevice>,
    pub gbm_pipeline: Option<crate::drm_renderer::GbmRenderPipeline>,
    pub start_time: std::time::Instant,
    pub socket_name: OsString,
    pub display_handle: DisplayHandle,
    pub space: Space<Window>,
    pub loop_signal: LoopSignal,

    // Smithay protocol state
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    #[allow(dead_code)]
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: DataDeviceState,
    pub primary_selection_state: PrimarySelectionState,
    #[allow(dead_code)]
    pub xdg_decoration_state: XdgDecorationState,
    pub xdg_activation_state: XdgActivationState,
    #[allow(dead_code)]
    pub cursor_shape_state: CursorShapeManagerState,
    #[allow(dead_code)]
    pub fractional_scale_state: FractionalScaleManagerState,
    pub popups: PopupManager,
    pub seat: Seat<Self>,

    // layer shell for faelight-bar and faelight-notify
    pub layer_shell_state: WlrLayerShellState,
    pub layer_surfaces: Vec<smithay::wayland::shell::wlr_layer::LayerSurface>,
    // dmabuf support for wgpu clients
    pub dmabuf_state: Option<smithay::wayland::dmabuf::DmabufState>,
    pub dmabuf_global: Option<smithay::wayland::dmabuf::DmabufGlobal>,
    // Forest integration
    pub db: Option<Connection>,   // → state.db
    pub health: CompositorHealth, // → doctor
}

#[derive(Debug, Default)]
pub struct CompositorHealth {
    pub windows_open: usize,
    pub active_workspace: u8,
    pub uptime_secs: u64,
}

impl FaelightCompositor {
    pub fn new(event_loop: &mut EventLoop<Self>, display: Display<Self>) -> Self {
        let start_time = std::time::Instant::now();
        let dh = display.handle();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let data_device_state = DataDeviceState::new::<Self>(&dh);
        let primary_selection_state = PrimarySelectionState::new::<Self>(&dh);
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&dh);
        let xdg_activation_state = XdgActivationState::new::<Self>(&dh);
        let cursor_shape_state = CursorShapeManagerState::new::<Self>(&dh);
        let fractional_scale_state = FractionalScaleManagerState::new::<Self>(&dh);
        let popups = PopupManager::default();

        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&dh, "faelight");
        seat.add_keyboard(Default::default(), 200, 25).unwrap();
        seat.add_pointer();

        let space = Space::default();
        let socket_name = Self::init_wayland_listener(display, event_loop);
        let loop_signal = event_loop.get_signal();

        // Open state.db for event emission
        let db = Self::open_db();
        let layer_shell_state = WlrLayerShellState::new::<FaelightCompositor>(&dh);

        tracing::info!("FaelightCompositor initialized");

        Self {
            start_time,
            socket_name,
            display_handle: dh,
            space,
            loop_signal,
            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            data_device_state,
            primary_selection_state,
            xdg_decoration_state,
            xdg_activation_state,
            cursor_shape_state,
            fractional_scale_state,
            popups,
            seat,
            db,
            health: CompositorHealth::default(),
            layer_shell_state,
            layer_surfaces: Vec::new(),
            dmabuf_state: None,
            dmabuf_global: None,
            session: None,
            drm_device: None,
            gbm_pipeline: None,
        }
    }

    fn open_db() -> Option<Connection> {
        let db_path = faelight_core::paths::state_db();

        match Connection::open(&db_path) {
            Ok(conn) => {
                tracing::info!(path = ?db_path, "Connected to state.db");
                Some(conn)
            }
            Err(e) => {
                tracing::warn!("Could not open state.db: {} — events will be lost", e);
                None
            }
        }
    }

    /// Emit an event into the forest ledger (state.db).
    /// Schema: (domain, action, payload, timestamp)
    /// Payload format matches core's EventWriter:
    ///   {"actor":"faelight-compositor","result":"ok","detail":{...}}
    pub fn emit(&mut self, action: &'static str, detail: String) {
        self.health.uptime_secs = self.start_time.elapsed().as_secs();

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let payload = format!(
            r#"{{"actor":"faelight-compositor","result":"ok","detail":{}}}"#,
            detail
        );

        if let Some(db) = &self.db {
            let result = db.execute(
                "INSERT INTO events (domain, action, payload, timestamp) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["compositor", action, payload, ts],
            );
            match result {
                Ok(_) => tracing::info!(action, "event written to state.db"),
                Err(e) => tracing::warn!("Failed to write event: {}", e),
            }
        } else {
            tracing::warn!(action, "state.db not available — event dropped");
        }
    }

    /// Read active intent from /etc/faelight/INTENT (written by faelight-export)
    pub fn active_intent() -> String {
        std::fs::read_to_string("/etc/faelight/INTENT")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "forest".to_string())
    }

    fn init_wayland_listener(display: Display<Self>, event_loop: &mut EventLoop<Self>) -> OsString {
        let listening_socket = ListeningSocketSource::new_auto()
            .expect("Failed to create Wayland socket — is XDG_RUNTIME_DIR set?");
        let socket_name = listening_socket.socket_name().to_os_string();
        let loop_handle = event_loop.handle();

        loop_handle
            .insert_source(listening_socket, move |client_stream, _, state| {
                state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                    .unwrap();
            })
            .expect("Failed to init Wayland socket");

        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, state| {
                    unsafe {
                        display.get_mut().dispatch_clients(state).unwrap();
                        let _ = display.get_mut().flush_clients();
                    }
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

        socket_name
    }
}

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

impl smithay::wayland::dmabuf::DmabufHandler for FaelightCompositor {
    fn dmabuf_state(&mut self) -> &mut smithay::wayland::dmabuf::DmabufState {
        self.dmabuf_state.as_mut().unwrap()
    }
    fn dmabuf_imported(
        &mut self,
        _global: &smithay::wayland::dmabuf::DmabufGlobal,
        _dmabuf: smithay::backend::allocator::dmabuf::Dmabuf,
        notifier: smithay::wayland::dmabuf::ImportNotifier,
    ) {
        let _ = notifier.successful::<FaelightCompositor>();
    }
}
smithay::delegate_dmabuf!(FaelightCompositor);

impl WlrLayerShellHandler for FaelightCompositor {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.layer_shell_state
    }
    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        _output: Option<WlOutput>,
        _layer: Layer,
        namespace: String,
    ) {
        tracing::info!("Layer surface created: {}", namespace);
        // Map the layer surface to the first output
        let output = self.space.outputs().next().cloned();
        if let Some(output) = output {
            let desktop_surface =
                smithay::desktop::LayerSurface::new(surface.clone(), namespace.clone());
            let mut map = layer_map_for_output(&output);
            let _ = map.map_layer(&desktop_surface);
            map.arrange();
            // Send initial configure so bar knows its size
            drop(map);
        }
        // Send configure to the layer surface
        surface.send_configure();
        self.layer_surfaces.push(surface);
    }
}
smithay::delegate_layer_shell!(FaelightCompositor);
