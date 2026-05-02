//! Smart changelog engine — reads git log + intent ledger, generates structured output.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

// ─── COMMIT ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub prefix: String,  // feat, fix, chore, docs, perf, refactor
    pub scope: String,   // (core), (notify), etc — empty if none
    pub message: String, // the actual message after prefix(scope):
    #[allow(dead_code)]
    pub intent_id: Option<u32>, // extracted INT-XXX reference
    pub raw: String,
}

impl Commit {
    pub fn parse(hash: &str, subject: &str) -> Self {
        let raw = subject.to_string();
        let hash = hash.to_string();

        // Extract intent reference INT-NNN
        let intent_id = extract_intent_id(subject);

        // Parse conventional commit: prefix(scope): message
        if let Some((prefix, rest)) = parse_conventional(subject) {
            let (scope, message) = parse_scope(rest);
            return Self {
                hash,
                prefix,
                scope,
                message,
                intent_id,
                raw,
            };
        }

        // Non-conventional commit
        Self {
            hash,
            prefix: "other".to_string(),
            scope: String::new(),
            message: subject.to_string(),
            intent_id,
            raw,
        }
    }

    pub fn is_noise(&self) -> bool {
        // Skip pure noise commits
        matches!(self.prefix.as_str(), "wip" | "merge")
            || self.raw.starts_with("Merge ")
            || self.raw.contains("update lazyvim")
    }
}

fn parse_conventional(s: &str) -> Option<(String, &str)> {
    let prefixes = [
        "feat", "fix", "perf", "refactor", "docs", "chore", "bump", "style", "test", "build",
    ];
    for p in &prefixes {
        if let Some(rest) = s.strip_prefix(p) {
            if rest.starts_with('(') || rest.starts_with(':') {
                return Some((p.to_string(), rest));
            }
        }
    }
    None
}

fn parse_scope(s: &str) -> (String, String) {
    if s.starts_with('(') {
        if let Some(end) = s.find(')') {
            let scope = s[1..end].to_string();
            let rest = s[end + 1..].trim_start_matches(':').trim().to_string();
            return (scope, rest);
        }
    }
    let message = s.trim_start_matches(':').trim().to_string();
    (String::new(), message)
}

fn extract_intent_id(s: &str) -> Option<u32> {
    // Match INT-NNN or INT-NN patterns
    let s_upper = s.to_uppercase();
    if let Some(pos) = s_upper.find("INT-") {
        let rest = &s_upper[pos + 4..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            return digits.parse().ok();
        }
    }
    None
}

// ─── INTENT ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ShippedIntent {
    pub id: u32,
    pub title: String,
    #[allow(dead_code)]
    pub description: String,
}

pub fn find_shipped_intents(core_root: &PathBuf, since_tag: &str) -> Vec<ShippedIntent> {
    // Read complete/ directory and find intents modified since the last tag
    let complete_dir = core_root.join("intents/complete");
    if !complete_dir.exists() {
        return vec![];
    }

    // Get files ADDED to complete/ since last tag using git (not just modified)
    let output = Command::new("git")
        .args([
            "-C",
            core_root.to_str().unwrap_or("."),
            "-c",
            "core.quotePath=false",
            "diff",
            "--name-only",
            "--diff-filter=A",
            since_tag,
            "HEAD",
            "--",
            "intents/complete/",
        ])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: vec![],
        });

    let changed = String::from_utf8_lossy(&output.stdout);
    let mut intents = vec![];

    for line in changed.lines() {
        if !line.ends_with(".md") {
            continue;
        }
        let path = core_root.join(line);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some(intent) = parse_intent_frontmatter(&content) {
                intents.push(intent);
            }
        }
    }

    intents.sort_by_key(|i| i.id);
    intents
}

fn parse_intent_frontmatter(content: &str) -> Option<ShippedIntent> {
    let mut id = None;
    let mut title = None;
    let mut in_frontmatter = false;

    for line in content.lines() {
        if line == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                continue;
            } else {
                break;
            }
        }
        if !in_frontmatter {
            continue;
        }

        if line.starts_with("id:") {
            id = line
                .split(':')
                .nth(1)
                .and_then(|s| s.trim().parse::<u32>().ok());
        }
        if line.starts_with("title:") {
            title = line
                .split_once(':')
                .map(|x| x.1)
                .map(|s| s.trim().trim_matches('"').to_string());
        }
    }

    match (id, title) {
        (Some(id), Some(title)) => {
            let description = extract_vision_first_line(content);
            Some(ShippedIntent {
                id,
                title,
                description,
            })
        }
        _ => None,
    }
}

fn extract_vision_first_line(content: &str) -> String {
    let mut in_vision = false;
    for line in content.lines() {
        if line.contains("## Vision") {
            in_vision = true;
            continue;
        }
        if in_vision && !line.trim().is_empty() && !line.starts_with('#') {
            return line.trim().trim_matches('*').chars().take(80).collect();
        }
        if in_vision && line.starts_with('#') {
            break;
        }
    }
    String::new()
}

// ─── GIT LOG ─────────────────────────────────────────────────────────────────

pub fn get_commits_since(core_root: &PathBuf, since_tag: &str) -> Result<Vec<Commit>> {
    let output = Command::new("git")
        .args([
            "-C",
            core_root.to_str().unwrap_or("."),
            "log",
            &format!("{}..HEAD", since_tag),
            "--pretty=format:%h|%s",
        ])
        .output()
        .context("failed to run git log")?;

    let log = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<Commit> = log
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(2, '|');
            let hash = parts.next()?.trim().to_string();
            let subject = parts.next()?.trim().to_string();
            Some(Commit::parse(&hash, &subject))
        })
        .filter(|c| !c.is_noise())
        .collect();

    Ok(commits)
}

pub fn get_last_tag(core_root: &PathBuf) -> String {
    // Get all tags sorted by version, filter to only vX.Y.Z release tags
    let out = Command::new("git")
        .args([
            "-C",
            core_root.to_str().unwrap_or("."),
            "tag",
            "--sort=-version:refname",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    // Return the most recent vX.Y.Z tag
    out.lines()
        .find(|t| t.starts_with('v') && t.chars().nth(1).map(|c| c.is_ascii_digit()).unwrap_or(false))
        .unwrap_or("HEAD~50")
        .to_string()
}

// ─── GROUPED CHANGELOG ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ChangelogData {
    pub version: String,
    pub date: String,
    pub theme: String,
    pub intents: Vec<ShippedIntent>,
    pub features: Vec<Commit>,
    pub fixes: Vec<Commit>,
    pub performance: Vec<Commit>,
    #[allow(dead_code)]
    pub refactors: Vec<Commit>,
    #[allow(dead_code)]
    pub docs: Vec<Commit>,
    pub internal: Vec<Commit>, // chore, bump, other — condensed
    pub total_commits: usize,
    pub last_tag: String,
}


// INT-264: Strip internal references from public-facing commit messages
fn clean_commit_message(msg: &str) -> String {
    // Remove trailing malformed "... && gp &&..." artifacts
    let msg = if let Some(pos) = msg.find(" && gp") {
        &msg[..pos]
    } else {
        msg
    };
    // Strip leading INT-NNN: prefix
    let mut result = msg.trim().to_string();
    loop {
        let upper = result.to_uppercase();
        if upper.starts_with("INT-") {
            let rest = &result[4..];
            let digit_end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            let after = result[4 + digit_end..].trim_start_matches(|c| c == ':' || c == ' ' || c == '-').trim();
            if after.is_empty() { break; }
            result = after.to_string();
        } else {
            break;
        }
    }
    result.trim_matches('"').trim().to_string()
}
impl ChangelogData {
    pub fn build(core_root: &PathBuf, version: &str, theme: &str) -> Result<Self> {
        let last_tag = get_last_tag(core_root);
        let commits = get_commits_since(core_root, &last_tag)?;
        let intents = find_shipped_intents(core_root, &last_tag);
        let total_commits = commits.len();

        let mut features = vec![];
        let mut fixes = vec![];
        let mut performance = vec![];
        let mut refactors = vec![];
        let mut docs = vec![];
        let mut internal = vec![];

        for commit in commits {
            match commit.prefix.as_str() {
                "feat" => features.push(commit),
                "fix" => fixes.push(commit),
                "perf" => performance.push(commit),
                "refactor" => refactors.push(commit),
                "docs" => docs.push(commit),
                _ => internal.push(commit),
            }
        }
        // Newest first in all sections
        features.reverse();
        fixes.reverse();
        performance.reverse();
        refactors.reverse();
        docs.reverse();
        internal.reverse();

        let date = chrono::Local::now().format("%Y-%m-%d").to_string();

        Ok(Self {
            version: version.to_string(),
            date,
            theme: theme.to_string(),
            intents,
            features,
            fixes,
            performance,
            refactors,
            docs,
            internal,
            total_commits,
            last_tag,
        })
    }

    pub fn render_markdown(&self, stats: &ReleaseStats) -> String {
        let mut out = String::new();

        out.push_str(&format!(
            "## [{}] — {} ({})\n\n",
            self.version, self.theme, self.date
        ));

        // Completed intents — the most important section
        if !self.intents.is_empty() {
            out.push_str("### 🎯 Completed Intents\n");
            for intent in &self.intents {
                // INT-264: human title only, no INT-NNN in public output
                let clean_title = intent.title.trim_matches(|c| c == '"' || c == '\\')
                    .trim_matches(|c| c == '"' || c == ' ')
                    .replace(" -- ", " -- ");
                let clean_title = clean_title.as_str();
                out.push_str(&format!("- {}\n", clean_title));
            }
            out.push('\n');
        }

        // Features — grouped by scope
        if !self.features.is_empty() {
            out.push_str("### ✨ Features\n");
            // Group by scope
            let mut scopes: Vec<String> = self
                .features
                .iter()
                .map(|c| {
                    if c.scope.is_empty() {
                        "general".to_string()
                    } else {
                        c.scope.clone()
                    }
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            scopes.sort();
            // Show scoped groups first, then general
            for scope in &scopes {
                let group: Vec<&Commit> = self
                    .features
                    .iter()
                    .filter(|c| {
                        let s = if c.scope.is_empty() {
                            "general"
                        } else {
                            &c.scope
                        };
                        s == scope.as_str()
                    })
                    .collect();
                if !group.is_empty() {
                    if scope != "general" && !scope.to_uppercase().starts_with("INT-") {
                        out.push_str(&format!("\n**{}**\n", scope));
                    }
                    for c in group {
                        out.push_str(&format!("- {}\n", clean_commit_message(&c.message)));
                    }
                }
            }
            out.push('\n');
        }

        // Fixes
        if !self.fixes.is_empty() {
            out.push_str("### 🔧 Fixes\n");
            for c in &self.fixes {
                let _scope = if c.scope.is_empty() {
                    String::new()
                } else {
                    format!("({}) ", c.scope)
                };
                out.push_str(&format!("- {}\n", clean_commit_message(&c.message)));
            }
            out.push('\n');
        }

        // Docs
        if !self.docs.is_empty() {
            out.push_str("### 📚 Documentation\n");
            for c in &self.docs {
                out.push_str(&format!("- {}\n", clean_commit_message(&c.message)));
            }
            out.push('\n');
        }

        // Performance
        if !self.performance.is_empty() {
            out.push_str("### ⚡ Performance\n");
            for c in &self.performance {
                out.push_str(&format!("- {}\n", clean_commit_message(&c.message)));
            }
            out.push('\n');
        }

        // Internal — full list, grouped by intent where possible
        if !self.internal.is_empty() {
            out.push_str(&format!(
                "### 🔩 Internal ({} commits)\n",
                self.internal.len()
            ));
            // Group by intent prefix
            let mut intent_groups: std::collections::BTreeMap<String, Vec<String>> =
                std::collections::BTreeMap::new();
            let mut ungrouped: Vec<String> = Vec::new();
            for c in &self.internal {
                if let Some(int_id) = extract_intent_id(&c.message) {
                    intent_groups
                        .entry(int_id.to_string())
                        .or_default()
                        .push(c.message.clone());
                } else {
                    ungrouped.push(c.message.clone());
                }
            }
            for (intent, messages) in &intent_groups {
                if messages.len() == 1 {
                    out.push_str(&format!("- {}\n", clean_commit_message(&messages[0])));
                } else {
                    out.push_str(&format!("- **{}** ({} commits)\n", intent, messages.len()));
                    for msg in messages {
                        out.push_str(&format!("  - {}\n", msg));
                    }
                }
            }
            for msg in &ungrouped {
                out.push_str(&format!("- {}\n", clean_commit_message(msg)));
            }
            out.push('\n');
        }

        // Stats
        out.push_str(&format!(
            "### 📊 Stats\n- Health: {}%  ·  Commits: {}  ·  Tools: {} deployed  ·  Intents: {} complete\n\n",
            stats.health, stats.total_commits, stats.tools_deployed, stats.intents_complete
        ));

        out.push_str("---\n");
        out
    }
}

#[derive(Debug, Clone)]
pub struct ReleaseStats {
    pub health: u32,
    pub total_commits: u32,
    pub tools_deployed: u32,
    pub intents_complete: u32,
}

impl ReleaseStats {
    pub fn gather(core_root: &PathBuf) -> Self {
        let commits = Command::new("git")
            .args([
                "-C",
                core_root.to_str().unwrap_or("."),
                "rev-list",
                "--count",
                "HEAD",
            ])
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        let tools = count_tools(core_root);
        let intents = count_complete_intents(core_root);

        Self {
            health: {
                let cache = std::fs::read_to_string(
                    std::path::Path::new(&std::env::var("HOME").unwrap_or_default())
                        .join(".cache/faelight/health-status"),
                )
                .unwrap_or_else(|_| "100".to_string());
                cache.trim().parse::<u32>().unwrap_or(100)
            },
            total_commits: commits,
            tools_deployed: tools,
            intents_complete: intents,
        }
    }
}

fn count_tools(core_root: &PathBuf) -> u32 {
    let tools_toml = core_root.join("01-registry/tools.toml");
    let content = match std::fs::read_to_string(&tools_toml) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    // Count only non-retired tools — split by [[tool]] blocks
    let mut count = 0u32;
    let mut in_retired = false;
    let mut saw_name = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[tool]]" {
            if saw_name && !in_retired {
                count += 1;
            }
            in_retired = false;
            saw_name = false;
        } else if trimmed == "retired = true" {
            in_retired = true;
        } else if trimmed.starts_with("name =") {
            saw_name = true;
        }
    }
    // Count last block
    if saw_name && !in_retired {
        count += 1;
    }
    count
}

fn count_complete_intents(core_root: &PathBuf) -> u32 {
    // Count by scanning all intent subdirs for status: complete in frontmatter
    // This matches how core intent stats works
    let intents_dir = core_root.join("intents");
    let mut count = 0u32;
    if let Ok(entries) = std::fs::read_dir(&intents_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Ok(files) = std::fs::read_dir(entry.path()) {
                    for file in files.flatten() {
                        let p = file.path();
                        if p.extension().and_then(|x| x.to_str()) == Some("md") {
                            if let Ok(content) = std::fs::read_to_string(&p) {
                                if content.contains("status: complete")
                                    || content.contains("type: complete")
                                    || content.contains("[complete]")
                                {
                                    count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if count > 0 {
        count
    } else {
        // Hard fallback: just count complete/ dir
        std::fs::read_dir(core_root.join("intents/complete"))
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
                    .count() as u32
            })
            .unwrap_or(0)
    }
}
