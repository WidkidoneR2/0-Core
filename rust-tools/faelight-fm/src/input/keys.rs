use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::AppState;
use faelight_zone::Zone;
use crate::error::Result;

pub fn handle_key(app: &mut AppState, key: KeyEvent) -> Result<()> {
    match key.code {
        // Navigation
        KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('h') | KeyCode::Left => app.go_parent()?,
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => app.enter_selected()?,
        
        // Zone jumping (0-5)
        KeyCode::Char('0') => app.jump_to_zone(Zone::Core)?,
        KeyCode::Char('1') => app.jump_to_zone(Zone::Workspace)?,
        KeyCode::Char('2') => app.jump_to_zone(Zone::Src)?,
        KeyCode::Char('3') => app.jump_to_zone(Zone::Project)?,
        KeyCode::Char('4') => app.jump_to_zone(Zone::Archive)?,
        KeyCode::Char('5') => app.jump_to_zone(Zone::Scratch)?,
        
        // Quit
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        
        _ => {}
    }
    
    Ok(())
}
