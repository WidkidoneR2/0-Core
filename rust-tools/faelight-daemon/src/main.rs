mod protocol;
mod daemon;

use daemon::Daemon;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = "/tmp/faelight-daemon.sock".to_string();
    
    println!("🌲 Starting faelight-daemon v0.1.0");
    
    let daemon = Daemon::new(socket_path);
    daemon.run().await?;
    
    Ok(())
}
