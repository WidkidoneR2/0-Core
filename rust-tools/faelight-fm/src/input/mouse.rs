use crate::app::AppState;
use crossterm::event::{MouseButton, MouseEvent};
use faelight_fm::error::Result;
use faelight_zone::Zone;
use std::time::{Duration, Instant};

static mut LAST_CLICK: Option<(Instant, u16, u16)> = None;
const DOUBLE_CLICK_MS: u64 = 500;

pub fn handle_mouse(app: &mut AppState, event: MouseEvent) -> Result<()> {
    use crossterm::event::MouseEventKind;

    match event.kind {
        MouseEventKind::ScrollDown => app.select_next(),
        MouseEventKind::ScrollUp => app.select_prev(),

        MouseEventKind::Down(MouseButton::Left) => {
            handle_left_click(app, event.column, event.row)?;
        }

        _ => {}
    }

    Ok(())
}

fn handle_left_click(app: &mut AppState, x: u16, y: u16) -> Result<()> {
    let is_double_click = unsafe {
        if let Some((last_time, last_x, last_y)) = LAST_CLICK {
            let elapsed = last_time.elapsed();
            if elapsed < Duration::from_millis(DOUBLE_CLICK_MS) && last_x == x && last_y == y {
                LAST_CLICK = None;
                true
            } else {
                LAST_CLICK = Some((Instant::now(), x, y));
                false
            }
        } else {
            LAST_CLICK = Some((Instant::now(), x, y));
            false
        }
    };

    // CHECK ZONES FIRST (priority over files)
    if let Some(zone) = find_clicked_zone(app, x, y) {
        app.jump_to_zone(zone)?;
        return Ok(());
    }

    // Then check files
    if is_double_click {
        app.enter_selected()?;
    } else if let Some(file_idx) = find_clicked_file(app, x, y) {
        app.selected = file_idx;
    }

    Ok(())
}

fn find_clicked_file(app: &AppState, _x: u16, y: u16) -> Option<usize> {
    for (row, _width, file_idx) in &app.file_click_regions {
        if *row == y && *file_idx < app.filtered_entries.len() {
            return Some(*file_idx);
        }
    }
    None
}

fn find_clicked_zone(app: &AppState, x: u16, y: u16) -> Option<Zone> {
    for (zx, zy, zw, zh, zone_num) in &app.zone_click_regions {
        if x >= *zx && x < (*zx + *zw) && y >= *zy && y < (*zy + *zh) {
            return match zone_num {
                0 => Some(Zone::Core),
                1 => Some(Zone::Workspace),
                2 => Some(Zone::Src),
                3 => Some(Zone::Project),
                4 => Some(Zone::Archive),
                5 => Some(Zone::Scratch),
                _ => None,
            };
        }
    }
    None
}
