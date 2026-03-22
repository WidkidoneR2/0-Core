// faelight-shell — Natural Language Pipeline Translation
// INT-139 Layer 1: Pattern library — no AI required
//
// Usage:
//   ?find biggest files     → files | sort size desc | first 10
//   ?memory hogs            → ps | sort memory desc | first 5
//   ?failing services       → services | where status == failed
//
// THE RULE: Show generated pipeline before executing.
// User confirms, rejects, or edits. Never silent execution.

use colored::*;

// ── Pattern ───────────────────────────────────────────────────────────────────

pub struct Pattern {
    pub phrases:  &'static [&'static str],
    pub pipeline: &'static str,
    pub context:  &'static str,
}

// ── Pattern Library — 35+ patterns ───────────────────────────────────────────

pub const PATTERNS: &[Pattern] = &[
    // ── Filesystem ────────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["biggest files", "largest files", "most space", "large files", "find biggest"],
        pipeline: "files | sort size desc | first 10",
        context:  "filesystem",
    },
    Pattern {
        phrases:  &["smallest files", "tiny files"],
        pipeline: "files | sort size | first 10",
        context:  "filesystem",
    },
    Pattern {
        phrases:  &["recent files", "changed recently", "modified recently", "new files", "what changed"],
        pipeline: "files | sort modified desc | first 10",
        context:  "filesystem",
    },
    Pattern {
        phrases:  &["list files", "show files", "files here", "what files"],
        pipeline: "files",
        context:  "filesystem",
    },
    // ── Processes ─────────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["memory hogs", "using most memory", "ram usage", "memory usage", "eating memory", "using memory", "mem", "memory"],
        pipeline: "ps | sort memory desc | first 5",
        context:  "processes",
    },
    Pattern {
        phrases:  &["cpu hogs", "using most cpu", "slow processes", "cpu usage", "eating cpu", "using cpu", "why slow", "computer slow", "system slow", "cpu", "slow"],
        pipeline: "ps | sort cpu desc | first 5",
        context:  "processes",
    },
    Pattern {
        phrases:  &["all processes", "running processes", "show processes", "list processes"],
        pipeline: "ps | sort cpu desc",
        context:  "processes",
    },
    Pattern {
        phrases:  &["my processes", "user processes"],
        pipeline: "ps | where user == christian | sort cpu desc",
        context:  "processes",
    },
    // ── Services ──────────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["failing services", "broken services", "service errors", "failed services", "which services failing"],
        pipeline: "services | where status == failed",
        context:  "services",
    },
    Pattern {
        phrases:  &["running services", "active services", "services running"],
        pipeline: "services | where active == active",
        context:  "services",
    },
    Pattern {
        phrases:  &["all services", "list services", "show services"],
        pipeline: "services",
        context:  "services",
    },
    // ── Network ───────────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["open ports", "listening ports", "network ports", "what ports", "which ports", "ports"],
        pipeline: "ports",
        context:  "network",
    },
    Pattern {
        phrases:  &["network interfaces", "network info", "ip address", "my ip"],
        pipeline: "net",
        context:  "network",
    },
    // ── Forest tools ──────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["unhealthy tools", "stale tools", "needs attention", "low score tools", "bad tools"],
        pipeline: "tt | sort score | first 10",
        context:  "forest",
    },
    Pattern {
        phrases:  &["best tools", "high score tools", "healthy tools"],
        pipeline: "tt | sort score desc | first 10",
        context:  "forest",
    },
    Pattern {
        phrases:  &["all tools", "list tools", "show tools", "forest tools"],
        pipeline: "tt",
        context:  "forest",
    },
    Pattern {
        phrases:  &["deployed tools", "installed tools"],
        pipeline: "tt | where deployed == true",
        context:  "forest",
    },
    // ── Git ───────────────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["recent commits", "latest changes", "git history", "what committed", "last commits", "commits", "git"],
        pipeline: "gc | first 10",
        context:  "git",
    },
    Pattern {
        phrases:  &["my commits", "my changes", "what i committed"],
        pipeline: "gc | where author == christian | first 10",
        context:  "git",
    },
    Pattern {
        phrases:  &["today commits", "committed today", "changes today"],
        pipeline: "gc | first 20",
        context:  "git",
    },
    // ── Events ────────────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["what happened today", "today events", "recent events", "forest events"],
        pipeline: "et today",
        context:  "forest",
    },
    Pattern {
        phrases:  &["git events", "git activity"],
        pipeline: "et today | where domain == git",
        context:  "forest",
    },
    Pattern {
        phrases:  &["shell events", "shell activity", "what i ran"],
        pipeline: "et today | where domain == shell",
        context:  "forest",
    },
    // ── Forest state ──────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["check forest", "check health", "forest health", "how healthy", "system health"],
        pipeline: "health",
        context:  "forest",
    },
    Pattern {
        phrases:  &["what planned", "planned intents", "upcoming work", "what next", "next tasks"],
        pipeline: "intents | where status == planned",
        context:  "forest",
    },
    Pattern {
        phrases:  &["active intents", "in progress", "working on", "current work"],
        pipeline: "intents | where status == in-progress",
        context:  "forest",
    },
    Pattern {
        phrases:  &["recent decisions", "last decisions", "decisions made"],
        pipeline: "dt | last 5",
        context:  "forest",
    },
    Pattern {
        phrases:  &["audit scores", "tool scores", "check scores"],
        pipeline: "tt | sort score | first 10",
        context:  "forest",
    },
    // ── History ───────────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["command history", "recent commands", "what i typed", "last commands"],
        pipeline: "ht | last 10",
        context:  "history",
    },
    Pattern {
        phrases:  &["most used commands", "frequent commands"],
        pipeline: "ht | count",
        context:  "history",
    },
    // ── Packages ──────────────────────────────────────────────────────────────
    Pattern {
        phrases:  &["installed packages", "system packages", "list packages", "what installed"],
        pipeline: "pkgs | first 20",
        context:  "system",
    },
];

// ── Translation Result ────────────────────────────────────────────────────────

pub struct Translation {
    pub pipeline:   String,
    pub confidence: f32,
    pub context:    String,
    pub matched_phrase: String,
}

// ── Translate — find best matching pattern ────────────────────────────────────

#[allow(dead_code)]
pub fn translate(input: &str) -> Option<Translation> {
    let input_lower = input.to_lowercase();
    let input_lower = input_lower.trim_start_matches('?').trim();

    let mut best: Option<(f32, &Pattern, String)> = None;

    for pattern in PATTERNS {
        for phrase in pattern.phrases {
            let score = similarity(input_lower, phrase);
            if score > 0.4 {
                if best.as_ref().map(|(s, _, _)| score > *s).unwrap_or(true) {
                    best = Some((score, pattern, phrase.to_string()));
                }
            }
        }
    }

    best.map(|(score, pattern, phrase)| Translation {
        pipeline:       pattern.pipeline.to_string(),
        confidence:     score,
        context:        pattern.context.to_string(),
        matched_phrase: phrase,
    })
}

// ── Similarity — token overlap score ─────────────────────────────────────────

fn similarity(input: &str, phrase: &str) -> f32 {
    let input_tokens: Vec<&str> = input.split_whitespace().collect();
    let phrase_tokens: Vec<&str> = phrase.split_whitespace().collect();

    if input_tokens.is_empty() || phrase_tokens.is_empty() {
        return 0.0;
    }

    // Count how many input tokens appear in the phrase
    let matches = input_tokens.iter()
        .filter(|t| phrase_tokens.contains(t))
        .count();

    // Also check reverse — phrase tokens in input
    let rev_matches = phrase_tokens.iter()
        .filter(|t| input_tokens.contains(t))
        .count();

    let forward  = matches as f32 / input_tokens.len() as f32;
    let backward = rev_matches as f32 / phrase_tokens.len() as f32;

    // Weighted average — forward match is more important
    (forward * 0.6) + (backward * 0.4)
}

// ── Display ───────────────────────────────────────────────────────────────────

pub fn render_translation(t: &Translation) -> String {
    let confidence_label = if t.confidence >= 0.8 {
        "high".bright_green().to_string()
    } else if t.confidence >= 0.6 {
        "medium".yellow().to_string()
    } else {
        "low".bright_red().to_string()
    };

    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("  {} {}\n",
        "🌿 Natural language:".bright_cyan().bold(),
        t.matched_phrase.dimmed(),
    ));
    out.push_str(&format!("  {} {}\n",
        "→  Pipeline:".bright_white().bold(),
        t.pipeline.bright_green().bold(),
    ));
    out.push_str(&format!("  {} {}  {} {}\n",
        "Context:".dimmed(),    t.context.dimmed(),
        "Confidence:".dimmed(), confidence_label,
    ));
    out.push_str(&format!("\n  {} {} {}  {} {}\n",
        "Run?".bright_white(),
        "[y]".bright_green(), "yes, execute".dimmed(),
        "[n]".bright_red(),   "cancel".dimmed(),
    ));
    out
}

// ── Pattern list — shown when user types ? alone ──────────────────────────────

pub fn render_pattern_list() -> String {
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("  {}\n", "🌿 Natural Language Patterns".bright_cyan().bold()));
    out.push_str(&format!("  {}\n", "━".repeat(52).dimmed()));
    out.push_str(&format!("  {} {}\n\n",
        "Usage:".dimmed(),
        "?<phrase>  — translate to pipeline and confirm".bright_white(),
    ));

    let categories = [
        ("Filesystem",  &["biggest files", "recent files", "list files"][..]),
        ("Processes",   &["memory hogs", "cpu hogs", "all processes"]),
        ("Services",    &["failing services", "running services", "all services"]),
        ("Network",     &["open ports", "network interfaces"]),
        ("Forest",      &["check forest", "unhealthy tools", "all tools", "what planned", "audit scores"]),
        ("Git",         &["recent commits", "my commits"]),
        ("Events",      &["what happened today", "git events", "shell events"]),
        ("History",     &["command history", "most used commands"]),
        ("Packages",    &["installed packages"]),
    ];

    for (category, examples) in &categories {
        out.push_str(&format!("  {}\n", category.bright_white().bold()));
        for ex in *examples {
            out.push_str(&format!("    {} {}\n",
                "?".bright_cyan(),
                ex.dimmed(),
            ));
        }
        out.push('\n');
    }

    out.push_str(&format!("  {}\n",
        "Tip: partial phrases work too — ?slow, ?memory, ?commits".dimmed().italic()
    ));
    out.push('\n');
    out
}

// ── Custom TOML Patterns — INT-139 Criterion 8 ───────────────────────────────
// Load user-defined patterns from ~/.config/faelight-shell/nl-patterns.toml
// or ~/0-core/01-registry/shell-patterns.toml

#[derive(Debug, Clone)]
pub struct CustomPattern {
    pub phrases:  Vec<String>,
    pub pipeline: String,
    pub context:  String,
}

pub fn load_toml_patterns(core_root: &str) -> Vec<CustomPattern> {
    let mut patterns = vec![];

    let paths = vec![
        format!("{}/.config/faelight-shell/nl-patterns.toml",
            std::env::var("HOME").unwrap_or_default()),
        format!("{}/01-registry/shell-patterns.toml", core_root),
    ];

    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(value) = content.parse::<toml::Value>() {
                if let Some(arr) = value.get("pattern").and_then(|v| v.as_array()) {
                    for item in arr {
                        let phrases: Vec<String> = item.get("phrases")
                            .and_then(|v| v.as_array())
                            .map(|a| a.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect())
                            .unwrap_or_default();
                        let pipeline = item.get("pipeline")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let context = item.get("context")
                            .and_then(|v| v.as_str())
                            .unwrap_or("custom")
                            .to_string();
                        if !phrases.is_empty() && !pipeline.is_empty() {
                            patterns.push(CustomPattern { phrases, pipeline, context });
                        }
                    }
                }
            }
        }
    }
    patterns
}

pub fn translate_with_custom(input: &str, custom: &[CustomPattern]) -> Option<Translation> {
    let input_lower = input.to_lowercase();
    let input_lower = input_lower.trim_start_matches('?').trim();

    // Check built-in patterns first
    let mut best: Option<(f32, String, String, String)> = None;

    for pattern in PATTERNS {
        for phrase in pattern.phrases {
            let score = similarity(input_lower, phrase);
            if score > 0.4 {
                if best.as_ref().map(|(s, _, _, _)| score > *s).unwrap_or(true) {
                    best = Some((score,
                        pattern.pipeline.to_string(),
                        pattern.context.to_string(),
                        phrase.to_string()));
                }
            }
        }
    }

    // Check custom TOML patterns — can override built-ins if higher score
    for pattern in custom {
        for phrase in &pattern.phrases {
            let score = similarity(input_lower, phrase);
            if score > 0.4 {
                if best.as_ref().map(|(s, _, _, _)| score > *s).unwrap_or(true) {
                    best = Some((score,
                        pattern.pipeline.clone(),
                        pattern.context.clone(),
                        phrase.clone()));
                }
            }
        }
    }

    best.map(|(confidence, pipeline, context, matched_phrase)| Translation {
        pipeline,
        confidence,
        context,
        matched_phrase,
    })
}

// ── Phase 25 — Auto-diagnose patterns ────────────────────────────────────────
// These patterns trigger multi-step diagnosis rather than single pipelines.
// The shell becomes an amplifier — surfaces insights without being asked.

pub fn is_diagnostic(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.contains("why") || lower.contains("slow") || lower.contains("diagnos")
        || lower.contains("what's wrong") || lower.contains("whats wrong")
}

pub fn auto_diagnose(input: &str) -> Vec<String> {
    let lower = input.to_lowercase();
    let mut steps = vec![];

    if lower.contains("slow") || lower.contains("why") || lower.contains("performance") {
        steps.push("ps | sort cpu desc | first 5".to_string());
        steps.push("ps | sort memory desc | first 5".to_string());
        steps.push("find | where size > 500000000 | sort size desc | first 5".to_string());
    }
    if lower.contains("memory") || lower.contains("ram") {
        steps.push("ps | sort memory desc | first 10".to_string());
    }
    if lower.contains("disk") || lower.contains("space") || lower.contains("storage") {
        steps.push("find | group ext | sort count desc | first 10".to_string());
        steps.push("find | where size > 100000000 | sort size desc | first 10".to_string());
    }
    if lower.contains("network") || lower.contains("ports") || lower.contains("connection") {
        steps.push("ports".to_string());
        steps.push("ps | join ports on pid | where r_port != ".to_string());
    }
    if steps.is_empty() {
        steps.push("ps | sort cpu desc | first 5".to_string());
        steps.push("health".to_string());
    }
    steps
}
