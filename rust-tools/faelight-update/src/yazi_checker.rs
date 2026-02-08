
/// Check for Yazi package updates
pub fn check_yazi_packages() -> Vec<String> {
    // Yazi plugin management doesn't have a stable update command yet
    Vec::new()
}

/// Update Yazi packages
pub fn update_yazi() -> anyhow::Result<()> {
    println!("   ⏭️  Yazi plugin updates not yet supported");
    Ok(())
}
