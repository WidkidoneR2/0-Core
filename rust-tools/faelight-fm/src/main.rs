use std::io;
use std::env;
use std::path::PathBuf;
use crossterm::{
    event::{self, Event, KeyCode, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

// Import from library
use faelight_fm::error::Result;

// Import binary-only modules
mod app;
mod input;
mod ui;

use app::AppState;

fn main() -> Result<()> {
    // Get starting directory
    let start_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = AppState::new(start_path)?;
    
    // Main event loop
    let res = run_app(&mut terminal, &mut app);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }
    
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut AppState) -> Result<()> {
    loop {
        // Render
        terminal.draw(|f| ui::render(f, app))?;
        
        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Quick quit on ESC
                    if key.code == KeyCode::Esc {
                        app.quit();
                    }
                    
                    input::handle_key(key.code, app)?;
                }
                Event::Mouse(mouse) => {
                    input::handle_mouse(app, mouse)?;
                }
                _ => {}
            }
        }
        
        // Exit if quit
        if !app.running {
            break;
        }
    }
    
    Ok(())
}
