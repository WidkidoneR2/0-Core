// faelight-compositor — The compositor that joins the family
// INT-109 — Phase 3: faelight-compositor on Smithay
//
// "Every other compositor is substrate.
//  faelight-compositor is a participant."
//
// Backends:
//   --winit   Run nested inside Niri (development/testing)
//   --drm     Run on real hardware (production)

mod handlers;
mod input;
mod state;
mod udev_backend;
mod winit;

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};
use state::FaelightCompositor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure XDG_RUNTIME_DIR is set — needed for Wayland socket
    if std::env::var("XDG_RUNTIME_DIR").is_err() {
        let uid = unsafe { libc::getuid() };
        let runtime_dir = format!("/run/user/{}", uid);
        std::env::set_var("XDG_RUNTIME_DIR", &runtime_dir);
        eprintln!("ℹ️  XDG_RUNTIME_DIR not set — using {}", runtime_dir);
    }
    init_logging();

    let args: Vec<String> = std::env::args().collect();
    let use_drm = args.contains(&"--drm".to_string());
    let _use_winit = args.contains(&"--winit".to_string()) || !use_drm;

    if use_drm {
        tracing::info!("faelight-compositor starting — DRM/udev backend (real hardware)");
    } else {
        tracing::info!("faelight-compositor starting — winit backend (nested)");
    }
    tracing::info!("the last sibling comes home");

    let mut event_loop: EventLoop<FaelightCompositor> = EventLoop::try_new()?;
    let display: Display<FaelightCompositor> = Display::new()?;
    let mut state = FaelightCompositor::new(&mut event_loop, display);

    if use_drm {
        udev_backend::init_drm(&mut event_loop, &mut state)?;
    } else {
        winit::init_winit(&mut event_loop, &mut state)?;
    }

    std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);
    tracing::info!(
        socket = ?state.socket_name,
        backend = if use_drm { "drm" } else { "winit" },
        "faelight-compositor ready — forest socket open"
    );

    tracing::info!("💡 VT switch: Ctrl+Alt+F1-F7  |  Exit: Ctrl+Alt+Q");
    event_loop.run(None, &mut state, move |_| {})?;

    Ok(())
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}
