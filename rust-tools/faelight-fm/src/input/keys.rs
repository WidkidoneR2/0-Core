use crossterm::event::KeyCode;
use crate::app::AppState;
use faelight_fm::error::Result;
use faelight_zone::Zone;

pub fn handle_key(key: KeyCode, app: &mut AppState) -> Result<()> {
    // If any overlay is visible, any key closes it
    if app.help_visible {
        app.toggle_help();
        return Ok(());
    }
    
    if app.preview_visible {
        app.toggle_preview();
        return Ok(());
    }
    
    if app.info_visible {
        app.toggle_info();
        return Ok(());
    }
    
    // Search mode - handle text input
    if app.search_mode {
        match key {
            KeyCode::Char(c) => app.search_add_char(c),
            KeyCode::Backspace => app.search_backspace(),
            KeyCode::Esc | KeyCode::Enter => app.exit_search(),
            KeyCode::Down => app.select_next(),
            KeyCode::Up => app.select_prev(),
            _ => {}
        }
        return Ok(());
    }
    
    // Normal mode
    match key {
        // Navigation
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => app.enter_selected()?,
        KeyCode::Char('h') | KeyCode::Left => app.go_parent()?,
        
        // Zone jumping
        KeyCode::Char('0') => app.jump_to_zone(Zone::Core)?,
        KeyCode::Char('1') => app.jump_to_zone(Zone::Workspace)?,
        KeyCode::Char('2') => app.jump_to_zone(Zone::Src)?,
        KeyCode::Char('3') => app.jump_to_zone(Zone::Project)?,
        KeyCode::Char('4') => app.jump_to_zone(Zone::Archive)?,
        KeyCode::Char('5') => app.jump_to_zone(Zone::Scratch)?,
        
        // Overlays
        KeyCode::Char('?') => app.toggle_help(),
        KeyCode::Char('i') => app.toggle_info(),
        KeyCode::Char('p') => {
            app.load_preview();
            app.toggle_preview();
        },
        
        // Search
        KeyCode::Char('/') => app.start_search(),
        
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        
        _ => {}
    }
    
    Ok(())
}
