use crossterm::event::MouseEvent;
use crate::app::AppState;
use faelight_fm::error::Result;

pub fn handle_mouse(app: &mut AppState, event: MouseEvent) -> Result<()> {
    use crossterm::event::MouseEventKind;
    
    match event.kind {
        // Only allow safe scroll wheel navigation
        MouseEventKind::ScrollDown => app.select_next(),
        MouseEventKind::ScrollUp => app.select_prev(),
        
        // TODO: Add proper click detection later
        // Need to calculate exact widget boundaries for safe clicking
        _ => {}
    }
    
    Ok(())
}
