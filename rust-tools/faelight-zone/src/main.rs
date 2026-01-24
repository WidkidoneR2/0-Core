use std::env;
use std::path::PathBuf;
use faelight_zone::current_zone;

fn main() {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let home = env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));

    let (zone, path) = current_zone(&cwd, &home);

    // Output format: "🔒 0-core" (icon + path, no label)
    println!("{} {}", zone.icon(), path);
}
