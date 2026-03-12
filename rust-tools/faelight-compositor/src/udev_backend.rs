// faelight-compositor — DRM/udev backend
// INT-109 Phase 2: Real hardware, replaces winit

use smithay::{
    backend::{
        input::InputEvent,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        session::{libseat::LibSeatSession, Event as SessionEvent, Session},
        udev::{UdevBackend, UdevEvent},
    },
    reexports::{
        calloop::EventLoop,
        input::Libinput,
    },
};

use crate::FaelightCompositor;

pub fn init_drm(
    event_loop: &mut EventLoop<FaelightCompositor>,
    state: &mut FaelightCompositor,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing DRM/udev backend");

    // ── 1. LibSeat session ────────────────────────────────────────────
    let (session, notifier) = LibSeatSession::new()
        .map_err(|e| format!("Failed to create libseat session: {}", e))?;

    tracing::info!(seat = %session.seat(), "LibSeat session created");

    // ── 2. UdevBackend ────────────────────────────────────────────────
    let udev_backend = UdevBackend::new(session.seat())
        .map_err(|e| format!("Failed to initialize udev backend: {:?}", e))?;

    // ── 3. LibInput ───────────────────────────────────────────────────
    let session_interface: LibinputSessionInterface<LibSeatSession> = session.clone().into();
    let mut libinput_context = Libinput::new_with_udev(session_interface);
    libinput_context
        .udev_assign_seat(&session.seat())
        .map_err(|_| "Failed to assign libinput seat")?;
    let libinput_backend = LibinputInputBackend::new(libinput_context);

    event_loop
        .handle()
        .insert_source(libinput_backend, move |event, _, state| {
            state.process_input_event(event);
        })?;

    // ── 4. Session notifier ───────────────────────────────────────────
    event_loop
        .handle()
        .insert_source(notifier, move |event, _, _state| {
            match event {
                SessionEvent::PauseSession => {
                    tracing::info!("Session paused — VT switch");
                }
                SessionEvent::ActivateSession => {
                    tracing::info!("Session activated");
                }
            }
        })?;

    // ── 5. Udev events ────────────────────────────────────────────────
    event_loop
        .handle()
        .insert_source(udev_backend, move |event, _, state| {
            match event {
                UdevEvent::Added { device_id, path } => {
                    tracing::info!(path = ?path, "DRM device added");
                    let payload = format!(
                        r#"{{"event":"device.added","path":"{}"}}"#,
                        path.to_string_lossy()
                    );
                    state.emit("compositor.drm", payload);
                }
                UdevEvent::Changed { device_id } => {
                    tracing::debug!(?device_id, "DRM device changed");
                }
                UdevEvent::Removed { device_id } => {
                    tracing::info!(?device_id, "DRM device removed");
                }
            }
        })?;

    state.emit(
        "compositor.drm",
        format!(r#"{{"event":"backend.init","seat":"{}"}}"#, session.seat()),
    );

    tracing::info!("DRM/udev backend initialized — faelight-compositor on real hardware");
    Ok(())
}
