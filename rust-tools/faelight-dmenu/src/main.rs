//! faelight-dmenu v0.1 - Generic Selector
//! 🌲 Faelight Forest

use clap::{Parser, Subcommand};
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};

mod search;
mod ui;
mod desktop;
mod icons;

use ui::{DmenuState, renderer::DmenuApp};

#[derive(Parser)]
#[command(name = "faelight-dmenu")]
#[command(about = "Generic selector for Faelight Forest")]
#[command(version)]
struct Cli {
    /// Prompt text
    #[arg(short, long, default_value = "Select:")]
    prompt: String,
    
    /// Enable multi-select mode
    #[arg(long)]
    multi: bool,
    
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Subcommand)]
enum Mode {
    /// Select from intents
    Intents {
        /// Filter by status (active, complete, planned)
        #[arg(long)]
        status: Option<String>,
        
        /// Filter by category
        #[arg(long)]
        category: Option<String>,
    },
    /// Discover all commands (future)
    Commands,
    /// Launch applications
    Apps,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.mode {
        None => {
            // Default: stdin mode
            run_stdin_mode(&cli.prompt, cli.multi);
        }
        Some(Mode::Intents { status, category }) => {
            println!("Intent mode not implemented yet!");
            println!("  Status filter: {:?}", status);
            println!("  Category filter: {:?}", category);
        }
        Some(Mode::Commands) => {
            println!("Commands mode not implemented yet!");
        }
        Some(Mode::Apps) => {
            run_apps_mode(&cli.prompt);
        }
    }
}

fn run_stdin_mode(_prompt: &str, _multi: bool) {
    // Read all lines from stdin
    let stdin = io::stdin();
    let items: Vec<String> = stdin.lock()
        .lines()
        .filter_map(|line| line.ok())
        .collect();
    
    if items.is_empty() {
        eprintln!("No items provided");
        std::process::exit(1);
    }
    
    eprintln!("Loaded {} items", items.len());
    
    // Create shared state
    let state = Arc::new(Mutex::new(DmenuState::new(items)));
    
    // Initialize Wayland app
    let (mut app, conn, mut event_queue) = DmenuApp::new(state.clone());
    let qh = event_queue.handle();
    
    // Create surface
    app.init_surface(&qh);
    
    // Roundtrip to get initial configure event
    event_queue.roundtrip(&mut app).unwrap();
    
    // Event loop
    loop {
        event_queue.blocking_dispatch(&mut app).unwrap();
        
        if app.exit {
            break;
        }
        
        // Check if user made a selection
        let state_lock = state.lock().unwrap();
        if !state_lock.running {
            break;
        }
    }
    
    // Output result
    let state_lock = state.lock().unwrap();
    if let Some(result) = &state_lock.result {
        println!("{}", result);
    }
}

fn run_apps_mode(prompt: &str) {
    use desktop::DesktopEntry;
    
    eprintln!("🔍 Discovering applications...");
    let apps = desktop::scan_applications();
    
    if apps.is_empty() {
        eprintln!("No applications found");
        std::process::exit(1);
    }
    
    eprintln!("📱 Found {} applications", apps.len());
    
    let items: Vec<String> = apps.iter()
        .map(|app| app.name.clone())
        .collect();
    
    let state = Arc::new(Mutex::new(DmenuState::new(items)));
    let (mut app, conn, mut event_queue) = DmenuApp::new(state.clone());
    let qh = event_queue.handle();
    
    app.init_surface(&qh);
    event_queue.roundtrip(&mut app).unwrap();
    
    loop {
        event_queue.blocking_dispatch(&mut app).unwrap();
        if app.exit { break; }
        
        let state_lock = state.lock().unwrap();
        if !state_lock.running { break; }
    }
    
    let state_lock = state.lock().unwrap();
    if let Some(selected_name) = &state_lock.result {
        if let Some(app) = apps.iter().find(|a| &a.name == selected_name) {
            eprintln!("🚀 Launching: {}", app.name);
            app.exec();
        }
    }
}
