// faelight-shell — Command Registry
// INT-173 — The Shell Knows What It Can Do
//
// One source of truth for every command the shell knows about.
// "You cannot reason about what you cannot name."

#[allow(dead_code)]
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum CommandKind {
    Builtin,
    Alias,
    Script,
    Binary,
}

impl CommandKind {
    pub fn label(&self) -> &'static str {
        match self {
            CommandKind::Builtin => "builtin",
            CommandKind::Alias   => "alias",
            CommandKind::Script  => "script",
            CommandKind::Binary  => "binary",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub name:        String,
    pub kind:        CommandKind,
    pub source:      String,
    pub description: String,
    pub usage:       String,
}

#[allow(dead_code)]
impl CommandEntry {
    pub fn builtin(name: &str, description: &str, usage: &str) -> Self {
        Self {
            name:        name.to_string(),
            kind:        CommandKind::Builtin,
            source:      "faelight-shell built-in".to_string(),
            description: description.to_string(),
            usage:       usage.to_string(),
        }
    }

    pub fn alias(name: &str, target: &str, source: &str) -> Self {
        Self {
            name:        name.to_string(),
            kind:        CommandKind::Alias,
            source:      source.to_string(),
            description: format!("alias → {}", target),
            usage:       name.to_string(),
        }
    }

    pub fn script(name: &str, path: &str) -> Self {
        Self {
            name:        name.to_string(),
            kind:        CommandKind::Script,
            source:      path.to_string(),
            description: "forest script".to_string(),
            usage:       name.to_string(),
        }
    }

    pub fn binary(name: &str, path: &str) -> Self {
        Self {
            name:        name.to_string(),
            kind:        CommandKind::Binary,
            source:      path.to_string(),
            description: String::new(),
            usage:       name.to_string(),
        }
    }
}

pub struct Registry {
    pub entries: HashMap<String, CommandEntry>,
}

#[allow(dead_code)]
impl Registry {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    pub fn register(&mut self, entry: CommandEntry) {
        self.entries.insert(entry.name.clone(), entry);
    }

    pub fn get(&self, name: &str) -> Option<&CommandEntry> {
        self.entries.get(name)
    }

    pub fn set_description(&mut self, name: &str, description: &str) {
        if let Some(entry) = self.entries.get_mut(name) {
            entry.description = description.to_string();
        }
    }

    /// Populate registry from all known sources
    pub fn populate(
        &mut self,
        db: &crate::db::ForestDb,
        core_root: &str,
    ) {
        // ── Builtins ─────────────────────────────────────────────────────────
        let builtins: &[(&str, &str, &str)] = &[
            ("d",          "Run health check",                    "d"),
            ("health",     "System health and status",            "health"),
            ("events",     "Recent forest events",                "events [today|domain]"),
            ("decisions",  "Open decisions from ledger",          "decisions"),
            ("intents",    "Active intents",                      "intents"),
            ("tools",      "Tool deployment status",              "tools"),
            ("version",    "Forest version",                      "version"),
            ("gc",         "Git commits as structured table",     "gc [n]"),
            ("ps",         "Running processes as table",          "ps"),
            ("history",    "Command history as table",            "history [n]"),
            ("ht",         "History table shortcut",              "ht"),
            ("last_error", "Show last structured error",          "last_error [explain|suggest]"),
            ("errors",     "Session error log",                   "errors [n]"),
            ("which",      "Show command source",                 "which <cmd>"),
            ("describe",   "Describe a command",                  "describe <cmd>"),
            ("command",    "Command registry queries",            "command [list|info <cmd>]"),
            ("cd",         "Change directory",                    "cd <path>"),
            ("pwd",        "Print working directory",             "pwd"),
            ("ls",         "List directory",                      "ls [path]"),
            ("help",       "Show available commands",             "help"),
            ("exit",       "Exit the shell",                      "exit"),
            ("theme",      "Switch prompt theme",                 "theme <name>"),
            ("clear",      "Clear the terminal",                  "clear"),
            ("echo",       "Output text",                         "echo <text>"),
            ("cat",        "View file contents",                  "cat <file>"),
            ("find",       "Find files",                          "find <pattern>"),
            ("ports",      "Show open ports",                     "ports"),
            ("services",   "Show running services",               "services"),
        ];
        for (name, desc, usage) in builtins {
            self.register(CommandEntry::builtin(name, desc, usage));
        }

        // ── Aliases from db ───────────────────────────────────────────────────
        if let Ok(mut stmt) = db.conn.prepare(
            "SELECT name, command FROM shell_aliases"
        ) {
            let _ = stmt.query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            }).map(|rows| {
                for row in rows.flatten() {
                    let entry = CommandEntry::alias(&row.0, &row.1, "config.fsh");
                    self.register(entry);
                }
            });
        }

        // ── Forest scripts ────────────────────────────────────────────────────
        let scripts_dir = format!("{}/scripts", core_root);
        if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let path_str = path.to_string_lossy().to_string();
                        self.register(CommandEntry::script(name, &path_str));
                    }
                }
            }
        }
    }

    /// All entries sorted by name
    pub fn all_sorted(&self) -> Vec<&CommandEntry> {
        let mut entries: Vec<&CommandEntry> = self.entries.values().collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }
}
