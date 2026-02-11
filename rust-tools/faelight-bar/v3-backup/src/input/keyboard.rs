//! Keyboard input handling with mode awareness
use crate::state::{AppState, ModeState};
use smithay_client_toolkit::seat::keyboard::{KeyEvent, Keysym};
use std::process::Command;

pub fn handle_key_press(state: &mut AppState, event: KeyEvent) -> KeyAction {
    match &mut state.mode {
        ModeState::Bar(_) => KeyAction::None,
        ModeState::Menu(menu) => match event.keysym {
            Keysym::Escape => KeyAction::ExitMenu,

            Keysym::Return => {
                if let Some(item) = menu.get_selected_item() {
                    KeyAction::Execute(item)
                } else {
                    KeyAction::ExitMenu
                }
            }

            Keysym::BackSpace => {
                menu.input.pop();
                menu.refilter(&mut state.matcher);
                KeyAction::Redraw
            }

            Keysym::Up => {
                if menu.selected > 0 {
                    menu.selected -= 1;
                }
                KeyAction::Redraw
            }

            Keysym::Down => {
                if menu.selected < menu.filtered.len().saturating_sub(1) {
                    menu.selected += 1;
                }
                KeyAction::Redraw
            }

            _ => {
                if let Some(ref utf8) = event.utf8 {
                    for ch in utf8.chars() {
                        if ch.is_alphanumeric()
                            || ch.is_whitespace()
                            || ch == '-'
                            || ch == '_'
                            || ch == '.'
                        {
                            menu.input.push(ch);
                        }
                    }
                    menu.refilter(&mut state.matcher);
                    KeyAction::Redraw
                } else {
                    KeyAction::None
                }
            }
        },
    }
}

pub enum KeyAction {
    None,
    Redraw,
    ExitMenu,
    Execute(String),
}

pub fn execute_command(cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::env;

    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);

    // Ensure Wayland/X11 environment is passed to launched apps
    if let Ok(display) = env::var("WAYLAND_DISPLAY") {
        command.env("WAYLAND_DISPLAY", display);
    }
    if let Ok(display) = env::var("DISPLAY") {
        command.env("DISPLAY", display);
    }
    if let Ok(xdg) = env::var("XDG_RUNTIME_DIR") {
        command.env("XDG_RUNTIME_DIR", xdg);
    }

    command.spawn()?;
    Ok(())
}
