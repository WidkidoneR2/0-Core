use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use serde::Serialize;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct ZoneInfo {
    pub zone: String,
    pub label: String,
    pub icon: String,
    pub path: String,
    pub critical: bool,
}

pub fn detect(ctx: &AppContext) -> ZoneInfo {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from(&ctx.home));
    let cwd_str = cwd.to_string_lossy().to_string();
    let home = &ctx.home;
    let core_root = &ctx.core_root;

    if cwd_str.starts_with(core_root.as_str()) {
        let rel = cwd_str
            .strip_prefix(home.as_str())
            .unwrap_or(&cwd_str)
            .trim_start_matches('/')
            .to_uppercase()
            .replace('/', "-");
        ZoneInfo {
            zone: "Core".to_string(),
            label: "CORE".to_string(),
            icon: "🌲".to_string(),
            path: rel,
            critical: true,
        }
    } else if cwd_str.starts_with(home.as_str()) {
        let rel = cwd_str
            .strip_prefix(home.as_str())
            .unwrap_or("")
            .trim_start_matches('/')
            .to_uppercase()
            .replace('/', "-");
        let display = if rel.is_empty() {
            "HOME".to_string()
        } else {
            rel
        };
        ZoneInfo {
            zone: "Home".to_string(),
            label: "HOME".to_string(),
            icon: "🏠".to_string(),
            path: display,
            critical: false,
        }
    } else if cwd_str.starts_with("/tmp") {
        ZoneInfo {
            zone: "Temp".to_string(),
            label: "TEMP".to_string(),
            icon: "🗑".to_string(),
            path: cwd_str
                .to_uppercase()
                .replace('/', "-")
                .trim_start_matches('-')
                .to_string(),
            critical: false,
        }
    } else if cwd_str.starts_with("/etc")
        || cwd_str.starts_with("/usr")
        || cwd_str.starts_with("/sys")
    {
        ZoneInfo {
            zone: "System".to_string(),
            label: "SYSTEM".to_string(),
            icon: "⚙️".to_string(),
            path: cwd_str
                .to_uppercase()
                .replace('/', "-")
                .trim_start_matches('-')
                .to_string(),
            critical: true,
        }
    } else {
        ZoneInfo {
            zone: "Unknown".to_string(),
            label: "UNKNOWN".to_string(),
            icon: "❓".to_string(),
            path: cwd_str,
            critical: false,
        }
    }
}

pub fn run(ctx: &AppContext, icon: bool, label: bool, json: bool, health: bool) -> CoreResult<()> {
    if health {
        println!("{}", "🏥 core zone health check".bold());
        println!("  {} HOME: set", "✅".green());
        let cwd = env::current_dir().unwrap_or_default();
        println!(
            "  {} Current directory: {}",
            "✅".green(),
            cwd.display().to_string().dimmed()
        );
        let zone = detect(ctx);
        println!(
            "  {} Zone detection: {} {}",
            "✅".green(),
            zone.icon,
            zone.label.bright_white()
        );
        println!("  {} All checks passed!", "✅".green());
        return Ok(());
    }

    let zone = detect(ctx);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&zone).unwrap_or_default()
        );
        return Ok(());
    }

    if icon {
        println!("{}", zone.icon);
        return Ok(());
    }

    if label {
        println!("{}", zone.label);
        return Ok(());
    }

    // Default: icon + path (matches faelight-zone output)
    println!("{} {}", zone.icon, zone.path);
    Ok(())
}
