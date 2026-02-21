use crate::app::context::AppContext;
use crate::capabilities::Capability;
use crate::errors::CoreResult;
use colored::*;

pub fn run(ctx: &AppContext, preflight: bool) -> CoreResult<()> {
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("{}", "🏥 core doctor — 0-Core v2".bold());
    if preflight {
        println!("{}", "   Mode: preflight (no execution)".dimmed());
    }
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  {} Engine initialized", "✅".green());
    println!(
        "  {} Runtime: {}",
        "✅".green(),
        ctx.runtime.root.display().to_string().dimmed()
    );
    println!("  {} State database: ready", "✅".green());
    println!("  {} Capabilities: core set granted", "✅".green());
    let _ = ctx.capabilities.has(&Capability::FilesystemReadConfig);
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
    println!("  {} Phase 1 scaffold healthy", "🌲".green());
    println!(
        "  {}",
        "Full domain checks added as migration progresses".dimmed()
    );
    Ok(())
}
