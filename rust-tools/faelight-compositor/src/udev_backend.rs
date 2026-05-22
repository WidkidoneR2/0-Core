// faelight-compositor — DRM/udev backend
// INT-109 Phase 2: Real hardware, replaces winit

use smithay::{
    backend::{
        allocator::gbm::GbmDevice,
        drm::{DrmDevice, DrmDeviceFd, DrmEvent},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        session::{libseat::LibSeatSession, Event as SessionEvent, Session},
        udev::{UdevBackend, UdevEvent},
    },
    reexports::{
        calloop::EventLoop, drm::control::Device as DrmControlDevice, input::Libinput,
        rustix::fs::OFlags,
    },
};

use crate::FaelightCompositor;
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner, SimpleCrtcMapper};


// ── Probe mode -- enumerate DRM/GBM without taking over display ──────────────

pub fn probe_drm() {
    use smithay::reexports::rustix::fs::OFlags;
    println!("🌲 DRM probe starting...");
    
    // Find DRM devices
    for entry in std::fs::read_dir("/dev/dri").unwrap_or_else(|_| {
        println!("❌ /dev/dri not found");
        std::fs::read_dir("/dev").unwrap()
    }) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.starts_with("card") { continue; }
        
        println!("📍 Found DRM device: {}", path.display());
        
        let open_flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
        let fd = match smithay::backend::session::libseat::LibSeatSession::new() {
            Ok((mut session, _)) => {
                match session.open(&path, open_flags) {
                    Ok(fd) => fd,
                    Err(e) => { println!("❌ Cannot open via libseat: {e}"); continue; }
                }
            }
            Err(_) => {
                // Try direct open (may work as root)
                match std::fs::OpenOptions::new().read(true).write(true).open(&path) {
                    Ok(f) => {
                        use std::os::unix::io::IntoRawFd;
                        
                        println!("⚠️  Opened directly (root mode)");
                        let raw = f.into_raw_fd();
                        unsafe { <smithay::reexports::rustix::fd::OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(raw) }
                    }
                    Err(e) => { println!("❌ Cannot open directly: {e}"); continue; }
                }
            }
        };
        
        let drm_fd = smithay::backend::drm::DrmDeviceFd::new(fd.into());
        match smithay::backend::drm::DrmDevice::new(drm_fd, true) {
            Ok((drm, _)) => {
                println!("✅ DRM device opened");
                // Try GBM
                match smithay::backend::allocator::gbm::GbmDevice::new(drm.device_fd().clone()) {
                    Ok(_gbm) => println!("✅ GBM device created -- rendering possible!"),
                    Err(e) => println!("❌ GBM failed: {e}"),
                }
            }
            Err(e) => println!("❌ DRM device failed: {e}"),
        }
    }
    println!("🌲 DRM probe complete");
}

pub fn init_drm(
    event_loop: &mut EventLoop<FaelightCompositor>,
    state: &mut FaelightCompositor,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing DRM/udev backend");

    // ── 1. LibSeat session ────────────────────────────────────────────
    let (mut session, notifier) =
        LibSeatSession::new().map_err(|e| format!("Failed to create libseat session: {}", e))?;

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
                    tracing::info!("Session paused — VT switch, releasing DRM master");
                    // libseat handles DRM master release automatically
                }
                SessionEvent::ActivateSession => {
                    tracing::info!("Session activated — resuming");
                    // libseat handles DRM master reacquire automatically
                }
            }
        })?;

    // ── 5a. Enumerate existing DRM devices ───────────────────────────
    // udev only fires Added for NEW devices — must enumerate existing ones
    for (_device_id, path) in udev_backend.device_list() {
        let path = path.to_path_buf();
        tracing::info!(path = ?path, "device_list entry — checking if DRM card");
        // Only process card devices, not render nodes
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !filename.starts_with("card") {
            tracing::debug!(path = ?path, "Skipping non-card device");
            continue;
        }
        tracing::info!(path = ?path, "Processing DRM card device");
        // Only process the display card (card2 = AMD 780M iGPU with eDP)
        // card1 = RX 7700S dGPU, no display output in this configuration
        let card_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if card_name == "card1" {
            tracing::info!("Skipping card1 (RX 7700S dGPU — no display connectors)");
            continue;
        }
        tracing::info!(path = ?path, "Existing DRM device found — opening");
        let open_flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
        match session.open(&path, open_flags) {
            Ok(fd) => {
                tracing::info!("DRM device fd opened successfully");
                let drm_fd = DrmDeviceFd::new(fd.into());
                let drm_fd_clone = drm_fd.clone();
                match DrmDevice::new(drm_fd, true) {
                    Ok((mut drm, drm_notifier)) => {
                        // Wire VBlank events into calloop — required for DrmCompositor::frame_submitted
                        let _ = event_loop.handle().insert_source(drm_notifier, move |event, _metadata, state| {
                            match event {
                                DrmEvent::VBlank(_crtc) => {
                                    if let Some(pipeline) = state.gbm_pipeline.as_mut() {
                                        let _ = pipeline.compositor.frame_submitted();
                                    }
                                }
                                DrmEvent::Error(e) => tracing::error!("DRM error: {:?}", e),
                            }
                        });
                        let resources = drm.resource_handles().ok();
                        let connector_count = resources
                            .as_ref()
                            .map(|r| r.connectors().len())
                            .unwrap_or(0);
                        let crtc_count = resources.as_ref().map(|r| r.crtcs().len()).unwrap_or(0);
                        tracing::info!(
                            connectors = connector_count,
                            crtcs = crtc_count,
                            "🎉 DRM device opened — hardware enumerated"
                        );
                        let payload = format!(
                            r#"{{"event":"device.opened","path":"{}","connectors":{},"crtcs":{}}}"#,
                            path.display(),
                            connector_count,
                            crtc_count
                        );
                        state.emit("compositor.drm", payload);

                        // ── Session 4: GBM device + DrmScanner ──────────
                        match GbmDevice::new(drm.device_fd().clone()) {
                            Ok(_gbm) => {
                                tracing::info!("✅ GBM device created successfully");
                                // Initialize full GBM+EGL render pipeline
                                if state.gbm_pipeline.is_none() {
                                    match crate::drm_renderer::init_gbm_pipeline(&mut drm, drm_fd_clone.clone()) {
                                        Ok(pipeline) => {
                                            tracing::info!("🌲 GBM render pipeline initialized -- GPU compositing ready");
                                            state.gbm_pipeline = Some(pipeline);
                                        }
                                        Err(e) => tracing::warn!("GBM pipeline init failed: {e} -- falling back to dumb buffer"),
                                    }
                                }

                                // Use DrmScanner to find connector/CRTC pairs
                                let mut scanner: DrmScanner<SimpleCrtcMapper> = DrmScanner::new();
                                let scan_events = scanner.scan_connectors(&drm).unwrap_or_default();
                                for event in scan_events.iter() {
                                    match event {
                                        DrmScanEvent::Connected {
                                            connector,
                                            crtc: Some(crtc),
                                        } => {
                                            let mode = connector.modes().first().cloned();
                                            tracing::info!(
                                                connector = connector.interface().as_str(),
                                                crtc = ?crtc,
                                                width = mode.as_ref().map(|m| m.size().0).unwrap_or(0),
                                                height = mode.as_ref().map(|m| m.size().1).unwrap_or(0),
                                                refresh = mode.as_ref().map(|m| m.vrefresh()).unwrap_or(0),
                                                "🎨 Session 4 — connector+CRTC pair found, ready for first render"
                                            );
                                            tracing::warn!("Connector found but no CRTC available");
                                        }
                                        _ => {}
                                    }
                                }
                                if state.drm_device.is_none() {
                                    state.drm_device = Some(drm);
                                }
                            }
                            Err(e) => tracing::error!(?e, "Failed to create GBM device"),
                        }
                    }
                    Err(e) => tracing::error!(?e, "Failed to create DrmDevice"),
                }
            }
            Err(e) => tracing::error!(?e, path = ?path, "Failed to open DRM device"),
        }
    }

    // ── 5. Udev events ────────────────────────────────────────────────
    let mut udev_session = session.clone();
    event_loop
        .handle()
        .insert_source(udev_backend, move |event, _, state| {
            match event {
                UdevEvent::Added { device_id: _, path } => {
                    tracing::info!(path = ?path, "DRM device added — opening");

                    // ── Session 2: Open DRM Device ────────────────────
                    let open_flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
                    match udev_session.open(&path, open_flags) {
                        Ok(fd) => {
                            tracing::info!("DRM device fd opened successfully");
                            let drm_fd = DrmDeviceFd::new(fd.into());
                            match DrmDevice::new(drm_fd, true) {
                                Ok((drm, _drm_notifier)) => {
                                    // Log what we found
                                    let resources = drm.resource_handles().ok();
                                    let connector_count = resources.as_ref()
                                        .map(|r| r.connectors().len())
                                        .unwrap_or(0);
                                    let crtc_count = resources.as_ref()
                                        .map(|r| r.crtcs().len())
                                        .unwrap_or(0);

                                    tracing::info!(
                                        connectors = connector_count,
                                        crtcs = crtc_count,
                                        "DRM device opened — hardware enumerated"
                                    );

                                    let payload = format!(
                                        r#"{{"event":"device.opened","path":"{}","connectors":{},"crtcs":{}}}"#,
                                        path.to_string_lossy(),
                                        connector_count,
                                        crtc_count
                                    );
                                    state.emit("compositor.drm", payload);
                                }
                                Err(e) => {
                                    tracing::error!(?e, "Failed to create DrmDevice");
                                    let payload = format!(
                                        r#"{{"event":"device.error","path":"{}","error":"{}"}}"#,
                                        path.to_string_lossy(), e
                                    );
                                    state.emit("compositor.drm", payload);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(?e, path = ?path, "Failed to open DRM device");
                            let payload = format!(
                                r#"{{"event":"device.open_failed","path":"{}","error":"{}"}}"#,
                                path.to_string_lossy(), e
                            );
                            state.emit("compositor.drm", payload);
                        }
                    }
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

    state.session = Some(session);
    tracing::info!("DRM/udev backend initialized — faelight-compositor on real hardware");
    Ok(())
}

