// FaelightCompositor — central state container
// Modeled on smallvil's Smallvil struct, extended with forest integration.
//
// Pattern (repeated for every protocol):
//   1. Field in this struct
//   2. Initialize in new()
//   3. impl XxxHandler for FaelightCompositor in handlers/
//   4. delegate_xxx!(FaelightCompositor)

use std::{ffi::OsString, sync::Arc};

use smithay::{
    desktop::{PopupManager, Space, Window},
    input::{Seat, SeatState},
    reexports::{
        calloop::{generic::Generic, EventLoop, Interest, LoopSignal, Mode, PostAction},
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Display, DisplayHandle,
        },
    },
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        output::OutputManagerState,
        selection::data_device::DataDeviceState,
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

/// Central state container for faelight-compositor.
/// Every Smithay callback receives &mut FaelightCompositor.
pub struct FaelightCompositor {
    pub start_time: std::time::Instant,
    pub socket_name: OsString,
    pub display_handle: DisplayHandle,

    /// The 2D plane where windows live.
    pub space: Space<Window>,
    pub loop_signal: LoopSignal,

    // ── Smithay protocol state ──────────────────────────────
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<Self>,
    pub data_device_state: DataDeviceState,
    pub popups: PopupManager,
    pub seat: Seat<Self>,

    // ── Forest integration ──────────────────────────────────
    // These three fields are what make faelight-compositor
    // a participant in the forest, not just substrate.
    //
    // TODO Phase 2: wire these to faelight-daemon IPC
    pub event_log: Vec<CompositorEvent>,   // ledger events (→ state.db)
    pub health: CompositorHealth,          // → doctor check
}

/// Events emitted by the compositor into the forest ledger.
/// Maps to existing state.db schema: (domain, action, payload, timestamp)
#[derive(Debug, Clone)]
pub struct CompositorEvent {
    pub domain: &'static str,   // always "compositor"
    pub action: &'static str,   // "window.open", "window.focus", "window.close", "workspace.switch"
    pub payload: String,        // JSON: app_id, title, workspace
    pub timestamp: u64,
}

/// Health state reported to doctor.
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

        // Initialize Wayland protocols — same order as smallvil
        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let data_device_state = DataDeviceState::new::<Self>(&dh);
        let popups = PopupManager::default();

        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&dh, "faelight");
        seat.add_keyboard(Default::default(), 200, 25).unwrap();
        seat.add_pointer();

        let space = Space::default();

        let socket_name = Self::init_wayland_listener(display, event_loop);
        let loop_signal = event_loop.get_signal();

        tracing::info!("FaelightCompositor state initialized");

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
            popups,
            seat,
            event_log: Vec::new(),
            health: CompositorHealth::default(),
        }
    }

    /// Emit an event into the forest ledger.
    /// Phase 1: stored in memory.
    /// Phase 2: written to state.db via faelight-daemon IPC.
    pub fn emit(&mut self, action: &'static str, payload: String) {
        let event = CompositorEvent {
            domain: "compositor",
            action,
            payload,
            timestamp: self.start_time.elapsed().as_secs(),
        };
        tracing::info!(domain = "compositor", action, "event emitted");
        self.event_log.push(event);
        self.health.uptime_secs = self.start_time.elapsed().as_secs();
    }

    fn init_wayland_listener(
        display: Display<Self>,
        event_loop: &mut EventLoop<Self>,
    ) -> OsString {
        let listening_socket = ListeningSocketSource::new_auto().unwrap();
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
                    }
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

        socket_name
    }
}

/// Per-client state required by Smithay.
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
