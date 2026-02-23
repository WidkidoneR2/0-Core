#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::process::Command;

pub fn lock(_ctx: &AppContext) -> CoreResult<()> {
    _ctx.capabilities
        .require("lock", &[Capability::ControlSway])?;
    let status = Command::new("swaylock").status()?;
    if !status.success() {
        println!("  {} swaylock failed", "✗".bright_red());
    }
    Ok(())
}

pub fn health(_ctx: &AppContext) -> CoreResult<()> {
    // Check swaylock is available
    let available = Command::new("which")
        .arg("swaylock")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    println!("{}", "🔒 core lock health".bold());
    if available {
        println!("  {} swaylock available", "✅".green());
        println!("  {} All checks passed!", "✅".green());
    } else {
        println!("  {} swaylock not found", "✗".bright_red());
    }
    Ok(())
}
