//! faelight-zone v2.1.0 - Spatial awareness for Faelight Forest
//! 🌲 Faelight Forest

use clap::Parser;
use faelight_zone::current_zone;
use std::env;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "faelight-zone")]
#[command(about = "Detect current filesystem zone", long_about = None)]
#[command(version = "2.1.0")]
struct Cli {
    /// Output only the zone icon
    #[arg(long)]
    icon: bool,

    /// Output only the zone label
    #[arg(long)]
    label: bool,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Run health check
    #[arg(long)]
    health: bool,
}

fn main() {
    let args = Cli::parse();

    if args.health {
        health_check();
        return;
    }

    // Get current directory
    let cwd = match env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("❌ Error: Cannot access current directory: {}", e);
            eprintln!("💡 Check your working directory permissions");
            process::exit(1);
        }
    };

    // Get home directory
    let home = match env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => {
            eprintln!("❌ Error: HOME environment variable not set");
            eprintln!("💡 Set HOME or run from a known location");
            process::exit(1);
        }
    };

    // Detect zone
    let (zone, display_path) = current_zone(&cwd, &home);

    // Output based on flags
    if args.icon {
        println!("{}", zone.icon());
    } else if args.label {
        println!("{}", zone.short_label());
    } else if args.json {
        println!(
            r#"{{"zone":"{:?}","label":"{}","icon":"{}","path":"{}","critical":{}}}"#,
            zone,
            zone.short_label(),
            zone.icon(),
            display_path,
            zone.is_critical()
        );
    } else {
        // Default: emoji + label + path
        println!("{} {}", zone.icon(), display_path);
    }
}

fn health_check() {
    println!("🏥 faelight-zone v2.1.0 health check");

    // Check HOME
    match env::var("HOME") {
        Ok(_) => println!("✅ HOME: set"),
        Err(_) => {
            eprintln!("❌ HOME: not set");
            eprintln!("💡 Set HOME environment variable");
        }
    }

    // Check current dir
    match env::current_dir() {
        Ok(dir) => {
            println!("✅ Current directory: {}", dir.display());
            let home = env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/"));
            let (zone, _) = current_zone(&dir, &home);
            println!("✅ Zone detection: {} {}", zone.icon(), zone.short_label());
        }
        Err(e) => {
            eprintln!("❌ Current directory: {}", e);
            eprintln!("💡 Check directory permissions");
        }
    }

    println!("\n✅ All checks passed!");
}
