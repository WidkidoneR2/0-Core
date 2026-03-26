use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::env;
use std::io;
use std::path::PathBuf;

// Import from library
use faelight_fm::error::Result;

// Import binary-only modules
mod app;
mod input;
mod ui;

use app::AppState;

fn main() -> Result<()> {
    // Parse args: faelight-fm [start_path] [--cwd-file <path>]
    let args: Vec<String> = env::args().collect();
    let mut start_path = env::current_dir().expect("Failed to get current directory");
    let mut cwd_file: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--cwd-file" && i + 1 < args.len() {
            cwd_file = Some(args[i + 1].clone());
            i += 2;
        } else {
            start_path = PathBuf::from(&args[i]);
            i += 1;
        }
    }

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

    // Write cwd to file on exit if --cwd-file was passed
    if let Some(ref cwd_path) = cwd_file {
        let current = app.cwd.to_string_lossy().to_string();
        let _ = std::fs::write(cwd_path, &current);
    }
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

                    input::handle_key(key.code, app, terminal)?;
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
