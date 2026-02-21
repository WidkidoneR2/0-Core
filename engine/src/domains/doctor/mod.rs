#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use std::process::Command;

pub fn run(_ctx: &AppContext, preflight: bool) -> CoreResult<()> {
    // Phase 2: delegate to dot-doctor v1
    let mut cmd = Command::new("dot-doctor");
    if preflight {
        cmd.arg("--preflight");
    }
    cmd.status()?;
    Ok(())
}

pub fn aliases(_ctx: &AppContext, subcmd: Option<&str>) -> CoreResult<()> {
    let mut cmd = Command::new("alias-audit");
    if let Some(sub) = subcmd {
        cmd.arg(sub);
    }
    cmd.status()?;
    Ok(())
}

pub fn entropy(_ctx: &AppContext, baseline: bool, trends: bool, json: bool) -> CoreResult<()> {
    let mut cmd = Command::new("entropy-check");
    if baseline {
        cmd.arg("--baseline");
    }
    if trends {
        cmd.arg("--trends");
    }
    if json {
        cmd.arg("--json");
    }
    cmd.status()?;
    Ok(())
}

pub fn bins(_ctx: &AppContext, subcmd: Option<&str>) -> CoreResult<()> {
    let mut cmd = Command::new("bin-doctor");
    if let Some(sub) = subcmd {
        cmd.arg(sub);
    }
    cmd.status()?;
    Ok(())
}
