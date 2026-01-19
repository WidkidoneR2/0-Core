//! File search - Find recent files across configured directories
use super::SearchResult;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use walkdir::WalkDir;

/// Configuration for file search
pub struct FileSearchConfig {
    /// Directories to search
    pub search_dirs: Vec<PathBuf>,
    /// Maximum age in days (0 = all files)
    pub max_age_days: u64,
    /// Maximum number of results
    pub max_results: usize,
}

impl Default for FileSearchConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/christian".to_string());
        Self {
            search_dirs: vec![
                PathBuf::from(format!("{}/0-core", home)),
                PathBuf::from(format!("{}/1-src", home)),
                PathBuf::from(format!("{}/2-projects", home)),
            ],
            max_age_days: 30, // Last 30 days
            max_results: 50,
        }
    }
}

/// Search for files matching query
pub fn search_files(query: &str, config: &FileSearchConfig) -> Vec<SearchResult> {
    if query.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let max_age_secs = config.max_age_days * 24 * 60 * 60;

    for dir in &config.search_dirs {
        if !dir.exists() {
            continue;
        }

        for entry in WalkDir::new(dir)
            .max_depth(5)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden dirs, .git, target, node_modules
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') 
                    && name != "target" 
                    && name != "node_modules"
            })
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let file_name = match path.file_name() {
                Some(n) => n.to_string_lossy().to_string(),
                None => continue,
            };

            // Check age
            let metadata = match fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let modified = match metadata.modified() {
                Ok(m) => m.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
                Err(_) => continue,
            };

            if config.max_age_days > 0 && (now - modified) > max_age_secs {
                continue;
            }

            // Fuzzy match
            let score = fuzzy_score(query, &file_name);
            if score > 0.0 {
                // Boost score for recently modified files
                let age_days = (now - modified) / (24 * 60 * 60);
                let recency_boost = if age_days == 0 {
                    2.0
                } else if age_days < 7 {
                    1.5
                } else if age_days < 30 {
                    1.2
                } else {
                    1.0
                };

                results.push(SearchResult::File {
                    name: file_name,
                    path: path.to_string_lossy().to_string(),
                    modified,
                    score: score * recency_boost,
                });
            }
        }
    }

    // Sort by score
    results.sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap());
    results.truncate(config.max_results);
    results
}

/// Simple fuzzy scoring (matches existing launcher logic)
fn fuzzy_score(query: &str, target: &str) -> f32 {
    if query.is_empty() {
        return 100.0;
    }

    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();

    // Exact match
    if target_lower == query_lower {
        return 100.0;
    }

    // Prefix match
    if target_lower.starts_with(&query_lower) {
        return 90.0;
    }

    // Contains match
    if target_lower.contains(&query_lower) {
        return 70.0;
    }

    // Subsequence match (fuzzy)
    let mut query_chars = query_lower.chars().peekable();
    let mut matches = 0;
    for ch in target_lower.chars() {
        if query_chars.peek() == Some(&ch) {
            query_chars.next();
            matches += 1;
        }
    }

    if query_chars.peek().is_none() {
        // All query chars matched
        50.0 * (matches as f32 / target_lower.len() as f32)
    } else {
        0.0
    }
}
