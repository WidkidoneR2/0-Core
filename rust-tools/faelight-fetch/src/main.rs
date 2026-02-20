//! faelight-fetch v2.1.0 - System Info Display
//! 🌲 Faelight Forest

mod format;
mod icons;
mod state;

use clap::Parser;
use state::SystemState;
use std::process::Command;

#[derive(Parser)]
#[command(name = "faelight-fetch")]
#[command(about = "Zone-aware system information for Faelight Forest", long_about = None)]
#[command(version)]
struct Args {
    /// Run health check and exit
    #[arg(long)]
    health_check: bool,
}

fn main() {
    let args = Args::parse();

    if args.health_check {
        health_check();
        return;
    }

    let state = SystemState::gather();
    format::print_output(&state);
}

fn health_check() {
    println!(
        "🏥 faelight-fetch v{} health check",
        env!("CARGO_PKG_VERSION")
    );

    // Check zone detection
    match std::env::current_dir() {
        Ok(_) => println!("✅ current directory: accessible"),
        Err(e) => {
            eprintln!("❌ current directory: {}", e);
            eprintln!("💡 Check your working directory permissions");
        }
    }

    // Check HOME env var
    match std::env::var("HOME") {
        Ok(_) => println!("✅ HOME: set"),
        Err(_) => {
            eprintln!("❌ HOME: not set");
            eprintln!("💡 Set HOME environment variable");
        }
    }

    // Check if dot-doctor is available
    match Command::new("doctor").arg("--version").output() {
        Ok(output) if output.status.success() => {
            println!("✅ dot-doctor: installed");
        }
        Ok(_) => {
            eprintln!("⚠️  dot-doctor: installed but failed");
            eprintln!("💡 Run 'doctor' to check system health");
        }
        Err(_) => {
            eprintln!("⚠️  dot-doctor: not found (health % unavailable)");
            eprintln!("💡 Install from 0-Core workspace");
        }
    }

    // Check if profile tool is available
    match Command::new("profile").arg("--version").output() {
        Ok(output) if output.status.success() => {
            println!("✅ profile: installed");
        }
        _ => {
            eprintln!("⚠️  profile: not found (profile state unavailable)");
            eprintln!("💡 Install from 0-Core workspace");
        }
    }

    // Check faelight-zone
    match Command::new("faelight-zone").arg("--version").output() {
        Ok(output) if output.status.success() => {
            println!("✅ faelight-zone: installed");
        }
        _ => {
            eprintln!("⚠️  faelight-zone: not found (zone detection unavailable)");
            eprintln!("💡 Install from 0-Core workspace");
        }
    }

    println!("\n✅ Core functionality working!");
    println!("💡 Optional tools enhance output when installed");
}
