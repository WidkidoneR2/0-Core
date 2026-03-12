use crate::state::SystemState;
use colored::*;

pub fn print_output(state: &SystemState) {
    // Header
    println!(
        "{}",
        format!("╭─ 🌲 Faelight Forest {} ─╮", state.version)
            .cyan()
            .bold()
    );
    println!();

    let lw = 10usize;

    // 0-Core identity
    section("system", lw);
    row("zone", &format!("{} {}", state.zone_icon, state.zone), lw);
    row("host", &state.hostname, lw);
    row("profile", &state.profile, lw);
    row(
        "core",
        &format!("{} {}", state.core_icon, state.core_state),
        lw,
    );
    row(
        "health",
        &format!("{} {}", state.health_icon, state.health),
        lw,
    );
    println!();

    // Environment
    section("env", lw);
    row("wm", &state.wm, lw);
    row("term", &state.term, lw);
    row("shell", &state.shell, lw);
    row("kernel", &state.kernel, lw);
    row("rust", &state.rust_ver, lw);
    row("uptime", &state.uptime, lw);
    println!();

    // Resources
    section("resources", lw);
    row("cpu", &state.cpu_usage, lw);
    row("memory", &state.memory, lw);
    row("disk", &state.disk, lw);
    println!();

    // 0-Core stats
    section("0-core", lw);
    row("commits", &state.commits, lw);
    row("tools", &state.tools, lw);
}

fn section(name: &str, width: usize) {
    println!("{:>w$}", name.dimmed().bold(), w = width);
}

fn row(label: &str, value: &str, width: usize) {
    println!("{:>w$}  {}", label.dimmed(), value.white(), w = width);
}
