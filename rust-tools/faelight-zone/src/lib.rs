mod model;
mod detect;

pub use model::Zone;

use std::path::Path;

/// Detect the current zone and relevant path based on working directory
/// Returns (Zone, display_path)
pub fn current_zone(cwd: &Path, home: &Path) -> (Zone, String) {
    detect::detect_zone(cwd, home)
}
