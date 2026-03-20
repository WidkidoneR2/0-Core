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
        phrases:  &["memory hogs", "using most memory", "ram usage", "memory usage", "eating memory", "using memory"],
        pipeline: "ps | sort memory desc | first 5",
        context:  "processes",
    },
    Pattern {
        phrases:  &["cpu hogs", "using most cpu", "slow processes", "cpu usage", "eating cpu", "using cpu", "why slow", "computer slow", "system slow"],
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
        phrases:  &["open ports", "listening ports", "network ports", "what ports", "which ports"],
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
        phrases:  &["recent commits", "latest changes", "git history", "what committed", "last commits"],
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
