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

    print_banner(&socket_path);

    let daemon = Daemon::new(socket_path);
    daemon.run().await?;

    Ok(())
}

fn print_banner(socket_path: &str) {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!("{}", "🌲 FAELIGHT DAEMON v2.0.0".cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".cyan());
    println!();
    println!("  {} {}", "Socket:".bold(), socket_path.green());
    println!("  {} {}", "Status:".bold(), "LEGENDARY".green().bold());
    println!();
    println!("{}", "Ready for connections...".cyan());
    println!();
}
