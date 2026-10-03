use std::process::Command;

pub fn cleanup_cargo_cache() -> std::io::Result<()> {
    match Command::new("cargo-cache").arg("-a").status() {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("   ⚠️  cargo-cache not installed, skipping");
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
