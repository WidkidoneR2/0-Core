//! faelight-daemon v2.1.0 - Background daemon for Faelight Forest
//! 🌲 LEGENDARY EDITION

mod daemon;
mod dbus;
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
    /// Socket path (default: daemon.sock under paths::faelight_state_dir)
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
        // A HEALTH CHECK THAT CANNOT FAIL IS NOT A HEALTH CHECK. This printed Daemon ready
        // and returned, unconditionally, without looking at a socket or a process. Measured
        // 2026-09-02: it said ready on a machine where no faelight process was running at
        // all and pgrep returned nothing.
        //
        // Same class as the doctor checks INT-222 catalogues -- the answer was decided at
        // compile time. CONNECTING is the question, not existence: a socket file outlives
        // the process that made it, so a stat would be the same lie one step further on.
        let sock = cli.socket.clone().unwrap_or_else(|| {
            paths::faelight_state_dir()
                .join("daemon.sock")
                .display()
                .to_string()
        });
        match std::os::unix::net::UnixStream::connect(&sock) {
            Ok(_) => {
                println!("{} faelight-daemon: responding on {}", "OK".green(), sock);
                return Ok(());
            }
            Err(e) => {
                println!(
                    "{} faelight-daemon: not responding on {} -- {}",
                    "DOWN".red(),
                    sock,
                    e
                );
                std::process::exit(1);
            }
        }
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
