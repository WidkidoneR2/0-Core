// faelight-shell — Schema Registry
// Phase 11a: Formal schema system — the foundational layer
// INT-120: "Build the schema system first. It unlocks everything."
//
// This is the ground truth for:
//   - Autocomplete (Phase 11)
//   - Join validation (Phase 2 data pipelines)
//   - Query language type safety (Phase 21)
//   - AI assistant reasoning (Phase 25)
//
// THE RULE: Every system table has a registered schema.
// No hardcoded strings. No runtime surprises.

use colored::*;

// ── Column Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Text,
    Int,
    Float,
    Bool,
    Timestamp,
}

#[allow(dead_code)]
impl ColumnType {
    pub fn label(&self) -> &'static str {
        match self {
            ColumnType::Text      => "text",
            ColumnType::Int       => "int",
            ColumnType::Float     => "float",
            ColumnType::Bool      => "bool",
            ColumnType::Timestamp => "timestamp",
        }
    }

    pub fn color_label(&self) -> String {
        match self {
            ColumnType::Text      => "text".bright_cyan().to_string(),
            ColumnType::Int       => "int".bright_yellow().to_string(),
            ColumnType::Float     => "float".yellow().to_string(),
            ColumnType::Bool      => "bool".bright_green().to_string(),
            ColumnType::Timestamp => "timestamp".bright_magenta().to_string(),
        }
    }
}

// ── Column ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Column {
    pub name:        String,
    pub dtype:       ColumnType,
    pub nullable:    bool,
    pub description: String,
}

impl Column {
    fn new(name: &str, dtype: ColumnType, description: &str) -> Self {
        Column {
            name:        name.to_string(),
            dtype,
            nullable:    false,
            description: description.to_string(),
        }
    }

    fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }
}

// ── Schema Source — where does this table come from? ─────────────────────────

#[derive(Debug, Clone)]
pub enum SchemaSource {
    System,     // live OS data (ps, ports, net)
    ForestDb,   // state.db (events, history)
    GitLog,     // git history
    Registry,   // tools registry / toml
    Filesystem, // directory listing
}

impl SchemaSource {
    pub fn label(&self) -> &'static str {
        match self {
            SchemaSource::System     => "system",
            SchemaSource::ForestDb   => "state.db",
            SchemaSource::GitLog     => "git log",
            SchemaSource::Registry   => "registry",
            SchemaSource::Filesystem => "filesystem",
        }
    }
}

// ── Table Schema ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name:        String,
    pub aliases:     Vec<String>,
    pub columns:     Vec<Column>,
    pub source:      SchemaSource,
    pub description: String,
}

#[allow(dead_code)]
impl TableSchema {
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }

    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Validate that a field name exists in this schema
    pub fn has_column(&self, name: &str) -> bool {
        self.columns.iter().any(|c| c.name == name)
    }
}

// ── Schema Registry ───────────────────────────────────────────────────────────

pub struct SchemaRegistry {
    pub tables: Vec<TableSchema>,
}

impl SchemaRegistry {
    /// Build and return the global registry with all system tables registered.
    /// This is the single source of truth for all table shapes.
    pub fn build() -> Self {
        SchemaRegistry {
            tables: vec![
                schema_ps(),
                schema_files(),
                schema_services(),
                schema_ports(),
                schema_net(),
                schema_tt(),
                schema_et(),
                schema_gc(),
                schema_history(),
                schema_intents(),
            ],
        }
    }

    pub fn get(&self, name: &str) -> Option<&TableSchema> {
        self.tables.iter().find(|t| {
            t.name == name || t.aliases.iter().any(|a| a == name)
        })
    }

    pub fn names(&self) -> Vec<&str> {
        self.tables.iter().map(|t| t.name.as_str()).collect()
    }
}

// ── System Table Schemas ──────────────────────────────────────────────────────

fn schema_ps() -> TableSchema {
    TableSchema {
        name:        "ps".to_string(),
        aliases:     vec!["processes".to_string()],
        source:      SchemaSource::System,
        description: "Running processes".to_string(),
        columns: vec![
            Column::new("pid",    ColumnType::Int,   "process ID"),
            Column::new("name",   ColumnType::Text,  "process name"),
            Column::new("cpu",    ColumnType::Float, "CPU usage percent"),
            Column::new("memory", ColumnType::Float, "memory usage MB"),
            Column::new("user",   ColumnType::Text,  "owning user"),
            Column::new("status", ColumnType::Text,  "process status"),
        ],
    }
}

fn schema_files() -> TableSchema {
    TableSchema {
        name:        "files".to_string(),
        aliases:     vec!["ls".to_string(), "dir".to_string()],
        source:      SchemaSource::Filesystem,
        description: "Directory listing".to_string(),
        columns: vec![
            Column::new("name",     ColumnType::Text,      "file or directory name"),
            Column::new("kind",     ColumnType::Text,      "file or dir"),
            Column::new("size",     ColumnType::Int,       "size in bytes"),
            Column::new("modified", ColumnType::Timestamp, "last modified timestamp"),
        ],
    }
}

fn schema_services() -> TableSchema {
    TableSchema {
        name:        "services".to_string(),
        aliases:     vec!["svc".to_string(), "systemctl".to_string()],
        source:      SchemaSource::System,
        description: "Systemd service units".to_string(),
        columns: vec![
            Column::new("name",   ColumnType::Text, "service unit name"),
            Column::new("active", ColumnType::Text, "active state (active/inactive)"),
            Column::new("load",   ColumnType::Text, "load state (loaded/not-found)"),
            Column::new("status", ColumnType::Text, "sub-state (running/dead/exited)"),
        ],
    }
}

fn schema_ports() -> TableSchema {
    TableSchema {
        name:        "ports".to_string(),
        aliases:     vec!["listening".to_string()],
        source:      SchemaSource::System,
        description: "Open network ports".to_string(),
        columns: vec![
            Column::new("port",    ColumnType::Int,  "port number"),
            Column::new("state",   ColumnType::Text, "LISTEN or ESTABLISHED"),
            Column::new("address", ColumnType::Text, "bind address"),
            Column::new("process", ColumnType::Text, "owning process name").nullable(),
        ],
    }
}

fn schema_net() -> TableSchema {
    TableSchema {
        name:        "net".to_string(),
        aliases:     vec!["network".to_string(), "interfaces".to_string()],
        source:      SchemaSource::System,
        description: "Network interfaces".to_string(),
        columns: vec![
            Column::new("interface", ColumnType::Text, "interface name (eth0, wlan0)"),
            Column::new("mac",       ColumnType::Text, "MAC address"),
            Column::new("ip",        ColumnType::Text, "IP address").nullable(),
        ],
    }
}

fn schema_tt() -> TableSchema {
    TableSchema {
        name:        "tt".to_string(),
        aliases:     vec!["tools".to_string()],
        source:      SchemaSource::Registry,
        description: "Forest tool registry".to_string(),
        columns: vec![
            Column::new("name",     ColumnType::Text, "tool name"),
            Column::new("version",  ColumnType::Text, "deployed version"),
            Column::new("score",    ColumnType::Int,  "audit score 0-100"),
            Column::new("deployed", ColumnType::Bool, "is binary deployed"),
        ],
    }
}

fn schema_et() -> TableSchema {
    TableSchema {
        name:        "et".to_string(),
        aliases:     vec!["events".to_string()],
        source:      SchemaSource::ForestDb,
        description: "Forest event log from state.db".to_string(),
        columns: vec![
            Column::new("domain",    ColumnType::Text,      "event domain (shell, git, core)"),
            Column::new("action",    ColumnType::Text,      "event action (command, push, scan)"),
            Column::new("timestamp", ColumnType::Timestamp, "unix timestamp"),
            Column::new("time",      ColumnType::Text,      "human-readable time"),
        ],
    }
}

fn schema_gc() -> TableSchema {
    TableSchema {
        name:        "gc".to_string(),
        aliases:     vec!["commits".to_string(), "git".to_string()],
        source:      SchemaSource::GitLog,
        description: "Git commit history".to_string(),
        columns: vec![
            Column::new("hash",    ColumnType::Text, "short commit hash"),
            Column::new("author",  ColumnType::Text, "commit author"),
            Column::new("date",    ColumnType::Text, "commit date"),
            Column::new("message", ColumnType::Text, "commit message"),
        ],
    }
}

fn schema_history() -> TableSchema {
    TableSchema {
        name:        "history".to_string(),
        aliases:     vec!["hist".to_string()],
        source:      SchemaSource::ForestDb,
        description: "Shell command history".to_string(),
        columns: vec![
            Column::new("id",        ColumnType::Int,       "history entry ID"),
            Column::new("command",   ColumnType::Text,      "command string"),
            Column::new("timestamp", ColumnType::Timestamp, "unix timestamp"),
            Column::new("time",      ColumnType::Text,      "human-readable time"),
        ],
    }
}

fn schema_intents() -> TableSchema {
    TableSchema {
        name:        "intents".to_string(),
        aliases:     vec!["intent".to_string()],
        source:      SchemaSource::Filesystem,
        description: "Forest intent ledger".to_string(),
        columns: vec![
            Column::new("id",     ColumnType::Int,  "intent ID"),
            Column::new("title",  ColumnType::Text, "intent title"),
            Column::new("status", ColumnType::Text, "complete / in-progress / planned"),
            Column::new("date",   ColumnType::Text, "creation date"),
        ],
    }
}

// ── Display ───────────────────────────────────────────────────────────────────

/// Render the full registry as a summary list
pub fn render_registry(registry: &SchemaRegistry) -> String {
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("  {}\n", "Schema Registry".bright_cyan().bold()));
    out.push_str(&format!("  {}\n", "━".repeat(52).dimmed()));
    out.push_str(&format!("  {:<16} {:<12} {}\n",
        "Table".bright_white().bold(),
        "Source".bright_white().bold(),
        "Description".bright_white().bold(),
    ));
    out.push_str(&format!("  {}\n", "─".repeat(52).dimmed()));
    for t in &registry.tables {
        let aliases = if t.aliases.is_empty() { String::new() }
            else { format!(" ({})", t.aliases.join(", ")).dimmed().to_string() };
        out.push_str(&format!("  {:<16} {:<12} {}{}\n",
            t.name.bright_white(),
            t.source.label().dimmed(),
            t.description.dimmed(),
            aliases,
        ));
    }
    out.push_str(&format!("\n  {} {} {}",
        "Use".dimmed(),
        "schema <table>".bright_cyan(),
        "to see columns.".dimmed(),
    ));
    out.push('\n');
    out
}

/// Render a single table schema with all columns
pub fn render_table_schema(schema: &TableSchema) -> String {
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("  {} {}\n",
        "⬡".bright_cyan(),
        schema.name.bright_cyan().bold(),
    ));
    out.push_str(&format!("  {}\n", "━".repeat(52).dimmed()));
    out.push_str(&format!("  {}  {}\n",
        "Description:".dimmed(), schema.description.bright_white()));
    out.push_str(&format!("  {}  {}\n",
        "Source:".dimmed(), schema.source.label().bright_white()));
    if !schema.aliases.is_empty() {
        out.push_str(&format!("  {}  {}\n",
            "Aliases:".dimmed(), schema.aliases.join(", ").bright_white()));
    }
    out.push('\n');
    out.push_str(&format!("  {:<20} {:<12} {}\n",
        "Column".bright_white().bold(),
        "Type".bright_white().bold(),
        "Description".bright_white().bold(),
    ));
    out.push_str(&format!("  {}\n", "─".repeat(52).dimmed()));
    for col in &schema.columns {
        let nullable_marker = if col.nullable { " ?".dimmed().to_string() } else { String::new() };
        out.push_str(&format!("  {:<20} {:<20} {}{}\n",
            col.name.bright_white(),
            col.dtype.color_label(),
            col.description.dimmed(),
            nullable_marker,
        ));
    }
    out.push('\n');
    out
}
