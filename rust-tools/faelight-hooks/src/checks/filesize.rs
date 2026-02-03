use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const MAX_FILE_SIZE_MB: u64 = 50; // 50MB warning threshold
const MAX_FILE_SIZE_BYTES: u64 = MAX_FILE_SIZE_MB * 1024 * 1024;

pub fn check_file_sizes() -> Result<bool> {
    println!("{}", "🔍 Checking staged file sizes...".cyan());
    
    // Get staged files
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .context("Failed to get staged files")?;
    
    let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();
    
    if files.is_empty() {
        println!("{}", "  ℹ️  No staged files".dimmed());
        return Ok(true);
    }
    
    let mut large_files = Vec::new();
    
    for file in files {
        let path = PathBuf::from(&file);
        
        if !path.exists() {
            continue; // Deleted file
        }
        
        match fs::metadata(&path) {
            Ok(metadata) => {
                let size = metadata.len();
                
                if size > MAX_FILE_SIZE_BYTES {
                    large_files.push((file, size));
                }
            }
            Err(_) => continue,
        }
    }
    
    if !large_files.is_empty() {
        println!();
        println!("{}", "⚠️  LARGE FILES DETECTED!".yellow().bold());
        println!();
        
        for (file, size) in &large_files {
            let size_mb = *size as f64 / (1024.0 * 1024.0);
            println!("  {} - {:.2} MB", file.yellow(), size_mb);
        }
        
        println!();
        println!("{}", format!("Threshold: {} MB", MAX_FILE_SIZE_MB).dimmed());
        println!();
        println!("{}", "💡 Consider:".yellow());
        println!("  • Using Git LFS for large files");
        println!("  • Compressing the files");
        println!("  • Storing elsewhere (S3, etc.)");
        println!();
        println!("{}", "⚠️  This is a warning, not an error.".yellow());
        println!();
    } else {
        println!("{}", "✅ All files within size limits".green());
    }
    
    Ok(true)
}
