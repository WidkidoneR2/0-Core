mod icons;
mod state;
mod format;

use state::SystemState;

fn main() {
    let state = SystemState::gather();
    format::print_output(&state);
}
