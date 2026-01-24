mod model;
mod detect;

pub use model::Zone;

use std::path::Path;

/// Detect the current zone based on working directory
pub fn current_zone(cwd: &Path, home: &Path) -> Zone {
    detect::detect_zone(cwd, home)
}
