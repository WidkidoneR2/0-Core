#![allow(dead_code)]
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use std::process::Command;

pub fn update(_ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    Command::new("faelight-update").args(args).status()?;
    Ok(())
}

pub fn safe(_ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    Command::new("safe-update").args(args).status()?;
    Ok(())
}
