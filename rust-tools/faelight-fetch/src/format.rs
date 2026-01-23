use crate::state::SystemState;

pub fn print_output(state: &SystemState) {
    println!("🌲 Faelight Forest v{}", state.version);
    println!("────────────────────────");
    println!("profile   {}", state.profile);
    println!("core      {} {}", state.core_icon, state.core_state);
    println!("health    {} {}", state.health_icon, state.health);
    println!("wm        {}", state.wm);
    println!("term      {}", state.term);
    println!("shell     {}", state.shell);
    println!("kernel    {}", state.kernel);
    println!("uptime    {}", state.uptime);
    println!("host      {}", state.hostname);
}
