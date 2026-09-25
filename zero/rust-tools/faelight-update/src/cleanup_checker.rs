use std::process::Command;

pub fn cleanup_cargo_cache() -> std::io::Result<()> {
    Command::new("cargo-cache").arg("-a").status()?;
    Ok(())
}
