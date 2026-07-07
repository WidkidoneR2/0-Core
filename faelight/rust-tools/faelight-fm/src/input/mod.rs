// faelight-fm v3.1 -- input handling and keybindings

use crate::types::Mode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
#[allow(dead_code)]
pub enum Action {
    MoveDown,
    MoveUp,
    MoveTop,
    MoveBottom,
    NavigateInto,
    NavigateUp,
    YankPath,
    DeleteSelected,
    ConfirmDelete,
    CancelDelete,
    StageUnstage,
    ToggleHidden,
    NixInfo,
    FilterChar(char),
    FilterBackspace,
    FilterExit,
    Quit,
    None,
}

#[allow(dead_code)]
pub fn handle_key(key: KeyEvent, mode: &Mode) -> Action {
    match mode {
        Mode::ConfirmDelete(_) => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmDelete,
            _ => Action::CancelDelete,
        },
        Mode::Command(_) | Mode::Filter(_) => match key.code {
            KeyCode::Esc => Action::FilterExit,
            KeyCode::Enter => Action::FilterExit,
            KeyCode::Backspace => Action::FilterBackspace,
            KeyCode::Char(c) => Action::FilterChar(c),
            _ => Action::None,
        },
        Mode::Normal => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
            KeyCode::Char('g') => Action::MoveTop,
            KeyCode::Char('G') => Action::MoveBottom,
            KeyCode::Char('l') | KeyCode::Enter => Action::NavigateInto,
            KeyCode::Char('h') | KeyCode::Backspace => Action::NavigateUp,
            KeyCode::Char('y') => Action::YankPath,
            KeyCode::Char('d') => Action::DeleteSelected,
            KeyCode::Char('s') => Action::StageUnstage,
            KeyCode::Char('.') => Action::ToggleHidden,
            KeyCode::Char('n') => Action::NixInfo,
            KeyCode::Char('/') => Action::FilterChar('/'),
            KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                // Start filter mode on any printable char
                Action::FilterChar(c)
            }
            _ => Action::None,
        },
    }
}
