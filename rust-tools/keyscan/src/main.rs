//! keyscan v0.1 - Sway Keybind Conflict Detection
//! 🌲 Faelight Forest

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

// ============================================================================
// TYPES
// ============================================================================

#[derive(Debug, Clone)]
struct Keybind {
    line: usize,
    binding: String,
    command: String,
}

struct ConflictReport {
    conflicts: HashMap<String, Vec<Keybind>>,
    total_binds: usize,
}

// ============================================================================
// CORE LOGIC
// ============================================================================

fn get_sway_config_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".config/sway/config")
}

fn parse_sway_config(path: &PathBuf) -> Result<Vec<Keybind>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    
    let mut keybinds = Vec::new();
    
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        
        // Match bindsym or bindcode
        if line.starts_with("bindsym") || line.starts_with("bindcode") {
            if let Some(keybind) = parse_keybind_line(line, line_num + 1) {
                keybinds.push(keybind);
            }
        }
    }
    
    Ok(keybinds)
}

fn parse_keybind_line(line: &str, line_num: usize) -> Option<Keybind> {
    // Remove "bindsym" or "bindcode" prefix
    let line = line.trim_start_matches("bindsym").trim_start_matches("bindcode").trim();
    
    // Remove flags like --no-repeat, --release, etc.
    let line = line
        .trim_start_matches("--no-repeat")
        .trim_start_matches("--release")
        .trim_start_matches("--locked")
        .trim_start_matches("--to-code")
        .trim();
    
    // Split on first whitespace to separate binding from command
    let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
    
    if parts.len() >= 2 {
        Some(Keybind {
            line: line_num,
            binding: normalize_binding(parts[0]),
            command: parts[1].trim().to_string(),
        })
    } else {
        None
    }
}

fn normalize_binding(binding: &str) -> String {
    // Normalize modifiers for consistent comparison
    binding
        .replace("Mod4", "Super")
        .replace("$mod", "Super")
        .to_lowercase()
}

fn find_conflicts(keybinds: &[Keybind]) -> ConflictReport {
    let mut map: HashMap<String, Vec<Keybind>> = HashMap::new();
    
    for keybind in keybinds {
        map.entry(keybind.binding.clone())
            .or_insert_with(Vec::new)
            .push(keybind.clone());
    }
    
    // Keep only conflicts (bindings with multiple entries)
    let conflicts: HashMap<String, Vec<Keybind>> = map
        .into_iter()
        .filter(|(_, binds)| binds.len() > 1)
        .collect();
    
    ConflictReport {
        conflicts,
        total_binds: keybinds.len(),
    }
}

fn report_conflicts(report: &ConflictReport, json: bool) {
    if json {
        output_json(report);
    } else {
        output_human(report);
    }
}

fn output_human(report: &ConflictReport) {
    println!("🔍 keyscan - Sway Keybind Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    if report.conflicts.is_empty() {
        println!("✅ No conflicts detected");
        println!("   {} unique keybindings", report.total_binds);
        return;
    }
    
    println!("⚠️  {} conflict(s) detected", report.conflicts.len());
    println!("   {} total keybindings", report.total_binds);
    println!();
    
    for (binding, binds) in &report.conflicts {
        println!("Conflict: {}", binding);
        for bind in binds {
            println!("  Line {}: {}", bind.line, bind.command);
        }
        println!();
    }
    
    println!("💡 Fix conflicts by removing or modifying duplicate bindings");
}

fn output_json(report: &ConflictReport) {
    println!("{{");
    println!("  \"total_binds\": {},", report.total_binds);
    println!("  \"conflicts\": [");
    
    let conflicts: Vec<_> = report.conflicts.iter().collect();
    for (i, (binding, binds)) in conflicts.iter().enumerate() {
        println!("    {{");
        println!("      \"binding\": \"{}\",", binding);
        println!("      \"occurrences\": [");
        
        for (j, bind) in binds.iter().enumerate() {
            let comma = if j < binds.len() - 1 { "," } else { "" };
            println!("        {{ \"line\": {}, \"command\": \"{}\" }}{}", 
                     bind.line, 
                     bind.command.replace('"', "\\\""),
                     comma);
        }
        
        let comma = if i < conflicts.len() - 1 { "," } else { "" };
        println!("      ]");
        println!("    }}{}", comma);
    }
    
    println!("  ]");
    println!("}}");
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let json = args.contains(&"--json".to_string());
    let config_path = if let Some(pos) = args.iter().position(|a| a == "--config") {
        if pos + 1 < args.len() {
            PathBuf::from(&args[pos + 1])
        } else {
            eprintln!("Error: --config requires a path");
            process::exit(1);
        }
    } else {
        get_sway_config_path()
    };
    
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("keyscan v0.1 - Sway Keybind Conflict Detection");
        println!();
        println!("Usage: keyscan [OPTIONS]");
        println!();
        println!("Options:");
        println!("  --config PATH    Use specific config file");
        println!("  --json           Output in JSON format");
        println!("  -h, --help       Show this help");
        println!("  -v, --version    Show version");
        return;
    }
    
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("keyscan v0.1.0");
        return;
    }
    
    // Parse config
    let keybinds = match parse_sway_config(&config_path) {
        Ok(kb) => kb,
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            process::exit(1);
        }
    };
    
    // Find conflicts
    let report = find_conflicts(&keybinds);
    
    // Report
    report_conflicts(&report, json);
    
    // Exit with error code if conflicts found
    if !report.conflicts.is_empty() {
        process::exit(1);
    }
}
