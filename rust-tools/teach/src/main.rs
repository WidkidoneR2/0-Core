//! teach v1.0.0 - Interactive Learning System
//! 🌲 Faelight Forest

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command, Stdio};

const VERSION: &str = "1.0.0";

#[derive(Serialize, Deserialize, Default)]
struct LearningProgress {
    lessons_completed: Vec<usize>,
    quiz_scores: Vec<usize>,
    best_quiz_score: usize,
    achievements: Vec<String>,
    last_studied: String,
    total_sessions: usize,
}

impl LearningProgress {
    fn load() -> Self {
        let path = Self::get_path();
        
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }
    
    fn save(&self) {
        let path = Self::get_path();
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        
        if let Ok(json) = serde_json::to_string_pretty(&self) {
            fs::write(&path, json).ok();
        }
    }
    
    fn get_path() -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(&home).join(".local/state/faelight/learning.json")
    }
    
    fn mark_lesson_complete(&mut self, lesson: usize) {
        if !self.lessons_completed.contains(&lesson) {
            self.lessons_completed.push(lesson);
            self.lessons_completed.sort();
            self.check_new_achievements();
        }
    }
    
    fn is_lesson_complete(&self, lesson: usize) -> bool {
        self.lessons_completed.contains(&lesson)
    }
    
    fn record_quiz_score(&mut self, score: usize) {
        self.quiz_scores.push(score);
        if score > self.best_quiz_score {
            self.best_quiz_score = score;
        }
        self.check_new_achievements();
    }
    
    fn check_new_achievements(&mut self) {
        let mut new_achievements = Vec::new();
        
        // Individual lesson achievements
        if self.is_lesson_complete(1) && !self.achievements.contains(&"First Steps".to_string()) {
            new_achievements.push("🌱 First Steps");
        }
        if self.is_lesson_complete(2) && !self.achievements.contains(&"Directory Master".to_string()) {
            new_achievements.push("🌿 Directory Master");
        }
        if self.is_lesson_complete(3) && !self.achievements.contains(&"Tool Proficient".to_string()) {
            new_achievements.push("🔧 Tool Proficient");
        }
        if self.is_lesson_complete(4) && !self.achievements.contains(&"Workflow Expert".to_string()) {
            new_achievements.push("📅 Workflow Expert");
        }
        if self.is_lesson_complete(5) && !self.achievements.contains(&"Profile Pro".to_string()) {
            new_achievements.push("🎮 Profile Pro");
        }
        if self.is_lesson_complete(6) && !self.achievements.contains(&"Intent Scholar".to_string()) {
            new_achievements.push("📜 Intent Scholar");
        }
        if self.is_lesson_complete(7) && !self.achievements.contains(&"Customization Guru".to_string()) {
            new_achievements.push("🎨 Customization Guru");
        }
        if self.is_lesson_complete(8) && !self.achievements.contains(&"Alias Ninja".to_string()) {
            new_achievements.push("⚡ Alias Ninja");
        }
        
        // Full course achievement
        if self.lessons_completed.len() >= 8 && !self.achievements.contains(&"Full Course".to_string()) {
            new_achievements.push("🎓 Full Course");
        }
        
        // Quiz achievements
        if self.best_quiz_score == 10 && !self.achievements.contains(&"Forest Scholar".to_string()) {
            new_achievements.push("🏆 Forest Scholar");
        }
        
        // Master achievement
        if self.lessons_completed.len() >= 8 && self.best_quiz_score == 10 && !self.achievements.contains(&"Faelight Master".to_string()) {
            new_achievements.push("🌲 Faelight Master");
        }
        
        for achievement in new_achievements {
            self.achievements.push(achievement.to_string());
            show_achievement_unlock(achievement);
        }
    }
    
    fn completion_percentage(&self) -> usize {
        (self.lessons_completed.len() * 100) / 8
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Check for flags
    if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("teach v{}", VERSION);
        return;
    }
    
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        show_help();
        return;
    }
    
    if args.contains(&"--health".to_string()) {
        run_health_check();
        return;
    }
    
    if args.contains(&"--stats".to_string()) {
        show_statistics();
        return;
    }
    
    if args.contains(&"--tips".to_string()) {
        show_daily_wisdom();
        return;
    }
    
    if args.contains(&"--quiz".to_string()) {
        run_quiz();
        return;
    }
    
    if args.contains(&"--reset".to_string()) {
        reset_progress();
        return;
    }
    
    if args.contains(&"--begin".to_string()) {
        run_full_course();
        return;
    }
    
    if args.contains(&"--random".to_string()) {
        run_random_lesson();
        return;
    }
    
    // Check for gum
    if !check_command_exists("gum") {
        eprintln!("Error: gum is required. Install with: yay -S gum");
        process::exit(1);
    }
    
    // Update session count
    let mut progress = LearningProgress::load();
    progress.total_sessions += 1;
    progress.last_studied = get_timestamp();
    progress.save();
    
    main_menu();
}

fn check_command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn get_timestamp() -> String {
    Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
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

fn pause(lesson_num: Option<usize>) -> bool {
    let result = Command::new("gum")
        .args(["confirm", "Continue?", "--affirmative=Next →", "--negative=Exit"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    
    if result {
        // Mark lesson as complete
        if let Some(num) = lesson_num {
            let mut progress = LearningProgress::load();
            progress.mark_lesson_complete(num);
            progress.save();
        }
    }
    
    result
}

fn show_achievement_unlock(achievement: &str) {
    println!();
    gum_style("🎉 ACHIEVEMENT UNLOCKED! 🎉", 11);
    gum_style(achievement, 10);
    println!();
    std::thread::sleep(std::time::Duration::from_secs(2));
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
    println!("This is {}intentional stewardship{}.", "\x1b[1;32m", "\x1b[0m");
    println!();
    if !pause(Some(1)) { process::exit(0); }
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
    gum_style("  rust-tools/    → Rust tools (workspace-view, safe-update, etc.)", 10);
    gum_style("  scripts/       → Compiled binaries", 10);
    gum_style("  packages/      → Stow-managed dotfiles", 10);
    gum_style("  INTENT/        → Decision ledger & roadmap", 10);
    gum_style("  docs/          → Documentation", 10);
    println!();
    println!("The numbers enforce order. 0 comes before 1. Always.");
    println!();
    if !pause(Some(2)) { process::exit(0); }
}

fn lesson_tools() {
    gum_section("🔧 Core Tools");
    println!("Your daily drivers:");
    println!();
    gum_style("doctor (dot-doctor)", 11);
    println!("  Health check for your system. Run daily.");
    println!("  Try: doctor");
    println!();
    gum_style("workspace-view v1.0.0", 11);
    println!("  X-ray vision for Sway workspaces. NEW: --watch mode!");
    println!("  Try: workspace-view --active");
    println!();
    gum_style("safe-update v1.0.0", 11);
    println!("  Update system safely with snapshots & rollback.");
    println!("  Try: safe-update --dry-run");
    println!();
    gum_style("entropy-check v1.0.0", 11);
    println!("  Detect system drift and unexpected changes.");
    println!("  Try: entropy-check");
    println!();
    gum_style("faelight-git v2.1.0", 11);
    println!("  Git intelligence with Risk Score engine.");
    println!("  Try: faelight-git status");
    println!();
    gum_style("core-protect v1.0.1", 11);
    println!("  Lock/unlock 0-core for safe editing.");
    println!("  Try: core-protect --status");
    println!();
    if !pause(Some(3)) { process::exit(0); }
}

fn lesson_workflow() {
    gum_section("📅 Daily Workflow");
    println!("Morning routine (2 minutes):");
    println!();
    gum_style("  1. doctor              → Check system health", 14);
    gum_style("  2. workspace-view      → See workspace state", 14);
    gum_style("  3. entropy-check       → Detect drift", 14);
    gum_style("  4. teach --tips        → Daily wisdom", 14);
    println!();
    println!("Before big changes:");
    gum_style("  entropy-check --baseline   → Update baseline", 10);
    println!();
    println!("Weekly:");
    gum_style("  safe-update                → Update system safely", 14);
    gum_style("  teach --quiz               → Test knowledge retention", 14);
    println!();
    println!("The workflow is {}simple{}, but {}powerful{}.",
             "\x1b[0;32m", "\x1b[0m", "\x1b[1;33m", "\x1b[0m");
    println!();
    if !pause(Some(4)) { process::exit(0); }
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
    println!("Profiles are {}declarative{}. Define the state, not the steps.",
             "\x1b[0;36m", "\x1b[0m");
    println!();
    if !pause(Some(5)) { process::exit(0); }
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
    gum_style("  complete/     → Finished intents", 11);
    gum_style("  cancelled/    → Abandoned ideas (with reasons!)", 11);
    println!();
    println!("Commands:");
    gum_style("  intent list       → See all intents", 14);
    gum_style("  intent show 001   → View specific intent", 14);
    gum_style("  intent add        → Create new intent", 14);
    println!();
    println!("Every decision has a {}why{}. Future you will thank you.",
             "\x1b[1;33m", "\x1b[0m");
    println!();
    println!("Example: Incident 001 taught us to never automate sudo at boot.");
    println!("That lesson became safe-update v1.0.0.");
    println!();
    if !pause(Some(6)) { process::exit(0); }
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
    gum_style("  mullvad connect", 10);
    gum_style("  ", 10);
    gum_style("  [deactivate]", 10);
    gum_style("  # Commands when switching away", 10);
    gum_style("  powerprofilesctl set balanced", 10);
    gum_style("  mullvad disconnect", 10);
    println!();
    println!("Add new Rust tools:");
    gum_style("  1. Create in ~/0-core/rust-tools/newtool/", 14);
    gum_style("  2. Add to workspace Cargo.toml", 14);
    gum_style("  3. cargo build --release --bin newtool", 14);
    gum_style("  4. cp target/release/newtool scripts/", 14);
    gum_style("  5. Document in INTENT/", 14);
    println!();
    println!("The system {}grows{} with you.", "\x1b[0;32m", "\x1b[0m");
    println!();
    if !pause(Some(7)) { process::exit(0); }
}

fn lesson_aliases() {
    gum_section("⚡ Useful Aliases");
    println!("Quick commands built into your shell:");
    println!();
    gum_style("Navigation:", 11);
    gum_style("  ..          → cd ..", 14);
    gum_style("  ...         → cd ../..", 14);
    gum_style("  core        → cd ~/0-core", 14);
    gum_style("  src         → cd ~/1-src", 14);
    println!();
    gum_style("System:", 11);
    gum_style("  doctor      → dot-doctor", 14);
    gum_style("  health      → dot-doctor", 14);
    gum_style("  ws          → workspace-view", 14);
    gum_style("  wsa         → workspace-view --active", 14);
    println!();
    gum_style("Core Protection:", 11);
    gum_style("  lock-core   → core-protect --lock", 14);
    gum_style("  unlock-core → core-protect --unlock", 14);
    println!();
    gum_style("Git:", 11);
    gum_style("  gs          → git status", 14);
    gum_style("  ga          → git add", 14);
    gum_style("  gc          → git commit", 14);
    gum_style("  gp          → git push", 14);
    gum_style("  gci         → git commit with intent", 14);
    println!();
    gum_style("Files:", 11);
    gum_style("  ls          → exa with icons", 14);
    gum_style("  ll          → exa long format", 14);
    gum_style("  tree        → exa tree view", 14);
    println!();
    println!("See all aliases: {}alias | less{}", "\x1b[0;36m", "\x1b[0m");
    println!();
    if !pause(Some(8)) { process::exit(0); }
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
    gum_style("  ✅ Useful aliases", 10);
    println!();
    println!("Next steps:");
    gum_style("  teach --quiz       → Test your knowledge", 14);
    gum_style("  teach --stats      → View your progress", 14);
    gum_style("  doctor             → Check system health", 14);
    println!();
    gum_style("Welcome to the Faelight Forest. 🌲", 11);
    println!();
}

fn run_quiz() {
    gum_section("🎯 0-Core Knowledge Quiz");
    println!("Test your understanding of the Faelight Forest.");
    println!();
    println!("10 questions. Pass with 7/10 (70%).");
    println!("Perfect score earns: 🏆 Forest Scholar");
    println!();
    
    Command::new("gum")
        .args(["confirm", "Ready to begin?", "--affirmative=Let's go!", "--negative=Not yet"])
        .status()
        .ok();
    
    let questions = vec![
        ("What's the core philosophy of 0-Core?",
         vec!["Automation over control", "Manual control over automation", "Convenience over understanding", "Perfection over recovery"],
         1),
        
        ("Which tool checks system health?",
         vec!["health-check", "system-doctor", "doctor", "check-health"],
         2),
        
        ("What does workspace-view do?",
         vec!["Manage windows", "Show Sway workspace state", "Create workspaces", "Delete workspaces"],
         1),
        
        ("How do you safely update the system?",
         vec!["sudo pacman -Syu", "yay", "safe-update", "update-system"],
         2),
        
        ("Where are decision logs stored?",
         vec!["docs/", "INTENT/", "logs/", "decisions/"],
         1),
        
        ("What does entropy-check detect?",
         vec!["File changes", "System drift", "Errors", "Viruses"],
         1),
        
        ("Which directory is for active projects?",
         vec!["~/0-core/", "~/1-src/", "~/2-projects/", "~/3-archive/"],
         2),
        
        ("What does core-protect do?",
         vec!["Delete files", "Lock/unlock 0-core", "Backup system", "Check security"],
         1),
        
        ("How do you switch to gaming mode?",
         vec!["gaming-mode", "profile gaming", "mode gaming", "switch gaming"],
         1),
        
        ("What's the purpose of Intent Ledger?",
         vec!["Store code", "Track decisions & reasoning", "List bugs", "Plan sprints"],
         1),
    ];
    
    let mut score = 0;
    
    for (i, (question, options, correct)) in questions.iter().enumerate() {
        gum_section(&format!("Question {}/10", i + 1));
        println!("{}", question);
        println!();
        
        let output = Command::new("gum")
            .args(["choose"])
            .args(options)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to run gum");
        
        let answer = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let user_choice = options.iter().position(|&o| o == answer).unwrap_or(99);
        
        if user_choice == *correct {
            score += 1;
            gum_style("✅ Correct!", 10);
        } else {
            gum_style("❌ Incorrect", 9);
            println!("   Correct answer: {}", options[*correct]);
        }
        
        println!();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    
    // Show results
    gum_section("🎯 Quiz Results");
    println!("Score: {}/10 ({}%)", score, score * 10);
    println!();
    
    if score == 10 {
        gum_style("🏆 PERFECT SCORE! Forest Scholar!", 11);
    } else if score >= 7 {
        gum_style("✅ PASSED! Well done!", 10);
    } else {
        gum_style("⚠️  Keep learning! Try teach --begin", 14);
    }
    
    println!();
    
    // Save score
    let mut progress = LearningProgress::load();
    progress.record_quiz_score(score);
    progress.save();
}

fn show_statistics() {
    let progress = LearningProgress::load();
    
    gum_section("🎓 Your Learning Journey");
    
    println!("📊 Progress: {}% ({}/8 lessons)", progress.completion_percentage(), progress.lessons_completed.len());
    
    if !progress.quiz_scores.is_empty() {
        println!("🎯 Best Quiz Score: {}/10 ({}%)", progress.best_quiz_score, progress.best_quiz_score * 10);
        let avg: f32 = progress.quiz_scores.iter().sum::<usize>() as f32 / progress.quiz_scores.len() as f32;
        println!("📈 Average Quiz Score: {:.1}/10", avg);
    }
    
    println!("🏆 Achievements: {}/11", progress.achievements.len());
    println!("📅 Last Studied: {}", progress.last_studied);
    println!("🔢 Total Sessions: {}", progress.total_sessions);
    println!();
    
    println!("Completed Lessons:");
    for i in 1..=8 {
        let status = if progress.is_lesson_complete(i) { "✅" } else { "⬜" };
        let lesson_name = match i {
            1 => "Welcome & Philosophy",
            2 => "Directory Structure",
            3 => "Core Tools",
            4 => "Daily Workflow",
            5 => "System Profiles",
            6 => "Intent Ledger",
            7 => "Customization",
            8 => "Useful Aliases",
            _ => "Unknown",
        };
        println!("  {} {}. {}", status, i, lesson_name);
    }
    
    println!();
    
    if !progress.achievements.is_empty() {
        println!("Achievements Earned:");
        for achievement in &progress.achievements {
            println!("  {}", achievement);
        }
        println!();
    }
    
    // Next goal
    if progress.lessons_completed.len() < 8 {
        println!("Next Goal: Complete all lessons for 🎓 Full Course");
    } else if progress.best_quiz_score < 10 {
        println!("Next Goal: Perfect quiz score for 🏆 Forest Scholar");
    } else {
        println!("🌲 Congratulations! You've mastered the Faelight Forest!");
    }
    
    println!();
}

fn show_daily_wisdom() {
    let tips = vec![
        ("Use workspace-view --watch for live workspace monitoring.", "Lesson 3"),
        ("safe-update --dry-run shows what would update without doing it.", "Lesson 3"),
        ("entropy-check detects silent system drift over time.", "Lesson 3"),
        ("The Intent Ledger preserves the 'why' behind every decision.", "Lesson 6"),
        ("Profiles are declarative - define the state, not the steps.", "Lesson 5"),
        ("doctor should be run daily. Make it part of your morning routine.", "Lesson 4"),
        ("core-protect locks 0-core with chattr. Safe from accidents.", "Lesson 3"),
        ("Directory numbers enforce order: 0-core before 1-src, always.", "Lesson 2"),
        ("Manual control over automation. You decide when things happen.", "Lesson 1"),
        ("Incident 001 taught us: Never automate sudo at boot.", "Lesson 6"),
        ("gci commits with intent reference. Links code to decisions.", "Lesson 8"),
        ("workspace-view shows terminal working directories for foot.", "Lesson 3"),
        ("safe-update creates snapshots before/after with rollback path.", "Lesson 3"),
        ("Profile prompt indicator hidden when in default mode.", "Lesson 5"),
        ("Understanding over convenience. Know how your system works.", "Lesson 1"),
        ("Recovery over perfection. Systems break. Be ready.", "Lesson 1"),
        ("The forest teaches those who listen. 🌲", "Philosophy"),
    ];
    
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let (tip, source) = &tips[seed % tips.len()];
    
    gum_section("💡 Daily Wisdom from the Faelight Forest");
    println!("{}", tip);
    println!();
    gum_style(&format!("—{} 🌲", source), 14);
    println!();
}

fn reset_progress() {
    println!("⚠️  This will delete all your learning progress!");
    println!();
    
    let confirm = Command::new("gum")
        .args(["confirm", "Are you SURE?", "--affirmative=Yes, reset", "--negative=Cancel"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    
    if confirm {
        let path = LearningProgress::get_path();
        fs::remove_file(&path).ok();
        println!();
        gum_style("✅ Progress reset. Start fresh with: teach --begin", 10);
        println!();
    } else {
        println!("Cancelled.");
    }
}

fn run_full_course() {
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

fn run_random_lesson() {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let lesson = (seed % 8) + 1;
    
    match lesson {
        1 => lesson_welcome(),
        2 => lesson_structure(),
        3 => lesson_tools(),
        4 => lesson_workflow(),
        5 => lesson_profiles(),
        6 => lesson_intent(),
        7 => lesson_customize(),
        8 => lesson_aliases(),
        _ => {}
    }
}

fn main_menu() {
    let progress = LearningProgress::load();
    
    loop {
        Command::new("clear").status().ok();
        Command::new("gum")
            .args(["style", "--border", "double", "--border-foreground", "10",
                   "--padding", "1 2", "--margin", "1", 
                   &format!("🌲 0-Core Teaching Mode v{}", VERSION)])
            .status()
            .ok();
        
        println!();
        println!("Progress: {}% | Achievements: {}/11 | Quiz: {}/10",
                 progress.completion_percentage(),
                 progress.achievements.len(),
                 progress.best_quiz_score);
        println!();
        
        let choices = vec![
            "🚀 Start from beginning".to_string(),
            format!("{} 1. Welcome & Philosophy", if progress.is_lesson_complete(1) { "✅" } else { "⬜" }),
            format!("{} 2. Directory Structure", if progress.is_lesson_complete(2) { "✅" } else { "⬜" }),
            format!("{} 3. Core Tools", if progress.is_lesson_complete(3) { "✅" } else { "⬜" }),
            format!("{} 4. Daily Workflow", if progress.is_lesson_complete(4) { "✅" } else { "⬜" }),
            format!("{} 5. System Profiles", if progress.is_lesson_complete(5) { "✅" } else { "⬜" }),
            format!("{} 6. Intent Ledger", if progress.is_lesson_complete(6) { "✅" } else { "⬜" }),
            format!("{} 7. Customization", if progress.is_lesson_complete(7) { "✅" } else { "⬜" }),
            format!("{} 8. Useful Aliases", if progress.is_lesson_complete(8) { "✅" } else { "⬜" }),
            "🎯 Take the Quiz".to_string(),
            "📊 View Statistics".to_string(),
            "💡 Daily Wisdom".to_string(),
            "🎲 Random Lesson".to_string(),
            "🚪 Exit".to_string(),
        ];
        
        let choice_refs: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();
        
        let output = Command::new("gum")
            .args(["choose"])
            .args(&choice_refs)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to run gum");
        
        let choice = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        if choice.contains("Start from beginning") {
            run_full_course();
        } else if choice.contains("1. Welcome") {
            lesson_welcome();
        } else if choice.contains("2. Directory") {
            lesson_structure();
        } else if choice.contains("3. Core Tools") {
            lesson_tools();
        } else if choice.contains("4. Daily Workflow") {
            lesson_workflow();
        } else if choice.contains("5. System Profiles") {
            lesson_profiles();
        } else if choice.contains("6. Intent Ledger") {
            lesson_intent();
        } else if choice.contains("7. Customization") {
            lesson_customize();
        } else if choice.contains("8. Useful Aliases") {
            lesson_aliases();
        } else if choice.contains("Take the Quiz") {
            run_quiz();
        } else if choice.contains("View Statistics") {
            show_statistics();
            Command::new("gum")
                .args(["confirm", "Back to menu?", "--affirmative=Yes"])
                .status()
                .ok();
        } else if choice.contains("Daily Wisdom") {
            show_daily_wisdom();
            Command::new("gum")
                .args(["confirm", "Back to menu?", "--affirmative=Yes"])
                .status()
                .ok();
        } else if choice.contains("Random Lesson") {
            run_random_lesson();
        } else {
            break;
        }
    }
}

fn run_health_check() {
    println!();
    println!("🏥 teach v{} - Health Check", VERSION);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    print!("  Checking gum... ");
    if check_command_exists("gum") {
        println!("✅");
    } else {
        println!("❌ Not found");
        println!("      Install with: yay -S gum");
        std::process::exit(1);
    }
    
    println!();
    println!("✅ All dependencies available");
    println!();
}

fn show_help() {
    println!("🎓 teach v{} - Interactive Learning System", VERSION);
    println!();
    println!("USAGE:");
    println!("   teach [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("   --begin              Start from lesson 1");
    println!("   --random             Random lesson");
    println!("   --quiz               Take the knowledge quiz");
    println!("   --stats              View progress & achievements");
    println!("   --tips               Daily wisdom tip");
    println!("   --reset              Reset all progress");
    println!("   --health             Check dependencies");
    println!("   --version, -v        Show version");
    println!("   --help, -h           Show help");
    println!();
    println!("EXAMPLES:");
    println!("   teach                # Interactive menu");
    println!("   teach --begin        # Full course walkthrough");
    println!("   teach --quiz         # Test your knowledge");
    println!("   teach --stats        # View your progress");
    println!("   teach --tips         # Get daily wisdom");
    println!();
    println!("LESSONS:");
    println!("   1. Welcome & Philosophy     (3 min)");
    println!("   2. Directory Structure      (2 min)");
    println!("   3. Core Tools               (5 min)");
    println!("   4. Daily Workflow           (3 min)");
    println!("   5. System Profiles          (4 min)");
    println!("   6. Intent Ledger            (5 min)");
    println!("   7. Customization            (6 min)");
    println!("   8. Useful Aliases           (4 min)");
    println!();
    println!("ACHIEVEMENTS:");
    println!("   🌱 First Steps        🌿 Directory Master");
    println!("   🔧 Tool Proficient    📅 Workflow Expert");
    println!("   🎮 Profile Pro        📜 Intent Scholar");
    println!("   🎨 Customization Guru ⚡ Alias Ninja");
    println!("   🎓 Full Course        🏆 Forest Scholar");
    println!("   🌲 Faelight Master");
    println!();
    println!("PHILOSOPHY:");
    println!("   'The forest teaches those who listen.'");
}
