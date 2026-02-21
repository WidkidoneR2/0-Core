#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use std::process::Command;

pub fn palette(_ctx: &AppContext, dmenu: bool, prompt: Option<&str>) -> CoreResult<()> {
    let mut cmd = Command::new("faelight-palette");
    if dmenu {
        cmd.arg("--dmenu");
    }
    if let Some(p) = prompt {
        cmd.args(["--prompt", p]);
    }
    cmd.status()?;
    Ok(())
}

pub fn dmenu(
    _ctx: &AppContext,
    subcmd: Option<&str>,
    prompt: Option<&str>,
    multi: bool,
) -> CoreResult<()> {
    let mut cmd = Command::new("faelight-dmenu");
    if let Some(sub) = subcmd {
        cmd.arg(sub);
    }
    if let Some(p) = prompt {
        cmd.args(["--prompt", p]);
    }
    if multi {
        cmd.arg("--multi");
    }
    cmd.status()?;
    Ok(())
}

pub fn launcher(_ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    Command::new("faelight-launcher").args(args).status()?;
    Ok(())
}
