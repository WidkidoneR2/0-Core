use std::env;
use std::path::PathBuf;
use faelight_zone::current_zone;

fn main() {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let home = env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));

    let zone = current_zone(&cwd, &home);

    // Output format: "🦀 WORK"
    println!("{} {}", zone.icon(), zone.short_label());
}
