#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use std::fs;

pub fn get_version(ctx: &AppContext, package: Option<&str>) -> CoreResult<()> {
    ctx.capabilities.require(
        "release",
        &[
            Capability::FilesystemReadHome,
            Capability::FilesystemWriteHome,
        ],
    )?;
    let tools_dir = zero_core::paths::tools_dir();

    match package {
        None => {
            // Show system version
            let version = fs::read_to_string(zero_core::paths::version_file())
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
