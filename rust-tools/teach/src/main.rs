use std::process::{self, Command, Stdio};

fn main() {
    // Check for gum
    if Command::new("which").arg("gum").output().map(|o| !o.status.success()).unwrap_or(true) {
        eprintln!("Error: gum is required. Install with: paru -S gum");
        process::exit(1);
    }
    
    main_menu();
}

fn gum_style(text: &str, fg: u8) {
    Command::new("gum")
        .args(["style", "--foreground", &fg.to_string(), text])
        .status()
        .ok();
}

fn gum_section(title: &str) {
    Command::new("clear").status().ok();
    Command::new("gum")
        .args(["style", "--border", "double", "--border-foreground", "10", 
               "--padding", "1 2", "--margin", "1", title])
        .status()
        .ok();
    println!();
}

fn pause() -> bool {
    Command::new("gum")
        .args(["confirm", "Continue?", "--affirmative=Next →", "--negative=Exit"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn lesson_welcome() {
    gum_section("🌲 Welcome to 0-Core");
    gum_style("The Faelight Forest Philosophy", 10);
    println!();
    println!("0-Core is not just dotfiles. It's a philosophy-driven system.");
    println!();
    println!("Core principles:");
    gum_style("  • Manual control over automation", 11);
    gum_style("  • Intent over convention", 11);
    gum_style("  • Understanding over convenience", 11);
    gum_style("  • Recovery over perfection", 11);
    println!();
    println!("You control when things happen. Nothing runs without your say.");
    println!();
    if !pause() { process::exit(0); }
}

fn lesson_structure() {
    gum_section("📁 Directory Structure");
    println!("Your system is organized by purpose:");
    println!();
    gum_style("  ~/0-core/      → Configuration (this repo)", 14);
    gum_style("  ~/1-src/       → Source code & projects", 14);
    gum_style("  ~/2-projects/  → Active work", 14);
    gum_style("  ~/3-archive/   → Completed/archived", 14);
    gum_style("  ~/4-media/     → Media files", 14);
    println!();
    println!("Inside 0-core:");
    println!();
    gum_style("  scripts/       → Your tools (dot-doctor, profile, etc.)", 10);
    gum_style("  profiles/      → System profiles (gaming, work, etc.)", 10);
    gum_style("  INTENT/        → Decision ledger & roadmap", 10);
    gum_style("  docs/          → Documentation", 10);
    println!();
    if !pause() { process::exit(0); }
}

fn lesson_tools() {
    gum_section("🔧 Core Tools");
    println!("Your daily drivers:");
    println!();
    gum_style("dot-doctor", 11);
    println!("  Health check for your system. Run daily.");
    println!("  Try: dot-doctor");
    println!();
    gum_style("core-diff", 11);
    println!("  See what changed in your config.");
    println!("  Try: core-diff");
    println!();
    gum_style("profile", 11);
    println!("  Switch system modes (gaming, work, low-power).");
    println!("  Try: profile list");
    println!();
    gum_style("intent", 11);
    println!("  Track decisions and future plans.");
    println!("  Try: intent list");
    println!();
    gum_style("safe-update", 11);
    println!("  Update system safely with snapshots.");
    println!("  Try: safe-update");
    println!();
    if !pause() { process::exit(0); }
}

fn lesson_workflow() {
    gum_section("📅 Daily Workflow");
    println!("Morning routine (2 minutes):");
    println!();
    gum_style("  1. dot-doctor      → Check system health", 14);
    gum_style("  2. core-diff       → See any config changes", 14);
    gum_style("  3. profile status  → Confirm your mode", 14);
    println!();
    println!("Before gaming:");
    gum_style("  profile gaming", 10);
    println!();
    println!("Before work:");
    gum_style("  profile work", 10);
    println!();
    println!("Back to normal:");
    gum_style("  profile default", 10);
    println!();
    println!("Weekly:");
    gum_style("  safe-update        → Update packages safely", 14);
    println!();
    if !pause() { process::exit(0); }
}

fn lesson_profiles() {
    gum_section("🎮 System Profiles");
    println!("Switch your entire system state with one command.");
    println!();
    println!("Available profiles:");
    println!();
    gum_style("  🏠 default    → Balanced daily driver", 10);
    gum_style("  🎮 gaming     → GPU performance, DnD on", 10);
    gum_style("  💼 work       → VPN on, focus mode", 10);
    gum_style("  🔋 low-power  → Battery saver", 10);
    println!();
    println!("Commands:");
    gum_style("  profile list      → See all profiles", 14);
    gum_style("  profile gaming    → Switch to gaming", 14);
    gum_style("  profile status    → Current state", 14);
    gum_style("  profile default   → Back to baseline", 14);
    println!();
    println!("Your prompt shows active profile (hidden when default).");
    println!();
    if !pause() { process::exit(0); }
}

fn lesson_intent() {
    gum_section("📜 Intent Ledger");
    println!("Track decisions, not just code.");
    println!();
    println!("Categories:");
    gum_style("  decisions/    → Major architectural choices", 11);
    gum_style("  experiments/  → Things you tried", 11);
    gum_style("  philosophy/   → Core beliefs", 11);
    gum_style("  future/       → Roadmap & planned features", 11);
    gum_style("  incidents/    → Failures & lessons learned", 11);
    println!();
    println!("Commands:");
    gum_style("  intent list       → See all intents", 14);
    gum_style("  intent show 001   → View specific intent", 14);
    gum_style("  intent add        → Create new intent", 14);
    gum_style("  intent search     → Find by keyword", 14);
    println!();
    println!("Every decision has a 'why'. Future you will thank you.");
    println!();
    if !pause() { process::exit(0); }
}

fn lesson_customize() {
    gum_section("🎨 Make It Yours");
    println!("Create your own profile:");
    gum_style("  nvim ~/0-core/profiles/myprofile.profile", 14);
    println!();
    println!("Profile format:");
    gum_style("  [activate]", 10);
    gum_style("  # Commands when profile activates", 10);
    gum_style("  powerprofilesctl set performance", 10);
    gum_style("  ", 10);
    gum_style("  [deactivate]", 10);
    gum_style("  # Commands when switching away", 10);
    gum_style("  powerprofilesctl set balanced", 10);
    println!();
    println!("Add new tools:");
    gum_style("  1. Create script in ~/0-core/scripts/", 14);
    gum_style("  2. Make executable: chmod +x", 14);
    gum_style("  3. Add intent: intent add", 14);
    println!();
    if !pause() { process::exit(0); }
}

fn lesson_aliases() {
    gum_section("⚡ Useful Aliases");
    println!("Quick commands built into your shell:");
    println!();
    gum_style("Navigation:", 11);
    gum_style("  ..          → cd ..", 14);
    gum_style("  ...         → cd ../..", 14);
    gum_style("  core        → cd ~/0-core", 14);
    println!();
    gum_style("System:", 11);
    gum_style("  doctor      → dot-doctor", 14);
    gum_style("  health      → dot-doctor", 14);
    gum_style("  keys        → keyscan", 14);
    println!();
    gum_style("Core Protection:", 11);
    gum_style("  lock-core   → Lock 0-core from changes", 14);
    gum_style("  unlock-core → Unlock for editing", 14);
    gum_style("  edit-core   → Unlock + open in editor", 14);
    println!();
    gum_style("Git:", 11);
    gum_style("  gs          → git status", 14);
    gum_style("  ga          → git add", 14);
    gum_style("  gc          → git commit", 14);
    gum_style("  gp          → git push", 14);
    println!();
    gum_style("Files:", 11);
    gum_style("  ls          → exa with icons", 14);
    gum_style("  ll          → exa long format", 14);
    gum_style("  tree        → exa tree view", 14);
    println!();
    println!("See all aliases: alias | less");
    println!();
    if !pause() { process::exit(0); }
}

fn lesson_complete() {
    gum_section("🌲 You're Ready");
    println!("You now know:");
    gum_style("  ✅ The philosophy", 10);
    gum_style("  ✅ Directory structure", 10);
    gum_style("  ✅ Core tools", 10);
    gum_style("  ✅ Daily workflow", 10);
    gum_style("  ✅ Profiles", 10);
    gum_style("  ✅ Intent ledger", 10);
    gum_style("  ✅ Customization", 10);
    println!();
    println!("Quick reference:");
    gum_style("  dot-doctor     → Health check", 14);
    gum_style("  profile list   → System modes", 14);
    gum_style("  intent list    → Roadmap", 14);
    gum_style("  safe-update    → Update system", 14);
    println!();
    gum_style("Welcome to the Faelight Forest. 🌲", 11);
    println!();
}

fn main_menu() {
    loop {
        Command::new("clear").status().ok();
        Command::new("gum")
            .args(["style", "--border", "double", "--border-foreground", "10",
                   "--padding", "1 2", "--margin", "1", "🌲 0-Core Teaching Mode"])
            .status()
            .ok();
        println!();
        
        let output = Command::new("gum")
            .args(["choose",
                "Start from beginning",
                "1. Welcome & Philosophy",
                "2. Directory Structure",
                "3. Core Tools",
                "4. Daily Workflow",
                "5. System Profiles",
                "6. Intent Ledger",
                "7. Customization",
                "8. Useful Aliases",
                "Exit"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to run gum");
        
        let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        match choice.as_str() {
            "Start from beginning" => {
                lesson_welcome();
                lesson_structure();
                lesson_tools();
                lesson_workflow();
                lesson_profiles();
                lesson_intent();
                lesson_customize();
                lesson_aliases();
                lesson_complete();
            }
            "1. Welcome & Philosophy" => lesson_welcome(),
            "2. Directory Structure" => lesson_structure(),
            "3. Core Tools" => lesson_tools(),
            "4. Daily Workflow" => lesson_workflow(),
            "5. System Profiles" => lesson_profiles(),
            "6. Intent Ledger" => lesson_intent(),
            "7. Customization" => lesson_customize(),
            "8. Useful Aliases" => lesson_aliases(),
            "Exit" | "" => break,
            _ => {}
        }
    }
}
