//! faelight-term v2 -- Phase 0: Foundation
#![allow(dead_code, unused_imports, unused_variables)]
//! Goal: Wayland window opens, PTY spawns faelight-shell, characters render.
//! Nothing else. One window. One shell. One character on screen.
mod config;
mod renderer;
mod terminal;
mod input;
mod pty;
use config::Config;
fn main() {
    let config = Config::load();
    
    println!("faelight-term v2 -- Phase 0");
    println!("GPU: wgpu + Vulkan (AMD Radeon RX 7700S)");
    println!("Text: cosmic-text");
    println!("Starting...");
    
    if let Err(e) = run(config) {
        eprintln!("faelight-term error: {}", e);
        std::process::exit(1);
    }
}
fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // Phase 0: bring up the event loop and window
    // Full implementation built gate by gate
    todo!("Phase 0: implement event loop and window")
}
