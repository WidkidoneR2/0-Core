use crate::app::context::AppContext;
use crate::errors::CoreResult;
use std::process::Command;

pub fn palette(ctx: &AppContext, dmenu: bool, _prompt: Option<&str>) -> CoreResult<()> {
    let bin = format!("{}/scripts/faelight-palette", ctx.core_root);
    let mut cmd = Command::new(&bin);
    if dmenu {
        cmd.arg("--dmenu");
    }
    cmd.status()?;
    Ok(())
}

pub fn dmenu(
    ctx: &AppContext,
    _subcmd: Option<&str>,
    _prompt: Option<&str>,
    _multi: bool,
) -> CoreResult<()> {
    let bin = format!("{}/scripts/faelight-palette", ctx.core_root);
    Command::new(&bin).arg("--dmenu").status()?;
    Ok(())
}

pub fn launcher(ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    let bin = format!("{}/scripts/faelight-palette", ctx.core_root);
    Command::new(&bin).args(args).status()?;
    Ok(())
}
