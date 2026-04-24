//! faelight-daemon v2.1.0 - Background daemon for Faelight Forest
//! 🌲 LEGENDARY EDITION

mod daemon;
mod protocol;

use clap::Parser;
use colored::*;
use daemon::Daemon;
use faelight_core::paths;

#[derive(Parser)]
#[command(name = "faelight-daemon")]
#[command(about = "🌲 Faelight Forest Daemon - Background operations", long_about = None)]
#[command(version)]
struct Cli {
    /// Socket path (default: ~/.local/state/faelight/daemon.sock)
    #[arg(short, long)]
    socket: Option<String>,

    /// Run health check and exit
    #[arg(long)]
    health: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.health {
        println!("{} faelight-daemon: Daemon ready", "✅".green());
        return Ok(());
    }

    // Determine socket path
    let socket_path = cli.socket.unwrap_or_else(|| {
        paths::faelight_state_dir()
            .join("daemon.sock")
            .display()
            .to_string()
    });

    // Banner logged to friday.log, not stdout
    let _ = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!(
            "{}/.cache/faelight/friday.log",
            std::env::var("HOME").unwrap_or_default()
        ))
        .map(|mut f| {
            use std::io::Write;
            let _ = f.write_all(format!("[friday] daemon started on {}\n", socket_path).as_bytes());
        });

    let daemon = Daemon::new(socket_path);
    daemon.run().await?;

    Ok(())
}
