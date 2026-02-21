#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use std::process::Command;

pub fn view(_ctx: &AppContext, active: bool, summary: bool, json: bool) -> CoreResult<()> {
    let mut args = vec![];
    if active {
        args.push("--active");
    }
    if summary {
        args.push("--summary");
    }
    if json {
        args.push("--json");
    }

    let status = Command::new("workspace-view").args(&args).status()?;
    if !status.success() {
        println!("  {} workspace-view failed", "✗".bright_red());
    }
    Ok(())
}

pub fn recent(_ctx: &AppContext, range: &str, limit: u32, full_paths: bool) -> CoreResult<()> {
    let mut args = vec![range.to_string()];
    args.push(format!("--limit={}", limit));
    if full_paths {
        args.push("--full-paths".to_string());
    }

    let status = Command::new("recent-files").args(&args).status()?;
    if !status.success() {
        println!("  {} recent-files failed", "✗".bright_red());
    }
    Ok(())
}

pub fn fm(_ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    // Full TUI — delegate entirely to faelight-fm
    let status = Command::new("faelight-fm").args(args).status()?;
    if !status.success() {
        println!("  {} faelight-fm failed", "✗".bright_red());
    }
    Ok(())
}
