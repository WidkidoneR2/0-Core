use std::process::Command;

pub fn cleanup_cargo_cache() -> std::io::Result<()> {
    Command::new("cargo-cache").arg("-a").status()?;
    Ok(())
}

pub fn cleanup_pacman_cache() -> std::io::Result<()> {
    Command::new("sudo")
        .args(["pacman", "-Scc", "--noconfirm"])
        .status()?;
    Ok(())
}
