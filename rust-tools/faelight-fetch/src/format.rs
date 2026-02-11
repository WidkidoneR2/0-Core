use crate::state::SystemState;

pub fn print_output(state: &SystemState) {
    // Header with box
    println!("╭─────────────────────────────────╮");
    println!("│ 🌲 Faelight Forest v{:<11} │", state.version);
    println!("╰─────────────────────────────────╯");
    println!();

    // Right-aligned labels, left-aligned values
    let label_width = 10;

    println!(
        "{:>width$}  {} {}",
        "zone",
        state.zone_icon,
        state.zone,
        width = label_width
    );
    println!(
        "{:>width$}  {}",
        "profile",
        state.profile,
        width = label_width
    );
    println!(
        "{:>width$}  {} {}",
        "core",
        state.core_icon,
        state.core_state,
        width = label_width
    );
    println!(
        "{:>width$}  {} {}",
        "health",
        state.health_icon,
        state.health,
        width = label_width
    );
    println!();
    println!("{:>width$}  {}", "wm", state.wm, width = label_width);
    println!("{:>width$}  {}", "term", state.term, width = label_width);
    println!("{:>width$}  {}", "shell", state.shell, width = label_width);
    println!(
        "{:>width$}  {}",
        "kernel",
        state.kernel,
        width = label_width
    );
    println!(
        "{:>width$}  {}",
        "uptime",
        state.uptime,
        width = label_width
    );
    println!(
        "{:>width$}  {}",
        "host",
        state.hostname,
        width = label_width
    );
}
