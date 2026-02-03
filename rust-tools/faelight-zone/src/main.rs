use std::env;
use faelight_core::paths;
use faelight_zone::current_zone;

fn main() {
    let cwd = env::current_dir().unwrap_or_else(|_| paths::home());
    let home = paths::home();
    
    let (zone, path) = current_zone(&cwd, &home);
    
    // Output format: "🔒 0-core" (icon + path, no label)
    println!("{} {}", zone.icon(), path);
}
