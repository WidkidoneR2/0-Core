use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use std::process::Command;

pub fn update(ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    ctx.capabilities.require(
        "update",
        &[
            Capability::FilesystemReadHome,
            Capability::ExecutePacman,
            Capability::SpawnProcess,
        ],
    )?;
    let bin = format!("{}/target/release/faelight-update", ctx.core_root);
    let status = Command::new(&bin).args(args).status()?;

    // Event Ledger
    let writer = crate::runtime::EventWriter::new(&ctx.runtime.db);
    writer.write(
        "update",
        "run",
        "core update",
        if status.success() { "ok" } else { "warn" },
        None,
    );

    Ok(())
}

pub fn safe(ctx: &AppContext, args: &[String]) -> CoreResult<()> {
    let bin = format!("{}/scripts/safe-update", ctx.core_root);
    Command::new(&bin).args(args).status()?;
    Ok(())
}
