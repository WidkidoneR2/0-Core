//! INT-244 v22 -- Pillar 4: System Cartographer
use crate::app::context::AppContext;
use crate::errors::CoreResult;
use colored::*;
use rusqlite::params;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
pub fn update(ctx: &AppContext) -> CoreResult<()> {
    super::ensure_tables(ctx)?;
    let db = &ctx.runtime.db;
    let ts = now_ts();
    let registry_path = ctx.fpath("registry/tools.toml");
    if let Ok(content) = std::fs::read_to_string(&registry_path) {
        for line in content.lines() {
            if let Some(name) = line.trim().strip_prefix("name = ") {
                let name = name.trim().trim_matches('"');
                if !name.is_empty() {
                    db.execute(
                        "INSERT INTO friday_map (entity_type, entity_name, updated_at)
                         VALUES ('tool', ?1, ?2)
                         ON CONFLICT(entity_type, entity_name) DO UPDATE SET updated_at=?2",
                        params![name, ts],
                    )?;
                }
            }
        }
    }
    println!("  {} System map updated", "✅".green());
    Ok(())
}
/// Tool, crate, and intent tree. One `git log --name-only`. The stored
/// `friday_map` table is not the source; `map-update` still writes it.
pub fn show(ctx: &AppContext) -> CoreResult<()> {
    println!("{}", render_map(Path::new(&ctx.core_root)));
    Ok(())
}

/// Intents, crates, and tools that touch `change`. One `git log --name-only`.
/// An INT id is shown only when `zero_core::ledger` has that intent.
pub fn impact(ctx: &AppContext, change: &str) -> CoreResult<()> {
    let repo = Path::new(&ctx.core_root);
    let file = normalize_path(change);
    println!("impact {file}");
    match read_commits(repo) {
        Ok(commits) => {
            let ids = intents_touching_file(&commits, file);
            if ids.is_empty() {
                println!("no intent commit touches {file}");
            } else {
                for id in &ids {
                    println!("{}", paint(id, "35"));
                }
            }
        }
        Err(err) => println!("{err}"),
    }
    let crates = load_crates(repo);
    let tools = load_tools(repo);
    let hit: Vec<usize> = crates
        .iter()
        .enumerate()
        .filter(|(_, c)| under(file, &c.dir))
        .map(|(i, _)| i)
        .collect();
    if hit.is_empty() {
        println!("no crate contains {file}");
    } else {
        for i in &hit {
            println!("crate {}", paint(&crates[*i].package, "32"));
        }
    }
    let mut named = BTreeSet::new();
    for tool in &tools {
        if let Some(i) = link_index(tool, &crates) {
            if hit.contains(&i) {
                named.insert(tool.name.as_str());
            }
        }
    }
    if named.is_empty() {
        println!("no tool crate contains {file}");
    } else {
        for name in named {
            println!("tool {}", paint(name, "36"));
        }
    }
    Ok(())
}

struct ToolRow {
    name: String,
    package: Option<String>,
}

struct CrateRow {
    package: String,
    dir: String,
}

impl CrateRow {
    fn folder(&self) -> &str {
        self.dir.rsplit('/').next().unwrap_or(&self.dir)
    }
}

struct Commit {
    ts: i64,
    ids: Vec<String>,
    files: Vec<String>,
}

fn normalize_path(path: &str) -> &str {
    path.trim().trim_start_matches("./")
}

fn quoted_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    let value = &rest[..end];
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn load_tools(repo: &Path) -> Vec<ToolRow> {
    let mut text = String::new();
    for rel in ["zero/registry/tools.toml", "registry/tools.toml"] {
        if let Ok(part) = std::fs::read_to_string(repo.join(rel)) {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&part);
        }
    }
    let mut tools = Vec::new();
    let mut name: Option<String> = None;
    let mut package: Option<String> = None;
    let flush =
        |tools: &mut Vec<ToolRow>, name: &mut Option<String>, package: &mut Option<String>| {
            if let Some(n) = name.take() {
                if !n.is_empty() && tools.iter().all(|t| t.name != n) {
                    tools.push(ToolRow {
                        name: n,
                        package: package.take(),
                    });
                } else {
                    package.take();
                }
            }
        };
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("[[tool]]") {
            flush(&mut tools, &mut name, &mut package);
            continue;
        }
        if let Some(v) = quoted_value(line, "name") {
            name = Some(v);
        } else if let Some(v) = quoted_value(line, "crate") {
            package = Some(v);
        }
    }
    flush(&mut tools, &mut name, &mut package);
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    tools
}

fn package_name(text: &str) -> Option<String> {
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if in_package {
            if let Some(name) = quoted_value(line, "name") {
                return Some(name);
            }
        }
    }
    None
}

fn load_crates(repo: &Path) -> Vec<CrateRow> {
    let mut out = Vec::new();
    let walker = walkdir::WalkDir::new(repo).into_iter().filter_entry(|ent| {
        let name = ent.file_name();
        name != ".git" && name != "target" && name != "node_modules"
    });
    for ent in walker.flatten() {
        if !ent.file_type().is_file() || ent.file_name() != "Cargo.toml" {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(ent.path()) else {
            continue;
        };
        let Some(package) = package_name(&text) else {
            continue;
        };
        let Some(dir) = ent.path().parent() else {
            continue;
        };
        let Ok(rel) = dir.strip_prefix(repo) else {
            continue;
        };
        let dir = rel.to_string_lossy().replace('\\', "/");
        out.push(CrateRow { package, dir });
    }
    out.sort_by(|a, b| a.dir.cmp(&b.dir).then(a.package.cmp(&b.package)));
    out
}

fn link_index(tool: &ToolRow, crates: &[CrateRow]) -> Option<usize> {
    let want = tool.package.as_deref().unwrap_or(tool.name.as_str());
    crates
        .iter()
        .position(|c| c.package == want || c.folder() == want)
}

fn under(path: &str, dir: &str) -> bool {
    if dir.is_empty() || dir == "." {
        return !path.contains('/');
    }
    path == dir || path.starts_with(&format!("{dir}/"))
}

fn read_commits(repo: &Path) -> Result<Vec<Commit>, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "-c",
            "color.ui=false",
            "--no-pager",
            "log",
            "--name-only",
            "--pretty=format:%x1e%ct%n%s%n%b%n%x1f",
        ])
        .output()
        .map_err(|e| format!("git log could not run: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let err = err.trim().lines().next().unwrap_or("git log failed");
        return Err(format!(
            "git log exited {}: {err}",
            out.status.code().unwrap_or(-1)
        ));
    }
    let mut commits = parse_commits(&String::from_utf8_lossy(&out.stdout));
    let known = ledger_numbers(repo);
    for commit in &mut commits {
        commit
            .ids
            .retain(|id| intent_number(id).is_some_and(|n| known.contains(&n)));
    }
    Ok(commits)
}

/// Intent numbers on the ledger under this repo. Records are not intents.
/// Tests inject the ledger by writing `zero/intents/` in the same temp repo
/// `render_map` and `impact` already read.
fn ledger_numbers(repo: &Path) -> HashSet<u64> {
    zero_core::ledger::load_from(&repo.join("zero/intents"))
        .into_iter()
        .filter(|file| !file.is_record())
        .filter_map(|file| intent_number(&file.id))
        .collect()
}

fn intent_number(id: &str) -> Option<u64> {
    let id = id.trim().strip_prefix("INT-").unwrap_or(id.trim());
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    id.parse().ok()
}

fn parse_commits(raw: &str) -> Vec<Commit> {
    let mut out = Vec::new();
    for rec in raw.split('\u{1e}') {
        let rec = rec.trim_start_matches(['\r', '\n']);
        if rec.is_empty() {
            continue;
        }
        let Some((head, files)) = rec.split_once('\u{1f}') else {
            continue;
        };
        let mut lines = head.split('\n');
        let Some(ts_line) = lines.next() else {
            continue;
        };
        let Ok(ts) = ts_line.trim().parse::<i64>() else {
            continue;
        };
        let subject = lines.next().unwrap_or("").trim_end_matches('\r');
        let body: Vec<&str> = lines.map(|l| l.trim_end_matches('\r')).collect();
        let ids = intent_ids(subject, &body);
        let files: Vec<String> = files
            .split('\n')
            .map(|l| l.trim().trim_matches('"').to_string())
            .filter(|l| !l.is_empty())
            .collect();
        out.push(Commit { ts, ids, files });
    }
    out
}

fn intent_ids(subject: &str, body: &[&str]) -> Vec<String> {
    let mut ids = BTreeSet::new();
    collect_ids(subject, &mut ids);
    for line in trailer_lines(body) {
        collect_ids(line, &mut ids);
    }
    ids.into_iter().collect()
}

fn collect_ids(text: &str, ids: &mut BTreeSet<String>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 4 < bytes.len() {
        let boundary = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if boundary && bytes[i..].starts_with(b"INT-") {
            let mut j = i + 4;
            if j < bytes.len() && bytes[j].is_ascii_digit() {
                while j < bytes.len() && bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if let Ok(id) = std::str::from_utf8(&bytes[i..j]) {
                    ids.insert(id.to_string());
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
}

fn is_trailer_line(line: &str) -> bool {
    let Some((key, _)) = line.split_once(':') else {
        return false;
    };
    !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn trailer_lines<'a>(body: &[&'a str]) -> Vec<&'a str> {
    let mut end = body.len();
    while end > 0 && body[end - 1].trim().is_empty() {
        end -= 1;
    }
    if end == 0 {
        return Vec::new();
    }
    let mut start = end;
    while start > 0 {
        let line = body[start - 1];
        if line.trim().is_empty() {
            break;
        }
        let cont = line.starts_with(' ') || line.starts_with('\t');
        if !cont && !is_trailer_line(line) {
            return Vec::new();
        }
        start -= 1;
    }
    if start < end && is_trailer_line(body[start]) {
        body[start..end].to_vec()
    } else {
        Vec::new()
    }
}

fn intent_ord(id: &str) -> (u64, &str) {
    let n = id
        .strip_prefix("INT-")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (n, id)
}

fn intents_touching_file(commits: &[Commit], file: &str) -> Vec<String> {
    let mut ids = BTreeSet::new();
    for c in commits {
        if c.files.iter().any(|f| f == file) {
            ids.extend(c.ids.iter().cloned());
        }
    }
    let mut ids: Vec<String> = ids.into_iter().collect();
    ids.sort_by(|a, b| intent_ord(a).cmp(&intent_ord(b)));
    ids
}

fn intents_in_dir<'a>(commits: &'a [Commit], dir: &str) -> Vec<&'a str> {
    let mut ids = BTreeSet::new();
    for c in commits {
        if c.ids.is_empty() || !c.files.iter().any(|f| under(f, dir)) {
            continue;
        }
        for id in &c.ids {
            ids.insert(id.as_str());
        }
    }
    let mut v: Vec<&str> = ids.into_iter().collect();
    v.sort_by(|a, b| intent_ord(a).cmp(&intent_ord(b)));
    v
}

fn changed_in(commits: &[Commit], dir: &str, cutoff: i64) -> bool {
    commits
        .iter()
        .any(|c| c.ts >= cutoff && c.files.iter().any(|f| under(f, dir)))
}

fn intent_changed_in(commits: &[Commit], dir: &str, id: &str, cutoff: i64) -> bool {
    commits.iter().any(|c| {
        c.ts >= cutoff && c.ids.iter().any(|x| x == id) && c.files.iter().any(|f| under(f, dir))
    })
}

fn paint(text: &str, code: &str) -> String {
    if colored::control::SHOULD_COLORIZE.should_colorize() {
        format!("\u{1b}[{code}m{text}\u{1b}[0m")
    } else {
        text.to_string()
    }
}

fn node(indent: &str, name: &str, code: &str, recent: bool) -> String {
    let text = if recent {
        format!("{name} \u{2726}")
    } else {
        name.to_string()
    };
    format!("{indent}{}", paint(&text, code))
}

fn tree_lines(
    tools: &[ToolRow],
    crates: &[CrateRow],
    commits: &[Commit],
    cutoff: i64,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut shown = HashSet::new();
    for tool in tools {
        let Some(i) = link_index(tool, crates) else {
            continue;
        };
        shown.insert(i);
        let dir = &crates[i].dir;
        let tool_recent = changed_in(commits, dir, cutoff);
        lines.push(node("", &tool.name, "36", tool_recent));
        lines.push(node("  ", &crates[i].package, "32", tool_recent));
        for id in intents_in_dir(commits, dir) {
            let recent = intent_changed_in(commits, dir, id, cutoff);
            lines.push(node("    ", id, "35", recent));
        }
    }
    for (i, krate) in crates.iter().enumerate() {
        if shown.contains(&i) {
            continue;
        }
        let recent = changed_in(commits, &krate.dir, cutoff);
        lines.push(node("", &krate.package, "32", recent));
        for id in intents_in_dir(commits, &krate.dir) {
            let spark = intent_changed_in(commits, &krate.dir, id, cutoff);
            lines.push(node("  ", id, "35", spark));
        }
    }
    for tool in tools {
        if link_index(tool, crates).is_some() {
            continue;
        }
        // No crate directory, so this node has no files a commit can touch.
        // A registry edit, a file mtime, or a binary date is not a spark.
        lines.push(node("", &tool.name, "36", false));
    }
    lines
}

// INT-316 G3. Intent ids from one `git log --name-only`, kept when the subject
// or a trailer names INT-NNN, the commit touches `file`, and the ledger has the id.
#[cfg(test)]
fn impact_intents(repo: &Path, file: &str) -> Vec<String> {
    let Ok(commits) = read_commits(repo) else {
        return Vec::new();
    };
    intents_touching_file(&commits, normalize_path(file))
}

// INT-316 G3. Tools cyan (36), crates green (32), intents magenta (35), and a
// spark on anything changed in the last 24 hours. Color follows the colored
// crate, so NO_COLOR and a non-tty stay plain unless an override is set.
fn render_map(repo: &Path) -> String {
    let tools = load_tools(repo);
    let crates = load_crates(repo);
    let (commits, mut lines) = match read_commits(repo) {
        Ok(commits) => (commits, Vec::new()),
        Err(err) => (Vec::new(), vec![err]),
    };
    let cutoff = now_ts().saturating_sub(86_400);
    lines.extend(tree_lines(&tools, &crates, &commits, cutoff));
    if lines.is_empty() {
        lines.push("no tools, crates, or intent commits were read".to_string());
    }
    lines.join("\n")
}

#[cfg(test)]
mod int316 {
    use super::{impact_intents, render_map};

    struct TempDir(std::path::PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn git(dir: &std::path::Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            .status()
            .unwrap();
        assert!(status.success(), "{args:?}");
    }

    // A temp repo whose tree names a tool and a crate, and whose history
    // mentions INT-310 and INT-314 on the same file.
    fn repo_touched_by_two_intents() -> TempDir {
        let dir = std::env::temp_dir().join(format!(
            "int316-map-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("zero/mapcrate/src/lib.rs");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::create_dir_all(dir.join("registry")).unwrap();
        std::fs::write(
            dir.join("registry/tools.toml"),
            "[[tool]]\nname = \"maptool\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("zero/mapcrate/Cargo.toml"),
            "[package]\nname = \"mapcrate\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(&file, "one\n").unwrap();
        git(&dir, &["init"]);
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-m", "INT-310: one reader"]);
        std::fs::write(&file, "two\n").unwrap();
        git(&dir, &["add", "zero/mapcrate/src/lib.rs"]);
        git(&dir, &["commit", "-m", "INT-314: tells the truth"]);
        write_ledger_intent(&dir, "310");
        write_ledger_intent(&dir, "314");
        TempDir(dir)
    }

    // A tool with no crate field, committed only as a registry row. The
    // registry file and a binary-shaped file are both touched just now, so a
    // spark taken from file mtimes, binary dates, or the registry commit is
    // wrong: this node has no crate directory and no intent commit.
    fn repo_with_crateless_tool() -> TempDir {
        let dir = std::env::temp_dir().join(format!(
            "int316-map-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("registry")).unwrap();
        std::fs::write(
            dir.join("registry/tools.toml"),
            "[[tool]]\nname = \"cargo-cache\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::write(dir.join("bin/cargo-cache"), "bin\n").unwrap();
        git(&dir, &["init"]);
        git(&dir, &["add", "."]);
        git(
            &dir,
            &["commit", "-m", "registry: cargo-cache has no crate"],
        );
        TempDir(dir)
    }

    // The ledger the map reads: one intent file under zero/intents/open.
    fn write_ledger_intent(dir: &std::path::Path, id: &str) {
        let open = dir.join("zero/intents/open");
        std::fs::create_dir_all(&open).unwrap();
        std::fs::write(
            open.join(format!("{id}-kept.md")),
            format!("---\nid: {id}\nstatus: complete\ntitle: \"kept\"\n---\n"),
        )
        .unwrap();
    }

    // INT-999 is named by a commit and is not a ledger file. INT-310 is both.
    fn repo_naming_an_id_the_ledger_lacks() -> TempDir {
        let dir = std::env::temp_dir().join(format!(
            "int316-map-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("zero/mapcrate/src/lib.rs");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::create_dir_all(dir.join("registry")).unwrap();
        std::fs::write(
            dir.join("registry/tools.toml"),
            "[[tool]]\nname = \"maptool\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("zero/mapcrate/Cargo.toml"),
            "[package]\nname = \"mapcrate\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::write(&file, "one\n").unwrap();
        git(&dir, &["init"]);
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-m", "INT-310: one reader"]);
        std::fs::write(&file, "two\n").unwrap();
        git(&dir, &["add", "zero/mapcrate/src/lib.rs"]);
        git(&dir, &["commit", "-m", "INT-999: throwaway test id"]);
        write_ledger_intent(&dir, "310");
        TempDir(dir)
    }

    struct ColorOverride;

    impl ColorOverride {
        fn on() -> Self {
            colored::control::set_override(true);
            Self
        }
    }

    impl Drop for ColorOverride {
        fn drop(&mut self) {
            colored::control::unset_override();
        }
    }

    fn painted(text: &str, name: &str, code: &str) -> bool {
        let mut rest = text;
        while let Some(at) = rest.find(name) {
            let prefix = &rest[..at];
            if let Some(i) = prefix.rfind('\u{1b}') {
                let seq = &prefix[i..];
                let closed = seq.contains("\u{1b}[0m");
                if seq.contains(code) && !closed {
                    return true;
                }
            }
            rest = &rest[at + name.len()..];
        }
        false
    }

    #[test]
    fn g3_impact_of_a_file_returns_int_310_and_int_314() {
        let repo = repo_touched_by_two_intents();
        let file = "zero/mapcrate/src/lib.rs";
        let mut got = impact_intents(&repo.0, file);
        got.sort();
        assert_eq!(
            got,
            vec!["INT-310".to_string(), "INT-314".to_string()],
            "impact of {file} should name both intents"
        );
    }

    #[test]
    fn g3_map_colors_tools_crates_intents_and_marks_recent() {
        let repo = repo_touched_by_two_intents();
        let _color = ColorOverride::on();
        let text = render_map(&repo.0);
        drop(_color);
        let tool = painted(&text, "maptool", "36");
        let krate = painted(&text, "mapcrate", "32");
        let intent = painted(&text, "INT-310", "35");
        let recent = text.contains('✦');
        assert!(
            tool && krate && intent && recent,
            "colored tree: tool maptool cyan={tool}, crate mapcrate green={krate}, intent INT-310 magenta={intent}, spark on the last 24h={recent}; got {text:?}"
        );
    }

    #[test]
    fn g3_an_id_absent_from_the_ledger_is_not_shown() {
        let repo = repo_naming_an_id_the_ledger_lacks();
        let file = "zero/mapcrate/src/lib.rs";
        let got = impact_intents(&repo.0, file);
        let text = render_map(&repo.0);
        assert!(
            !got.iter().any(|id| id == "INT-999") && !text.contains("INT-999"),
            "INT-999 is not in the ledger, so impact and the map must not show it; impact={got:?} map={text:?}"
        );
        assert!(
            got.iter().any(|id| id == "INT-310") && text.contains("INT-310"),
            "INT-310 is in the ledger and touches {file}; impact={got:?} map={text:?}"
        );
    }

    #[test]
    fn g3_a_tool_with_no_crate_and_no_commits_has_no_spark() {
        let repo = repo_with_crateless_tool();
        let text = render_map(&repo.0);
        let line = text
            .lines()
            .find(|l| l.contains("cargo-cache"))
            .unwrap_or("");
        assert!(
            !line.is_empty() && !line.contains('✦'),
            "a tool with no crate and no intent commits must not spark (no registry mtime, no binary date); got {line:?} in {text:?}"
        );
    }
}
