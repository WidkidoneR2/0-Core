#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn get_version(ctx: &AppContext, package: Option<&str>) -> CoreResult<()> {
    ctx.capabilities.require(
        "release",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;
    let tools_dir = PathBuf::from(&ctx.core_root).join("rust-tools");

    match package {
        None => {
            // Show system version
            let version = fs::read_to_string(PathBuf::from(&ctx.core_root).join("00-meta/VERSION"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            println!("{}", version);
        }
        Some(pkg) => {
            // Read version from Cargo.toml
            let cargo_toml = tools_dir.join(pkg).join("Cargo.toml");
            if !cargo_toml.exists() {
                println!("unknown");
                return Ok(());
            }
            let content = fs::read_to_string(&cargo_toml).unwrap_or_default();
            for line in content.lines() {
                if let Some(v) = line.strip_prefix("version = ") {
                    let version = v.trim().trim_matches('"');
                    println!("{}", version);
                    return Ok(());
                }
            }
            println!("unknown");
        }
    }
    Ok(())
}

pub fn bump_tool(_ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    // Call v1 binary directly via full path to avoid wrapper loop
    let status = Command::new("bump-tool-version").args(args).status()?;
    if !status.success() {
        println!("  {} bump-tool-version failed", "✗".bright_red());
    }
    Ok(())
}

pub fn bump_system(_ctx: &AppContext, dry_run: bool) -> CoreResult<()> {
    let mut cmd = Command::new("bump-system-version");
    if dry_run {
        cmd.arg("--dry-run");
    }
    let status = cmd.status()?;
    if !status.success() {
        println!("  {} bump-system-version failed", "✗".bright_red());
    }
    Ok(())
}
