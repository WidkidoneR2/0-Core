use colored::*;
use crate::store::VaultEntry;
use crate::health;

pub fn print_banner() {
    println!();
    println!("  {} {}", "🔐".normal(),
        "faelight-vault".bright_green().bold());
    println!("  {}", "Forest-Native Credential Manager".dimmed());
    println!("{}", "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
}

pub fn prompt_master(prompt: &str) -> String {
    rpassword::prompt_password(format!("  {} 🔑 {}: ", "→".bright_cyan(), prompt))
        .unwrap_or_default()
}

pub fn print_health_bar(score: u32) {
    let (icon, label) = health::score_label(score);
    let bar_len = (score / 5) as usize;
    let bar = format!("{}{}",
        "█".repeat(bar_len).bright_green(),
        "░".repeat(20 - bar_len.min(20)).dimmed()
    );
    println!("  {}  {} {} {}%",
        "Health:".dimmed(), icon, bar, score.to_string().bright_white()
    );
    println!("  {}   {}", "".dimmed(), label.dimmed());
}

pub fn print_added(name: &str, cred_type: &str, score: u32) {
    let (icon, label) = health::score_label(score);
    println!("  {} {} added", "✅".normal(), name.bright_white().bold());
    println!("  {}  {}", "Type:".dimmed(), cred_type.bright_cyan());
    println!("  {}  {} {} {}% — {}",
        "Health:".dimmed(), icon,
        "█".repeat((score/5) as usize).bright_green(),
        score, label.dimmed()
    );
}

pub fn print_list(entries: &[VaultEntry], filter: Option<&str>) {
    let filtered: Vec<&VaultEntry> = entries.iter()
        .filter(|e| filter.map(|f| e.name.contains(f)).unwrap_or(true))
        .collect();

    if filtered.is_empty() {
        println!("  {} No credentials found", "○".dimmed());
        return;
    }

    println!("  ╭─ {} Vault ({} credentials) {}",
        "🔐".normal(),
        filtered.len().to_string().bright_white(),
        "────────────────────────────────".dimmed()
    );

    for entry in &filtered {
        let score = health::score("", entry.age_days);
        let (icon, _) = health::score_label(score);
        let bar_len = (score / 10) as usize;
        let bar = format!("{}{}",
            "█".repeat(bar_len).bright_green(),
            "░".repeat(10 - bar_len.min(10)).dimmed()
        );
        println!("  │  {:20} {} {}  {}%  {} {}d ago",
            entry.name.bright_white(),
            icon,
            bar,
            score.to_string().bright_white(),
            entry.cred_type.dimmed(),
            entry.age_days.to_string().dimmed()
        );
    }
    println!("  ╰{}", "────────────────────────────────────────────".dimmed());
    println!();
    println!("  {} vault list | where score < 50",
        "Pipeline:".dimmed());
}

pub fn print_audit(entries: &[VaultEntry]) {
    println!("  ╭─ {} Vault Audit Report {}", "🔐".normal(),
        "────────────────────────────────".dimmed());

    let mut critical: Vec<(&VaultEntry, u32)> = vec![];
    let mut weak: Vec<(&VaultEntry, u32)> = vec![];
    let mut good: Vec<(&VaultEntry, u32)> = vec![];

    for entry in entries {
        let score = health::score("", entry.age_days);
        match score {
            0..=49  => critical.push((entry, score)),
            50..=69 => weak.push((entry, score)),
            _       => good.push((entry, score)),
        }
    }

    if !critical.is_empty() {
        println!("  │");
        println!("  │  {} Critical ({}):", "🔴".normal(), critical.len());
        for (entry, score) in &critical {
            println!("  │    {:20} {}%  {} days old",
                entry.name.bright_red(),
                score.to_string().bright_red(),
                entry.age_days.to_string().dimmed()
            );
        }
    }
    if !weak.is_empty() {
        println!("  │");
        println!("  │  {} Weak ({}):", "🟡".normal(), weak.len());
        for (entry, score) in &weak {
            println!("  │    {:20} {}%  {} days old",
                entry.name.yellow(),
                score.to_string().yellow(),
                entry.age_days.to_string().dimmed()
            );
        }
    }
    if !good.is_empty() {
        println!("  │");
        println!("  │  {} Healthy ({}):", "🟢".normal(), good.len());
        for (entry, score) in &good {
            println!("  │    {:20} {}%",
                entry.name.bright_green(),
                score.to_string().bright_green()
            );
        }
    }

    println!("  │");
    let avg = if entries.is_empty() { 0 }
        else { entries.iter().map(|e| health::score("", e.age_days) as u64).sum::<u64>() / entries.len() as u64 };
    println!("  │  {} Average vault health: {}%",
        "📊".normal(), avg.to_string().bright_white()
    );
    println!("  ╰{}", "────────────────────────────────────────────".dimmed());
}
