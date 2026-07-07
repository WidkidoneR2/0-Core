//! INT-326: fsh Semantic Architecture
//! Three-layer execution model: Human Intent → Semantic Plan → Concrete Execution

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Delete,
    Find,
    Show,
    Deploy,
    Repair,
    Archive,
    Move,
    Rename,
    Enable,
    Disable,
    Compare,
    Check,
    List,
    Inspect,
    Observe,
    Build,
    Test,
    Commit,
    Push,
    Rollback,
    Install,
    Remove,
    History,
    Snapshot,
    Rewind,
    Doctor,
    Enter,
    Rename2,
    Execute,
    Filter,
    Terminate,
    Unknown(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Target {
    File(String),
    Service(String),
    System(String),
    Tool(String),
    Intent(String),
    Pattern(String),
    Command(String),
    Process(String),
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerbCategory {
    Observation,       // never mutates state -- always safe
    RecoverableAction, // reversible, state captured before
    Destructive,       // confirm required, irreversible
    Deployment,        // system-changing, tracked in deploy_patterns
    Intelligence,      // Friday-mediated, confidence-gated
    Session,           // state management, always reversible
}

impl VerbCategory {
    pub fn requires_confirm(&self) -> bool {
        matches!(self, VerbCategory::Destructive)
    }
    #[allow(dead_code)]
    pub fn is_safe(&self) -> bool {
        matches!(self, VerbCategory::Observation | VerbCategory::Session)
    }
    pub fn label(&self) -> &'static str {
        match self {
            VerbCategory::Observation => "observation (read-only)",
            VerbCategory::RecoverableAction => "recoverable action",
            VerbCategory::Destructive => "destructive (confirm required)",
            VerbCategory::Deployment => "deployment (tracked)",
            VerbCategory::Intelligence => "intelligence (Friday-mediated)",
            VerbCategory::Session => "session management",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticIntent {
    pub raw_input: String,
    pub action: Action,
    pub target: Target,
    pub category: VerbCategory,
    pub confidence: f64,
    pub reversible: bool,
    pub layer2_description: String,   // what the forest understands
    pub layer3_commands: Vec<String>, // actual execution commands
}

impl SemanticIntent {
    pub fn unknown(input: &str) -> Self {
        SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Unknown(input.to_string()),
            target: Target::Unknown(input.to_string()),
            category: VerbCategory::Observation,
            confidence: 0.0,
            reversible: true,
            layer2_description: format!("Unknown command -- treating as raw UNIX"),
            layer3_commands: vec![input.to_string()],
        }
    }
}

/// Build a SemanticIntent from raw input -- the core of the three-layer model
pub fn interpret(input: &str) -> SemanticIntent {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let verb = parts.first().copied().unwrap_or("");
    let rest = parts.get(1..).map(|p| p.join(" ")).unwrap_or_default();

    match verb {
        // ── Destructive ─────────────────────────────────────────────────────
        "delete" | "del" | "rm" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Delete,
            target: Target::File(rest.clone()),
            category: VerbCategory::Destructive,
            confidence: 1.0,
            reversible: false,
            layer2_description: format!("Delete(File(\"{}\"))", rest),
            layer3_commands: vec![format!("rm -i {}", rest)],
        },
        // ── Observation ─────────────────────────────────────────────────────
        "find" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Find,
            target: Target::File(rest.clone()),
            category: VerbCategory::Observation,
            confidence: 1.0,
            reversible: true,
            layer2_description: format!("Find(File(\"{}\"))", rest),
            layer3_commands: vec![format!("find . -name \"{}\"", rest)],
        },
        "show" if rest == "processes" || rest == "procs" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Show,
            target: Target::Process("all".to_string()),
            category: VerbCategory::Observation,
            confidence: 1.0,
            reversible: true,
            layer2_description: "Observe(Processes(all))".to_string(),
            layer3_commands: vec!["ps aux --sort=-%cpu | head -20".to_string()],
        },
        "show" | "d" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Show,
            target: Target::System(rest.clone()),
            category: VerbCategory::Observation,
            confidence: 1.0,
            reversible: true,
            layer2_description: format!(
                "Observe(System(\"{}\"))",
                if rest.is_empty() { "health" } else { &rest }
            ),
            layer3_commands: vec!["core doctor run --summary".to_string()],
        },
        "filter" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Filter,
            target: Target::Process(rest.clone()),
            category: VerbCategory::Observation,
            confidence: 1.0,
            reversible: true,
            layer2_description: format!("Filter(Process({}))", rest),
            layer3_commands: vec![format!(
                "awk '{{if ($3 > {}) print}}'",
                rest.replace("cpu >", "").replace("cpu>", "").trim()
            )],
        },
        "terminate" | "kill" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Terminate,
            target: Target::Process(rest.clone()),
            category: VerbCategory::Destructive,
            confidence: 0.9,
            reversible: false,
            layer2_description: format!(
                "Terminate(Process({}))",
                if rest.is_empty() { "from_pipe" } else { &rest }
            ),
            layer3_commands: vec!["kill".to_string()],
        },
        "history" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::History,
            target: Target::System("shell_history".to_string()),
            category: VerbCategory::Observation,
            confidence: 1.0,
            reversible: true,
            layer2_description: format!("Observe(History(\"{}\"))", rest),
            layer3_commands: vec![format!("history {}", rest)],
        },
        "rewind" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Rewind,
            target: Target::System("shell_snapshots".to_string()),
            category: VerbCategory::Session,
            confidence: 1.0,
            reversible: true,
            layer2_description: "Rewind(Snapshots) -- time-travel timeline".to_string(),
            layer3_commands: vec!["rewind".to_string()],
        },
        // ── Deployment ──────────────────────────────────────────────────────
        "deploy" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Deploy,
            target: Target::Tool(rest.clone()),
            category: VerbCategory::Deployment,
            confidence: 1.0,
            reversible: true,
            layer2_description: format!(
                "Deploy(Tool(\"{}\")) -- tracked, rollback available",
                rest
            ),
            layer3_commands: vec![
                format!("cargo build --release -p {}", rest),
                format!("cp target/release/{} ~/.cargo/bin/", rest),
            ],
        },
        // ── Session ─────────────────────────────────────────────────────────
        "snapshot" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Snapshot,
            target: Target::System("state".to_string()),
            category: VerbCategory::Session,
            confidence: 1.0,
            reversible: true,
            layer2_description: format!("Snapshot(System) -- capture current state"),
            layer3_commands: vec![format!("snapshot {}", rest)],
        },
        "fsh" if rest.starts_with("doctor") => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Doctor,
            target: Target::System("fsh".to_string()),
            category: VerbCategory::Observation,
            confidence: 1.0,
            reversible: true,
            layer2_description: "Doctor(fsh) -- shell health check, 7 checks".to_string(),
            layer3_commands: vec!["fsh doctor".to_string()],
        },
        // ── Intelligence ────────────────────────────────────────────────────
        "friday" => SemanticIntent {
            raw_input: input.to_string(),
            action: Action::Inspect,
            target: Target::System("friday".to_string()),
            category: VerbCategory::Intelligence,
            confidence: 0.9,
            reversible: true,
            layer2_description: format!("Friday(Query(\"{}\")) -- confidence-gated response", rest),
            layer3_commands: vec![format!("friday {}", rest)],
        },
        _ => SemanticIntent::unknown(input),
    }
}

/// Format a SemanticIntent as the three-layer display
pub fn format_three_layers(si: &SemanticIntent) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "\n  ┌─ Layer 1: Human Intent ────────────────────────────"
    );
    let _ = writeln!(out, "  │  {}", si.raw_input);
    let _ = writeln!(
        out,
        "  ├─ Layer 2: Semantic Plan ───────────────────────────"
    );
    let _ = writeln!(out, "  │  {}", si.layer2_description);
    let _ = writeln!(
        out,
        "  │  category: {}  confidence: {:.0}%  reversible: {}",
        si.category.label(),
        si.confidence * 100.0,
        if si.reversible {
            "yes"
        } else {
            "no (confirm required)"
        }
    );
    let _ = writeln!(
        out,
        "  ├─ Layer 3: Concrete Execution ──────────────────────"
    );
    for cmd in &si.layer3_commands {
        let _ = writeln!(out, "  │  $ {}", cmd);
    }
    let _ = writeln!(
        out,
        "  └────────────────────────────────────────────────────"
    );
    out
}

/// Multiple interpretations for ambiguous commands
pub struct AmbiguousCommand {
    pub raw_input: String,
    pub options: Vec<(SemanticIntent, f64)>, // (intent, confidence)
}

/// Commands known to be ambiguous -- require disambiguation
pub fn interpret_ambiguous(input: &str) -> Option<AmbiguousCommand> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let verb = parts.first().copied().unwrap_or("");
    let rest = parts.get(1..).map(|p| p.join(" ")).unwrap_or_default();

    match verb {
        "clean" => Some(AmbiguousCommand {
            raw_input: input.to_string(),
            options: vec![
                (
                    SemanticIntent {
                        raw_input: input.to_string(),
                        action: Action::Delete,
                        target: Target::File(format!("{} (temp files older than 7 days)", rest)),
                        category: VerbCategory::Destructive,
                        confidence: 0.71,
                        reversible: false,
                        layer2_description: format!(
                            "Delete(TempFiles(\"{}\", older_than=7d))",
                            rest
                        ),
                        layer3_commands: vec![format!(
                            "find {} -name \'*.tmp\' -mtime +7 -delete",
                            if rest.is_empty() { "." } else { &rest }
                        )],
                    },
                    0.71,
                ),
                (
                    SemanticIntent {
                        raw_input: input.to_string(),
                        action: Action::Archive,
                        target: Target::File(format!("{} (files older than 30 days)", rest)),
                        category: VerbCategory::RecoverableAction,
                        confidence: 0.58,
                        reversible: true,
                        layer2_description: format!("Archive(Files(\"{}\", older_than=30d))", rest),
                        layer3_commands: vec![format!(
                            "find {} -mtime +30 | tar czf archive.tar.gz -T -",
                            if rest.is_empty() { "." } else { &rest }
                        )],
                    },
                    0.58,
                ),
                (
                    SemanticIntent {
                        raw_input: input.to_string(),
                        action: Action::Delete,
                        target: Target::File(format!("{} (duplicates)", rest)),
                        category: VerbCategory::Destructive,
                        confidence: 0.43,
                        reversible: false,
                        layer2_description: format!("Delete(Duplicates(\"{}\"))", rest),
                        layer3_commands: vec!["fdupes -r -d .".to_string()],
                    },
                    0.43,
                ),
            ],
        }),
        "fix" => Some(AmbiguousCommand {
            raw_input: input.to_string(),
            options: vec![
                (
                    SemanticIntent {
                        raw_input: input.to_string(),
                        action: Action::Repair,
                        target: Target::System(rest.clone()),
                        category: VerbCategory::RecoverableAction,
                        confidence: 0.75,
                        reversible: true,
                        layer2_description: format!(
                            "Repair(System(\"{}\"))",
                            if rest.is_empty() {
                                "auto-detect"
                            } else {
                                &rest
                            }
                        ),
                        layer3_commands: vec![format!("core doctor --fix {}", rest)],
                    },
                    0.75,
                ),
                (
                    SemanticIntent {
                        raw_input: input.to_string(),
                        action: Action::Repair,
                        target: Target::File(rest.clone()),
                        category: VerbCategory::RecoverableAction,
                        confidence: 0.52,
                        reversible: true,
                        layer2_description: format!("Repair(File(\"{}\"))", rest),
                        layer3_commands: vec![format!(
                            "cargo fix --manifest-path {}/Cargo.toml",
                            rest
                        )],
                    },
                    0.52,
                ),
            ],
        }),
        _ => None,
    }
}

/// Format ambiguous command choices for display
pub fn format_ambiguous(amb: &AmbiguousCommand) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "\n  fsh detected ambiguity for: '{}'", amb.raw_input);
    let _ = writeln!(out, "  Multiple interpretations possible:\n");
    for (i, (si, conf)) in amb.options.iter().enumerate() {
        let _ = writeln!(
            out,
            "  {}. {} (confidence: {:.0}%)",
            i + 1,
            si.layer2_description,
            conf * 100.0
        );
        if !si.layer3_commands.is_empty() {
            let _ = writeln!(out, "     → {}", si.layer3_commands[0]);
        }
    }
    let _ = writeln!(out, "\n  Which? (1/{}/n or explain N):", amb.options.len());
    out
}
