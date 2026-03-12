// faelight-compositor — The compositor that joins the family
// INT-109 — Phase 3: faelight-compositor on Smithay
//
// "Every other compositor is substrate.
//  faelight-compositor is a participant."

mod handlers;
mod state;
mod winit;

use smithay::reexports::{calloop::EventLoop, wayland_server::Display};
use state::FaelightCompositor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    tracing::info!("faelight-compositor starting — the last sibling comes home");

    let mut event_loop: EventLoop<FaelightCompositor> = EventLoop::try_new()?;
    let display: Display<FaelightCompositor> = Display::new()?;
    let mut state = FaelightCompositor::new(&mut event_loop, display);

    // Initialize winit backend (nested inside Niri for testing)
    winit::init_winit(&mut event_loop, &mut state)?;

    std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);
    tracing::info!(
        socket = ?state.socket_name,
        "faelight-compositor ready — forest socket open"
    );

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
