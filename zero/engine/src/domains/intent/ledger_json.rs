//! INT-330. The intent ledger as JSON schema 1.
//!
//! Reads the open rows, their gates, closed.tsv, the completed-id set and
//! dep_state. Writes nothing. A value that could not be read is null.

use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// JSON for `core intent list --json`, plus the warnings that belong on stderr.
pub struct Built {
    pub value: serde_json::Value,
    pub warnings: Vec<String>,
}

/// The live ledger. Tests use [`build_at`] so they never read this checkout.
pub fn build() -> Built {
    build_at(
        &zero_core::paths::intents_dir(),
        &zero_core::paths::core_dir(),
    )
}

/// One intent from `show <id> --json`, or [`Shown::Unknown`] when the id is not open and not closed.
pub enum Shown {
    Value(serde_json::Value),
    Unknown,
}

/// Open id: the list object plus `gates_list`. Closed id: the closed.tsv row,
/// `"closed": true`, and the git show command. Unknown: nothing to print.
pub fn show_at(dir: &Path, repo: &Path, id: &str) -> Shown {
    let (live, _) = load_live(dir);
    let intents: Vec<super::Intent> = live.iter().map(|row| row.intent.clone()).collect();
    if let Some(row) = live.iter().find(|row| row.intent.id == id) {
        let mut value =
            serde_json::to_value(intent_json(row, &intents)).unwrap_or(serde_json::Value::Null);
        let gates = gate_json(&row.content);
        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "gates_list".to_string(),
                serde_json::to_value(&gates).unwrap_or(serde_json::Value::Null),
            );
        }
        return Shown::Value(value);
    }
    if let Ok(index) = zero_core::ledger::read_closed(dir) {
        if let Some(row) = index
            .rows
            .iter()
            .find(|row| super::trace::same_intent_id(&row.id, id))
        {
            let shown = ShowClosed {
                id: row.id.clone(),
                status: row.status.clone(),
                closed: true,
                closed_on: row.closed.clone(),
                last_commit: row.last_commit.clone(),
                title: row.title.clone(),
                git_show: super::trace::closed_text_command(repo, &row.last_commit, &row.id),
            };
            return Shown::Value(serde_json::to_value(&shown).unwrap_or(serde_json::Value::Null));
        }
    }
    Shown::Unknown
}

fn load_live(dir: &Path) -> (Vec<Row>, Vec<String>) {
    let mut unread = Vec::new();
    let mut live = Vec::new();
    for file in zero_core::ledger::load_from(dir) {
        if let Some(intent) = super::parse_intent(&file.path, &file.folder) {
            if file.is_record() {
                continue;
            }
            live.push(Row {
                intent,
                content: file.content,
            });
        } else if !super::is_index_frontmatter(&file.content) {
            let filename = file
                .path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            unread.push(format!("{}/{}", file.folder, filename));
        }
    }
    unread.sort();
    (live, unread)
}

fn gate_json(content: &str) -> Vec<GateJson> {
    super::gates::read_gates(content)
        .gates
        .into_iter()
        .map(|gate| GateJson {
            label: gate.id.unwrap_or_default(),
            state: match gate.state {
                super::gates::GateState::Open => "open",
                super::gates::GateState::Done => "done",
                super::gates::GateState::Deferred => "deferred",
            }
            .to_string(),
            text: gate.text,
        })
        .collect()
}

/// The ledger under `dir`, with git answers from `repo`.
pub fn build_at(dir: &Path, repo: &Path) -> Built {
    let mut warnings = Vec::new();
    warnings.extend(unreadable_warnings(dir));

    let (live, unread) = load_live(dir);
    warnings.extend(unread_warnings(&unread));

    let intents: Vec<super::Intent> = live.iter().map(|row| row.intent.clone()).collect();
    let mut rows: Vec<IntentJson> = live
        .iter()
        .filter(|row| row.intent.status != "complete")
        .map(|row| intent_json(row, &intents))
        .collect();
    rows.sort_by(|a, b| {
        status_rank(&a.status)
            .cmp(&status_rank(&b.status))
            .then_with(|| num_id(&a.id).cmp(&num_id(&b.id)))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut done_ids: HashSet<String> = HashSet::new();
    let mut cancelled_ids: HashSet<String> = HashSet::new();
    for row in &live {
        if row.intent.status == "complete" {
            done_ids.insert(row.intent.id.clone());
        } else if row.intent.status == "cancelled" {
            cancelled_ids.insert(row.intent.id.clone());
        }
    }

    let (recent_closed, last_close) = match zero_core::ledger::read_closed(dir) {
        Ok(index) => {
            for warning in index.warnings {
                warnings.push(format!("closed.tsv: {warning}"));
            }
            for row in &index.rows {
                if row.status == "complete" {
                    done_ids.insert(row.id.clone());
                } else if row.status == "cancelled" {
                    cancelled_ids.insert(row.id.clone());
                }
            }
            let recent = recent_closed_rows(&index.rows);
            let last = recent.first().map(|row| LastClose {
                id: row.id.clone(),
                status: row.status.clone(),
                closed: row.closed.clone(),
                last_commit: row.last_commit.clone(),
                title: row.title.clone(),
            });
            let recent = recent
                .into_iter()
                .map(|row| RecentClosed {
                    id: row.id,
                    status: row.status,
                    closed: row.closed,
                    title: row.title,
                })
                .collect();
            (recent, last)
        }
        Err(err) => {
            warnings.push(format!("closed.tsv: {err}"));
            (Vec::new(), None)
        }
    };

    if let Some(archived) = archived_complete_ids(repo) {
        done_ids.extend(archived);
    }

    let in_progress = live
        .iter()
        .filter(|row| row.intent.status == "in-progress")
        .count() as u64;
    let planned = live
        .iter()
        .filter(|row| row.intent.status == "planned")
        .count() as u64;
    let blocked = rows
        .iter()
        .filter(|row| row.status == "planned" && row.blocked)
        .count() as u64;
    let done = done_ids.len() as u64;
    let (clean, pushed) = tree_answers(repo);
    let ledger = LedgerJson {
        schema: 1,
        counts: Counts {
            total: done + in_progress + planned,
            done,
            in_progress,
            planned,
            blocked,
            cancelled: cancelled_ids.len() as u64,
        },
        intents: rows,
        recent_closed,
        last_close,
        tree: Tree { clean, pushed },
    };
    let value = serde_json::to_value(&ledger).unwrap_or(serde_json::Value::Null);
    Built { value, warnings }
}

struct Row {
    intent: super::Intent,
    content: String,
}

fn intent_json(row: &Row, intents: &[super::Intent]) -> IntentJson {
    let report = super::gates::read_gates(&row.content);
    let done = report
        .gates
        .iter()
        .filter(|gate| gate.state == super::gates::GateState::Done)
        .count() as u64;
    let blocked = row.intent.status == "planned"
        && row
            .intent
            .depends_on
            .iter()
            .any(|dep| super::dep_state(intents, dep) == super::DepState::Blocked);
    IntentJson {
        id: row.intent.id.clone(),
        title: row.intent.title.clone(),
        status: row.intent.status.clone(),
        blocked,
        vision: vision_sentence(&row.content),
        gates: Gates {
            done,
            total: report.gates.len() as u64,
        },
        format: super::format_version(frontmatter(&row.content)),
        depends_on: row.intent.depends_on.clone(),
        file: format!("{}/{}", row.intent.folder, row.intent.filename),
    }
}

fn status_rank(status: &str) -> u8 {
    match status {
        "in-progress" => 0,
        "planned" => 1,
        "deferred" => 2,
        "cancelled" => 3,
        _ => 4,
    }
}

fn num_id(id: &str) -> u32 {
    id.parse().unwrap_or(0)
}

fn frontmatter(content: &str) -> &str {
    if content.starts_with("---") {
        content.split("---").nth(1).unwrap_or("")
    } else {
        ""
    }
}

fn vision_sentence(content: &str) -> String {
    let mut in_vision = false;
    let mut body = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            if in_vision {
                break;
            }
            let heading = trimmed[3..].trim().to_ascii_lowercase();
            in_vision = heading == "vision" || heading.starts_with("vision ");
            continue;
        }
        if in_vision {
            body.push_str(trimmed);
            body.push(' ');
        }
    }
    let flat = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.is_empty() {
        return String::new();
    }
    match flat.find('.') {
        Some(at) => flat[..=at].to_string(),
        None => flat,
    }
}

fn recent_closed_rows(rows: &[zero_core::ledger::ClosedRow]) -> Vec<zero_core::ledger::ClosedRow> {
    let start = rows.len().saturating_sub(5);
    let mut recent: Vec<_> = rows[start..].to_vec();
    recent.reverse();
    recent
}

fn unread_warnings(names: &[String]) -> Vec<String> {
    if names.is_empty() {
        return Vec::new();
    }
    vec![
        format!(
            "  ⚠ {} intent file(s) could not be read: {}",
            names.len(),
            names.join(", ")
        ),
        "    run core intent validate for the reason".to_string(),
    ]
}

fn unreadable_warnings(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for folder in zero_core::ledger::LIFECYCLE
        .iter()
        .chain(zero_core::ledger::RECORDS.iter())
    {
        let Ok(entries) = std::fs::read_dir(dir.join(folder)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "README.md" {
                continue;
            }
            if let Err(err) = std::fs::read_to_string(&path) {
                names.push(format!("{folder}/{name}: {err}"));
            }
        }
    }
    names.sort();
    if names.is_empty() {
        return Vec::new();
    }
    vec![format!(
        "  ⚠ {} intent file(s) could not be read: {}",
        names.len(),
        names.join(", ")
    )]
}

/// Ids git has stored under `zero/intents/complete/`. None when git cannot answer,
/// the same rule as `zero_core::ledger::completed_ids`, against a chosen repo so a
/// test does not ask the live checkout.
fn archived_complete_ids(repo: &Path) -> Option<HashSet<String>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "log",
            "--all",
            "--name-only",
            "--format=",
            "--",
            "zero/intents",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let mut ids = HashSet::new();
    for line in text.lines() {
        if let Some(id) = complete_history_id(line) {
            ids.insert(id);
        }
    }
    Some(ids)
}

fn complete_history_id(path: &str) -> Option<String> {
    let path = path.trim().trim_matches('"');
    let mut parts = path.split('/');
    if parts.next() != Some("zero") || parts.next() != Some("intents") {
        return None;
    }
    let folder = parts.next()?;
    let file = parts.next()?;
    if parts.next().is_some() || folder != "complete" || !file.ends_with(".md") {
        return None;
    }
    let stem = &file[..file.len() - 3];
    let (num, rest) = stem.split_once('-')?;
    if rest.is_empty() || num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(num.to_string())
}

fn tree_answers(repo: &Path) -> (Option<bool>, Option<bool>) {
    let clean = match Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["status", "--porcelain"])
        .output()
    {
        Ok(out) if out.status.success() => String::from_utf8(out.stdout)
            .ok()
            .map(|text| text.lines().all(|line| line.trim().is_empty())),
        _ => None,
    };
    let pushed = match Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-list", "--count", "origin/main..HEAD"])
        .output()
    {
        Ok(out) if out.status.success() => String::from_utf8(out.stdout)
            .ok()
            .and_then(|text| text.trim().parse::<u64>().ok())
            .map(|count| count == 0),
        _ => None,
    };
    (clean, pushed)
}

#[derive(Serialize)]
struct LedgerJson {
    schema: u32,
    counts: Counts,
    intents: Vec<IntentJson>,
    recent_closed: Vec<RecentClosed>,
    last_close: Option<LastClose>,
    tree: Tree,
}

#[derive(Serialize)]
struct Counts {
    total: u64,
    done: u64,
    in_progress: u64,
    planned: u64,
    blocked: u64,
    cancelled: u64,
}

#[derive(Serialize)]
struct IntentJson {
    id: String,
    title: String,
    status: String,
    blocked: bool,
    vision: String,
    gates: Gates,
    format: u32,
    depends_on: Vec<String>,
    file: String,
}

#[derive(Serialize)]
struct Gates {
    done: u64,
    total: u64,
}

#[derive(Serialize)]
struct RecentClosed {
    id: String,
    status: String,
    closed: String,
    title: String,
}

#[derive(Serialize)]
struct LastClose {
    id: String,
    status: String,
    closed: String,
    last_commit: String,
    title: String,
}

#[derive(Serialize)]
struct Tree {
    clean: Option<bool>,
    pushed: Option<bool>,
}

#[derive(Serialize)]
struct GateJson {
    label: String,
    state: String,
    text: String,
}

#[derive(Serialize)]
struct ShowClosed {
    id: String,
    status: String,
    closed: bool,
    closed_on: String,
    last_commit: String,
    title: String,
    git_show: String,
}

#[cfg(test)]
mod tests {
    use super::{build_at, show_at, Shown};
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(tag: &str) -> Scratch {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("ledger-json-{tag}-{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("open")).unwrap();
        Scratch(dir)
    }

    fn write_intent(dir: &Path, name: &str, body: &str) {
        fs::write(dir.join("open").join(name), body).unwrap();
    }

    fn intent(id: &str, status: &str, title: &str, fm: &str, body: &str) -> String {
        format!("---\nid: {id}\ntitle: \"{title}\"\nstatus: {status}\ndate: 2026-11-19\n{fm}---\n\n{body}")
    }

    fn closed_row(id: &str, status: &str, date: &str, commit: &str, title: &str) -> String {
        format!("{id}\t{status}\t{date}\t{commit}\t{title}\n")
    }

    fn key_paths(value: &Value) -> BTreeSet<String> {
        fn walk(v: &Value, prefix: &str, out: &mut BTreeSet<String>) {
            match v {
                Value::Object(map) => {
                    for (k, child) in map {
                        let path = if prefix.is_empty() {
                            k.clone()
                        } else {
                            format!("{prefix}.{k}")
                        };
                        out.insert(path.clone());
                        walk(child, &path, out);
                    }
                }
                Value::Array(items) => {
                    if let Some(first) = items.first() {
                        walk(first, &format!("{prefix}[]"), out);
                    }
                }
                _ => {}
            }
        }
        let mut out = BTreeSet::new();
        walk(value, "", &mut out);
        out
    }

    fn git(repo: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args([
                "-c",
                "user.email=int330@example.com",
                "-c",
                "user.name=int330",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .expect("git");
        assert!(
            out.status.success(),
            "{args:?}\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// INT-330 G1. Schema 1 has the top-level keys, and each value has its type.
    #[test]
    fn list_json_has_schema_1_and_the_top_level_keys() {
        let dir = scratch("keys");
        write_intent(
            &dir.0,
            "20-one.md",
            &intent(
                "20",
                "planned",
                "One",
                "format: 3\ndepends_on: [21]\n",
                "## Vision\nChristian sees the ledger.\n\n## Success Criteria\n- [x] G1 done\n      evidence: shown\n- [ ] G2 open\n",
            ),
        );
        let built = build_at(&dir.0, &dir.0.join("not-a-repo"));
        let v = &built.value;
        for key in [
            "schema",
            "counts",
            "intents",
            "recent_closed",
            "last_close",
            "tree",
        ] {
            assert!(v.get(key).is_some(), "missing {key}: {v}");
        }
        assert_eq!(v["schema"], 1);
        let counts = v.get("counts").unwrap();
        for key in [
            "total",
            "done",
            "in_progress",
            "planned",
            "blocked",
            "cancelled",
        ] {
            assert!(counts.get(key).unwrap().is_number(), "{key} in {counts}");
        }
        let row = &v["intents"][0];
        for key in [
            "id",
            "title",
            "status",
            "blocked",
            "vision",
            "gates",
            "format",
            "depends_on",
            "file",
        ] {
            assert!(row.get(key).is_some(), "intent missing {key}: {row}");
        }
        assert!(row["blocked"].is_boolean());
        assert!(row["vision"].is_string());
        assert!(row["format"].is_number());
        assert!(row["depends_on"].is_array());
        assert!(row["gates"].get("done").unwrap().is_number());
        assert!(row["gates"].get("total").unwrap().is_number());
        assert!(v["recent_closed"].is_array());
        assert!(v["tree"].get("clean").is_some());
        assert!(v["tree"].get("pushed").is_some());
    }

    /// INT-330 G1. Counts equal the rows. A planned intent whose dependency is
    /// still planned is blocked. A cancelled dependency clears, which is dep_state.
    #[test]
    fn counts_equal_the_rows_and_a_blocked_row_is_blocked() {
        let dir = scratch("counts");
        write_intent(
            &dir.0,
            "10-blocked.md",
            &intent(
                "10",
                "planned",
                "Blocked",
                "format: 3\ndepends_on: [11]\n",
                "## Vision\nFirst sentence. Second sentence.\n\n## Success Criteria\n- [x] G1 done\n      evidence: shown\n- [ ] G2 open\n",
            ),
        );
        write_intent(
            &dir.0,
            "11-open.md",
            &intent(
                "11",
                "planned",
                "Open dep",
                "format: 3\ndepends_on: []\n",
                "## Vision\nNo period here\n",
            ),
        );
        write_intent(
            &dir.0,
            "12-active.md",
            &intent("12", "in-progress", "Active", "format: 3\n", ""),
        );
        write_intent(
            &dir.0,
            "13-cleared.md",
            &intent(
                "13",
                "planned",
                "Cleared",
                "format: 3\ndepends_on: [14]\n",
                "",
            ),
        );
        write_intent(
            &dir.0,
            "14-cancelled.md",
            &intent("14", "cancelled", "Cancelled dep", "format: 1\n", ""),
        );
        write_intent(
            &dir.0,
            "15-done.md",
            &intent("15", "complete", "Done live", "format: 1\n", ""),
        );
        write_intent(
            &dir.0,
            "16-cancelled-live.md",
            &intent("16", "cancelled", "Cancelled live", "format: 1\n", ""),
        );
        let mut tsv = String::from("id\tstatus\tclosed\tlast_commit\ttitle\n");
        tsv.push_str(&closed_row("1", "complete", "2026-01-01", "aaaa", "t1"));
        tsv.push_str("not-a-row\n");
        tsv.push_str(&closed_row("2", "complete", "2026-01-02", "bbbb", "t2"));
        tsv.push_str(&closed_row("3", "complete", "2026-01-03", "cccc", "t3"));
        tsv.push_str(&closed_row("4", "complete", "2026-01-04", "dddd", "t4"));
        tsv.push_str(&closed_row("5", "complete", "2026-01-05", "eeee", "t5"));
        tsv.push_str(&closed_row("6", "cancelled", "2026-01-06", "ffff", "t6"));
        fs::write(dir.0.join("closed.tsv"), tsv).unwrap();

        let built = build_at(&dir.0, &dir.0.join("not-a-repo"));
        let v = &built.value;
        let counts = &v["counts"];
        assert_eq!(
            counts["done"], 6,
            "live complete plus five closed.tsv completes"
        );
        assert_eq!(counts["in_progress"], 1);
        assert_eq!(counts["planned"], 3);
        assert_eq!(counts["blocked"], 1);
        assert_eq!(
            counts["cancelled"], 3,
            "two live cancelled intents and one closed row"
        );
        assert_eq!(
            counts["total"],
            counts["done"].as_u64().unwrap()
                + counts["in_progress"].as_u64().unwrap()
                + counts["planned"].as_u64().unwrap()
        );
        let intents = v["intents"].as_array().unwrap();
        let planned = intents
            .iter()
            .filter(|row| row["status"] == "planned")
            .count();
        let active = intents
            .iter()
            .filter(|row| row["status"] == "in-progress")
            .count();
        let blocked = intents.iter().filter(|row| row["blocked"] == true).count();
        assert_eq!(planned as u64, counts["planned"].as_u64().unwrap());
        assert_eq!(active as u64, counts["in_progress"].as_u64().unwrap());
        assert_eq!(blocked as u64, counts["blocked"].as_u64().unwrap());
        let row10 = intents.iter().find(|row| row["id"] == "10").unwrap();
        assert_eq!(row10["blocked"], true);
        assert_eq!(row10["vision"], "First sentence.");
        assert_eq!(row10["gates"]["done"], 1);
        assert_eq!(row10["gates"]["total"], 2);
        assert_eq!(row10["format"], 3);
        assert_eq!(row10["depends_on"], serde_json::json!(["11"]));
        assert_eq!(row10["file"], "open/10-blocked.md");
        let row13 = intents.iter().find(|row| row["id"] == "13").unwrap();
        assert_eq!(row13["blocked"], false, "a cancelled dependency clears");
        let row11 = intents.iter().find(|row| row["id"] == "11").unwrap();
        assert_eq!(row11["vision"], "No period here");
        let ids: Vec<&str> = intents
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, ["12", "10", "11", "13", "14", "16"]);
        let recent: Vec<&str> = v["recent_closed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect();
        assert_eq!(recent, ["6", "5", "4", "3", "2"]);
        assert_eq!(v["last_close"]["id"], "6");
        assert_eq!(v["last_close"]["status"], "cancelled");
        assert_eq!(v["last_close"]["last_commit"], "ffff");
        assert!(
            built.warnings.iter().any(|w| w.contains("closed.tsv")),
            "a malformed closed.tsv row is a warning: {:?}",
            built.warnings
        );
    }

    /// INT-330 G1. A file with no format field takes the format 1 rules. No
    /// Vision section is an empty string, not a missing key.
    #[test]
    fn a_format_1_file_with_no_vision_has_an_empty_vision() {
        let dir = scratch("vision");
        write_intent(
            &dir.0,
            "1-old.md",
            &intent(
                "1",
                "planned",
                "Old",
                "",
                "## The Problem\nNo vision heading.\n",
            ),
        );
        let built = build_at(&dir.0, &dir.0.join("not-a-repo"));
        let row = &built.value["intents"][0];
        assert_eq!(row["vision"], "");
        assert_eq!(
            row["format"], 0,
            "an undeclared format is the format 1 rules"
        );
    }

    /// INT-330 G1. A file that cannot be read stays out of the JSON and is
    /// named on the warning list. The JSON itself parses.
    #[test]
    fn an_unreadable_file_keeps_stdout_json_and_warns() {
        let dir = scratch("unread");
        write_intent(
            &dir.0,
            "1-ok.md",
            &intent("1", "planned", "Ok", "format: 3\n", ""),
        );
        let secret = dir.0.join("open/2-secret.md");
        fs::write(&secret, "secret\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&secret).unwrap().permissions();
        perms.set_mode(0);
        fs::set_permissions(&secret, perms).unwrap();
        let built = build_at(&dir.0, &dir.0.join("not-a-repo"));
        assert!(built.value["schema"] == 1, "stdout JSON is schema 1");
        assert_eq!(built.value["intents"][0]["id"], "1");
        let text = serde_json::to_string(&built.value).unwrap();
        let parsed: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["schema"], 1);
        assert!(
            built
                .warnings
                .iter()
                .any(|w| w.contains("2-secret.md") && w.contains("could not be read")),
            "the unreadable file is a warning: {:?}",
            built.warnings
        );
    }

    /// INT-330 G1. When git cannot answer, tree.clean and tree.pushed are null.
    #[test]
    fn git_unknown_gives_tree_nulls() {
        let dir = scratch("nogit");
        write_intent(
            &dir.0,
            "1-one.md",
            &intent("1", "planned", "One", "format: 3\n", ""),
        );
        let built = build_at(&dir.0, &dir.0.join("not-a-repo"));
        let tree = built.value.get("tree").expect("tree");
        assert!(tree.get("clean").is_some(), "clean key: {tree}");
        assert!(tree["clean"].is_null(), "{tree}");
        assert!(tree.get("pushed").is_some(), "pushed key: {tree}");
        assert!(tree["pushed"].is_null(), "{tree}");
    }

    /// A real git answer is a bool. A missing origin/main does not become false.
    #[test]
    fn a_clean_repo_is_true_and_a_missing_origin_is_null() {
        let dir = scratch("git");
        write_intent(
            &dir.0,
            "1-one.md",
            &intent("1", "planned", "One", "format: 3\n", ""),
        );
        git(&dir.0, &["init", "-q"]);
        git(&dir.0, &["add", "open"]);
        git(&dir.0, &["commit", "-q", "-m", "ledger"]);
        let built = build_at(&dir.0, &dir.0);
        assert_eq!(built.value["tree"]["clean"], true);
        assert!(
            built.value["tree"]["pushed"].is_null(),
            "no origin is null, not false: {}",
            built.value["tree"]
        );
        fs::write(dir.0.join("open/dirty.txt"), "x\n").unwrap();
        let dirty = build_at(&dir.0, &dir.0);
        assert_eq!(dirty.value["tree"]["clean"], false);
        assert!(dirty.value["tree"]["pushed"].is_null());
    }

    /// INT-330 G3. fixture.json has exactly the keys `list --json` produces.
    #[test]
    fn fixture_json_has_the_keys_list_json_produces() {
        let dir = scratch("fixture");
        write_intent(
            &dir.0,
            "3-live.md",
            &intent(
                "3",
                "in-progress",
                "Live",
                "format: 3\ndepends_on: []\n",
                "## Vision\nA sentence.\n\n## Success Criteria\n- [ ] G1 open\n",
            ),
        );
        fs::write(
            dir.0.join("closed.tsv"),
            closed_row("2", "complete", "2026-11-18", "abcdef", "Closed"),
        )
        .unwrap();
        let built = build_at(&dir.0, &dir.0.join("not-a-repo"));
        let fixture_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../panels/ledger/fixture.json");
        let fixture: Value = serde_json::from_str(
            &fs::read_to_string(&fixture_path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", fixture_path.display())),
        )
        .expect("fixture.json is JSON");
        assert_eq!(
            key_paths(&built.value),
            key_paths(&fixture),
            "fixture keys are not the keys list --json produces"
        );
    }

    /// INT-330 G2. An open id carries gates_list. A closed id has "closed": true
    /// and the git show command. An unknown id is not a document.
    #[test]
    fn show_json_answers_an_open_id_a_closed_id_and_an_unknown_id() {
        let dir = scratch("show");
        let repo = dir.0.join("repo");
        let intents = repo.join("zero/intents");
        fs::create_dir_all(intents.join("open")).unwrap();
        fs::write(
            intents.join("open/8-gone.md"),
            intent("8", "complete", "Gone", "format: 1\n", "old text\n"),
        )
        .unwrap();
        git(&repo, &["init", "-q"]);
        git(&repo, &["add", "zero/intents/open/8-gone.md"]);
        git(&repo, &["commit", "-q", "-m", "close 8"]);
        let hash = String::from_utf8(
            Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(["rev-parse", "HEAD"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();
        let hash = hash.trim().to_string();
        fs::remove_file(intents.join("open/8-gone.md")).unwrap();
        fs::write(
            intents.join("closed.tsv"),
            closed_row("8", "complete", "2026-11-19", &hash, "Gone"),
        )
        .unwrap();
        write_intent(
            &intents,
            "9-open.md",
            &intent(
                "9",
                "planned",
                "Open",
                "format: 3\ndepends_on: []\n",
                "## Vision\nStill open.\n\n## Success Criteria\n- [ ] G1 the open gate\n- [x] G2 the done gate\n      evidence: shown\n",
            ),
        );

        match show_at(&intents, &repo, "9") {
            Shown::Value(value) => {
                assert_eq!(value["id"], "9");
                assert_eq!(value["vision"], "Still open.");
                let gates = value["gates_list"].as_array().expect("gates_list");
                assert_eq!(gates[0]["label"], "G1");
                assert_eq!(gates[0]["state"], "open");
                assert!(gates[0]["text"].as_str().unwrap().contains("open gate"));
                assert_eq!(gates[1]["label"], "G2");
                assert_eq!(gates[1]["state"], "done");
            }
            Shown::Unknown => panic!("open id 9 was unknown"),
        }

        match show_at(&intents, &repo, "8") {
            Shown::Value(value) => {
                assert_eq!(value["closed"], true);
                assert_eq!(value["closed_on"], "2026-11-19");
                assert_eq!(value["id"], "8");
                let cmd = value["git_show"].as_str().unwrap();
                assert!(cmd.starts_with("git show "), "{cmd}");
                assert!(cmd.contains(&hash), "{cmd}");
            }
            Shown::Unknown => panic!("closed id 8 was unknown"),
        }

        assert!(matches!(show_at(&intents, &repo, "404"), Shown::Unknown));
    }
}
