//! Test file search functionality

fn main() {
    use faelight_launcher::search::{files, SearchResult};

    println!("🔍 Testing file search...\n");

    let config = files::FileSearchConfig::default();
    println!("📁 Search directories:");
    for dir in &config.search_dirs {
        println!("  - {}", dir.display());
    }

    println!("\n🔎 Searching for 'main'...");
    let results = files::search_files("main", &config);

    println!("Found {} results:", results.len());
    for (i, result) in results.iter().take(10).enumerate() {
        if let SearchResult::File {
            name, path, score, ..
        } = result
        {
            println!("  {}. {} (score: {:.1})", i + 1, name, score);
            println!("     {}", path);
        }
    }
}
