//! Test client for faelight-daemon
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[path = "../protocol/mod.rs"]
mod protocol;

use protocol::{Command, Message, MessagePayload, Response};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Connecting to faelight-daemon...");

    let socket_path = std::env::var("HOME")
        .map(|h| format!("{}/.local/state/0-core/daemon.sock", h))
        .unwrap_or_else(|_| "/tmp/faelight-daemon.sock".to_string());

    let stream = UnixStream::connect(&socket_path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    println!("✅ Connected!");

    // Test 1: Ping
    println!("\n📡 Test 1: Ping");
    send_command(&mut writer, 1, Command::Ping).await?;
    let response = read_response(&mut reader).await?;
    println!("Response: {:?}", response);

    // Test 2: Get entries
    println!("\n📂 Test 2: Get entries for /tmp");
    send_command(
        &mut writer,
        2,
        Command::GetEntries {
            path: "/tmp".to_string(),
        },
    )
    .await?;
    let response = read_response(&mut reader).await?;
    if let Response::Entries { entries } = response {
        println!("Found {} entries:", entries.len());
        for entry in entries.iter().take(5) {
            println!(
                "  {} {}",
                if entry.is_dir { "📁" } else { "📄" },
                entry.name
            );
        }
    }

    // Test 3: Preview
    println!("\n📖 Test 3: Preview /etc/hostname");
    send_command(
        &mut writer,
        3,
        Command::Preview {
            path: "/etc/hostname".to_string(),
        },
    )
    .await?;
    let response = read_response(&mut reader).await?;
    if let Response::Preview { content } = response {
        println!("Content: {}", content.trim());
    }

    println!("\n✨ All tests passed!");
    Ok(())
}

async fn send_command(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    id: u64,
    command: Command,
) -> Result<(), Box<dyn std::error::Error>> {
    let msg = Message {
        id,
        payload: MessagePayload::Command(command),
    };
    let json = serde_json::to_string(&msg)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

async fn read_response(
    reader: &mut BufReader<tokio::net::unix::OwnedReadHalf>,
) -> Result<Response, Box<dyn std::error::Error>> {
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let msg: Message = serde_json::from_str(&line)?;

    match msg.payload {
        MessagePayload::Response(resp) => Ok(resp),
        _ => Err("Expected response".into()),
    }
}
