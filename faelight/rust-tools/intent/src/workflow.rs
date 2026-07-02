//! Intent workflow commands - lifecycle management
//!
//! Handles state transitions: planned → in-progress → complete

use faelight_core::paths;
use std::fs;
use std::path::PathBuf;

// Colors (re-export from main)
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const NC: &str = "\x1b[0m";

/// Get current date in YYYY-MM-DD format
fn get_current_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Find intent file by ID across all categories
fn find_intent_by_id(id: &str) -> Option<PathBuf> {
    let intent_dir = paths::intents_dir();
    let categories = [
        "future",
        "complete",
        "decisions",
        "deferred",
        "cancelled",
        "experiments",
        "philosophy",
        "incidents",
    ];

    for cat in &categories {
        let cat_dir = intent_dir.join(cat);
        if !cat_dir.exists() {
            continue;
        }

        if let Ok(entries) = fs::read_dir(&cat_dir) {
            for entry in entries.flatten() {
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();

                // Match by ID at start of filename
                if filename_str.starts_with(&format!("{}-", id))
                    || filename_str.starts_with(&format!("0{}-", id))
                    || filename_str.starts_with(&format!("00{}-", id))
                {
                    return Some(entry.path());
                }
            }
        }
    }
    None
}

/// Update a frontmatter field in markdown content
fn update_frontmatter_field(content: &str, key: &str, value: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::new();
    let mut in_frontmatter = false;
    let mut frontmatter_end = 0;
    let mut field_updated = false;

    // Find frontmatter bounds
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
            } else {
                frontmatter_end = i;
                break;
            }
        }
    }

    if frontmatter_end == 0 {
        eprintln!("{}⚠️  No frontmatter found{}", YELLOW, NC);
        return content.to_string();
    }

    // Build result, updating field if found
    for (i, line) in lines.iter().enumerate() {
        if i > 0
            && i < frontmatter_end
            && (line.starts_with(&format!("{}: ", key)) || line.starts_with(&format!("{}:", key)))
        {
            result.push_str(&format!("{}: {}\n", key, value));
            field_updated = true;
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }

    // If field not found, add it before closing ---
    if !field_updated {
        let mut lines_vec: Vec<String> = result.lines().map(|s| s.to_string()).collect();
        lines_vec.insert(frontmatter_end, format!("{}: {}", key, value));
        result = lines_vec.join("\n") + "\n";
    }

    result
}

/// Start an intent: planned → in-progress
pub fn start_intent(id: &str) {
    let intent_path = match find_intent_by_id(id) {
        Some(path) => path,
        None => {
            eprintln!("{}❌ Intent {} not found{}", RED, id, NC);
            return;
        }
    };

    let content = fs::read_to_string(&intent_path).expect("Failed to read intent file");

    let mut updated = update_frontmatter_field(&content, "status", "in-progress");
    updated = update_frontmatter_field(&updated, "started", &get_current_date());

    fs::write(&intent_path, updated).expect("Failed to write intent file");

    println!("{}🚀 Started intent {}{}", GREEN, id, NC);
    println!("{}   Status: planned → in-progress{}", GREEN, NC);
    println!("{}   Started: {}{}", GREEN, get_current_date(), NC);
}

/// Complete an intent: move to complete/ folder
pub fn complete_intent(id: &str) {
    let intent_path = match find_intent_by_id(id) {
        Some(path) => path,
        None => {
            eprintln!("{}❌ Intent {} not found{}", RED, id, NC);
            return;
        }
    };

    let filename = intent_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let content = fs::read_to_string(&intent_path).expect("Failed to read intent file");

    let mut updated = update_frontmatter_field(&content, "status", "complete");
    updated = update_frontmatter_field(&updated, "completed", &get_current_date());

    let complete_dir = paths::intents_complete();
    fs::create_dir_all(&complete_dir).ok();
    let new_path = complete_dir.join(&filename);

    fs::write(&new_path, updated).expect("Failed to write completed intent");
    fs::remove_file(&intent_path).expect("Failed to remove old intent file");

    println!();
    println!(
        "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        GREEN, NC
    );
    println!("{}🎊 INTENT {} COMPLETE!{}", GREEN, id, NC);
    println!(
        "{}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{}",
        GREEN, NC
    );
    println!("{}   Status: → complete{}", GREEN, NC);
    println!("{}   Completed: {}{}", GREEN, get_current_date(), NC);
    println!("{}   Moved to: complete/{}{}", GREEN, filename, NC);
    println!();
    println!(
        "{}🌲 Well done! Another step forward in the Forest.{}",
        GREEN, NC
    );
    println!();
}

/// Defer an intent: move to deferred/ folder
pub fn defer_intent(id: &str, reason: Option<&str>) {
    let intent_path = match find_intent_by_id(id) {
        Some(path) => path,
        None => {
            eprintln!("{}❌ Intent {} not found{}", RED, id, NC);
            return;
        }
    };

    let filename = intent_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let content = fs::read_to_string(&intent_path).expect("Failed to read intent file");

    let mut updated = update_frontmatter_field(&content, "status", "deferred");
    updated = update_frontmatter_field(&updated, "deferred_date", &get_current_date());

    if let Some(r) = reason {
        updated = update_frontmatter_field(&updated, "deferred_reason", &format!("\"{}\"", r));
    }

    let deferred_dir = paths::intents_deferred();
    fs::create_dir_all(&deferred_dir).ok();
    let new_path = deferred_dir.join(&filename);

    fs::write(&new_path, updated).expect("Failed to write deferred intent");
    fs::remove_file(&intent_path).expect("Failed to remove old intent");

    println!("{}📌 Deferred intent {}{}", YELLOW, id, NC);
    println!("{}   Moved to: deferred/{}{}", YELLOW, filename, NC);
    if let Some(r) = reason {
        println!("{}   Reason: {}{}", YELLOW, r, NC);
    }
}

/// Cancel an intent: move to cancelled/ folder
pub fn cancel_intent(id: &str, reason: Option<&str>) {
    let intent_path = match find_intent_by_id(id) {
        Some(path) => path,
        None => {
            eprintln!("{}❌ Intent {} not found{}", RED, id, NC);
            return;
        }
    };

    let filename = intent_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let content = fs::read_to_string(&intent_path).expect("Failed to read intent file");

    let mut updated = update_frontmatter_field(&content, "status", "cancelled");
    updated = update_frontmatter_field(&updated, "cancelled_date", &get_current_date());

    if let Some(r) = reason {
        updated = update_frontmatter_field(&updated, "cancelled_reason", &format!("\"{}\"", r));
    }

    let cancelled_dir = paths::intents_cancelled();
    fs::create_dir_all(&cancelled_dir).ok();
    let new_path = cancelled_dir.join(&filename);

    fs::write(&new_path, updated).expect("Failed to write cancelled intent");
    fs::remove_file(&intent_path).expect("Failed to remove old intent");

    println!("{}🚫 Cancelled intent {}{}", RED, id, NC);
    println!("{}   Moved to: cancelled/{}{}", RED, filename, NC);
    if let Some(r) = reason {
        println!("{}   Reason: {}{}", RED, r, NC);
    }
}
