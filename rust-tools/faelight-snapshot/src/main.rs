//! faelight-snapshot v0.1 - Snapper Wrapper for Btrfs Snapshots
//! 🌲 Faelight Forest

use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return;
    }
    
    match args[1].as_str() {
        "list" | "ls" => list_snapshots(&args),
        "create" => create_snapshot(&args),
        "delete" | "rm" => delete_snapshot(&args),
        "diff" => diff_snapshot(&args),
        "rollback" => rollback_snapshot(&args),
        "status" => show_status(),
        "-h" | "--help" | "help" => print_help(),
        _ => {
            eprintln!("❌ Unknown command: {}", args[1]);
            print_help();
        }
    }
}

fn print_help() {
    println!("🌲 faelight-snapshot v0.1 - Btrfs Snapshot Manager");
    println!();
    println!("USAGE:");
    println!("    faelight-snapshot <command> [options]");
    println!();
    println!("COMMANDS:");
    println!("    list [root|home]     List snapshots (default: both)");
    println!("    create <desc>        Create pre-update snapshot");
    println!("    delete <num>         Delete snapshot by number");
    println!("    diff <num>           Show changes since snapshot");
    println!("    rollback <num>       Rollback to snapshot (requires reboot)");
    println!("    status               Show snapshot system status");
    println!();
    println!("EXAMPLES:");
    println!("    faelight-snapshot list");
    println!("    faelight-snapshot create \"before v6.7.0 update\"");
    println!("    faelight-snapshot diff 22");
    println!();
    println!("NOTE: Automatic hourly snapshots are already enabled via snapper.");
}

fn list_snapshots(args: &[String]) {
    let config = if args.len() > 2 { &args[2] } else { "both" };
    
    println!("📸 Btrfs Snapshots");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if config == "both" || config == "root" {
        println!("\n🔧 Root (/) snapshots:");
        let output = Command::new("sudo")
            .args(["snapper", "-c", "root", "list"])
            .output();
        
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Show last 10 lines
            for line in stdout.lines().take(1).chain(stdout.lines().skip(stdout.lines().count().saturating_sub(10))) {
                println!("  {}", line);
            }
        }
    }
    
    if config == "both" || config == "home" {
        println!("\n🏠 Home (/home) snapshots:");
        let output = Command::new("sudo")
            .args(["snapper", "-c", "home", "list"])
            .output();
        
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines().take(1).chain(stdout.lines().skip(stdout.lines().count().saturating_sub(10))) {
                println!("  {}", line);
            }
        }
    }
}

fn create_snapshot(args: &[String]) {
    let desc = if args.len() > 2 {
        args[2..].join(" ")
    } else {
        "manual snapshot".to_string()
    };
    
    println!("📸 Creating snapshots: {}", desc);
    
    // Create root snapshot
    let root_result = Command::new("sudo")
        .args(["snapper", "-c", "root", "create", "-d", &desc])
        .status();
    
    match root_result {
        Ok(s) if s.success() => println!("  ✅ Root snapshot created"),
        _ => println!("  ❌ Failed to create root snapshot"),
    }
    
    // Create home snapshot
    let home_result = Command::new("sudo")
        .args(["snapper", "-c", "home", "create", "-d", &desc])
        .status();
    
    match home_result {
        Ok(s) if s.success() => println!("  ✅ Home snapshot created"),
        _ => println!("  ❌ Failed to create home snapshot"),
    }
    
    println!();
    println!("💡 Run 'faelight-snapshot list' to see snapshots");
}

fn delete_snapshot(args: &[String]) {
    if args.len() < 3 {
        eprintln!("❌ Usage: faelight-snapshot delete <number>");
        return;
    }
    
    let num = &args[2];
    
    println!("🗑️  Deleting snapshot #{}", num);
    println!();
    println!("⚠️  This will delete from BOTH root and home configs.");
    println!("    Press Ctrl+C to cancel, or wait 3 seconds...");
    
    std::thread::sleep(std::time::Duration::from_secs(3));
    
    let _ = Command::new("sudo")
        .args(["snapper", "-c", "root", "delete", num])
        .status();
    
    let _ = Command::new("sudo")
        .args(["snapper", "-c", "home", "delete", num])
        .status();
    
    println!("✅ Snapshot #{} deleted", num);
}

fn diff_snapshot(args: &[String]) {
    if args.len() < 3 {
        eprintln!("❌ Usage: faelight-snapshot diff <number>");
        return;
    }
    
    let num = &args[2];
    
    println!("📊 Changes since snapshot #{}:", num);
    println!();
    
    let output = Command::new("sudo")
        .args(["snapper", "-c", "root", "status", &format!("{}..0", num)])
        .output();
    
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        
        if lines.is_empty() {
            println!("  No changes detected");
        } else {
            println!("  {} files changed", lines.len());
            for line in lines.iter().take(20) {
                println!("  {}", line);
            }
            if lines.len() > 20 {
                println!("  ... and {} more", lines.len() - 20);
            }
        }
    }
}

fn rollback_snapshot(args: &[String]) {
    if args.len() < 3 {
        eprintln!("❌ Usage: faelight-snapshot rollback <number>");
        return;
    }
    
    let num = &args[2];
    
    println!("⚠️  ROLLBACK WARNING");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("This will rollback to snapshot #{}", num);
    println!("A reboot is required after rollback.");
    println!();
    println!("Type 'rollback-confirm' to proceed:");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    
    if input.trim() != "rollback-confirm" {
        println!("❌ Rollback cancelled");
        return;
    }
    
    println!();
    println!("🔄 Rolling back to snapshot #{}...", num);
    
    let result = Command::new("sudo")
        .args(["snapper", "-c", "root", "rollback", num])
        .status();
    
    match result {
        Ok(s) if s.success() => {
            println!("✅ Rollback prepared");
            println!();
            println!("🔄 Reboot required to complete rollback!");
            println!("   Run: sudo reboot");
        }
        _ => println!("❌ Rollback failed"),
    }
}

fn show_status() {
    println!("📊 Snapshot System Status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Show configs
    let configs = Command::new("snapper")
        .args(["list-configs"])
        .output();
    
    if let Ok(out) = configs {
        println!("\n🔧 Configurations:");
        print!("{}", String::from_utf8_lossy(&out.stdout));
    }
    
    // Count snapshots
    let root_count = Command::new("sudo")
        .args(["snapper", "-c", "root", "list"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count().saturating_sub(2))
        .unwrap_or(0);
    
    let home_count = Command::new("sudo")
        .args(["snapper", "-c", "home", "list"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count().saturating_sub(2))
        .unwrap_or(0);
    
    println!("\n📸 Snapshot counts:");
    println!("  Root: {} snapshots", root_count);
    println!("  Home: {} snapshots", home_count);
    
    // Disk usage
    println!("\n💾 Btrfs disk usage:");
    let _ = Command::new("sudo")
        .args(["btrfs", "filesystem", "df", "/"])
        .status();
}
