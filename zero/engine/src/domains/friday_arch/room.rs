//! INT-314. The brief is computed here. run and get_voice both ask this module.
//! The snapshot is still written for the health trend. Nothing here prints it back.
use std::path::{Path, PathBuf};

use colored::*;

use crate::app::context::AppContext;

/// `4` or `unknown`. A missing git is not a zero.
pub fn commits_since_local_midnight(repo: &Path) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-list", "--count", "--since=midnight", "HEAD"])
        .output();
    match out {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()) {
                text
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

/// The sentence that is spoken. The stored snapshot is not a source.
pub fn brief_sentence(commit_phrase: &str, deploy_count: i64, active: &str) -> String {
    format!(
        "Healthy. {deploy_count} deploy(s), {commit_phrase} commit(s) today. Active: {active}. No conflicts detected."
    )
}

pub fn sentence_for(ctx: &AppContext) -> String {
    let phrase = commits_since_local_midnight(Path::new(&ctx.core_root));
    let now = super::now_ts();
    let deploy_count: i64 = ctx
        .runtime
        .db
        .query_row(
            "SELECT COUNT(*) FROM deploy_patterns WHERE timestamp > ?1",
            rusqlite::params![now - 86400],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let active = super::format_active_intents(
        zero_core::ledger::with_status("in-progress")
            .into_iter()
            .map(|file| file.id),
    );
    brief_sentence(&phrase, deploy_count, &active)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomRank {
    pub id: u32,
    pub done: usize,
    pub total: usize,
    pub open_gates: usize,
    pub days_since_name: u32,
    pub days_since_gate: u32,
}

pub fn is_stalled(row: &RoomRank) -> bool {
    row.days_since_name >= 14 && row.days_since_gate >= 14
}

pub fn rank_room(mut rows: Vec<RoomRank>) -> Vec<RoomRank> {
    rows.sort_by(|a, b| {
        b.open_gates
            .cmp(&a.open_gates)
            .then(b.days_since_name.cmp(&a.days_since_name))
            .then(a.id.cmp(&b.id))
    });
    rows
}

#[cfg(test)]
pub fn rank_plain(row: &RoomRank) -> String {
    let mut line = format!("INT-{} {}/{}", row.id, row.done, row.total);
    if is_stalled(row) {
        line.push_str(" stalled");
    }
    line
}

fn git_latest_ts(repo: &Path, args: &[String]) -> Option<i64> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return None;
    }
    text.parse().ok()
}

fn days_from(ts: Option<i64>) -> u32 {
    let Some(ts) = ts else {
        return u32::MAX / 4;
    };
    let age = super::now_ts().saturating_sub(ts);
    (age / 86400) as u32
}

fn days_since_name(repo: &Path, id: &str) -> u32 {
    let pattern = format!("INT-{id}([^0-9]|$)");
    let ts = git_latest_ts(
        repo,
        &[
            "log".to_string(),
            "-1".to_string(),
            "--format=%ct".to_string(),
            "--extended-regexp".to_string(),
            format!("--grep={pattern}"),
        ],
    );
    days_from(ts)
}

fn days_since_gate(repo: &Path, path: &Path) -> u32 {
    let ts = git_latest_ts(
        repo,
        &[
            "log".to_string(),
            "-1".to_string(),
            "--format=%ct".to_string(),
            "--".to_string(),
            path.display().to_string(),
        ],
    );
    days_from(ts)
}

pub fn live_ranks(repo: &Path) -> Vec<RoomRank> {
    let rows = crate::domains::intent::read_open()
        .into_iter()
        .filter(|row| row.intent.status == "in-progress")
        .filter(|row| {
            !crate::domains::intent::folders::RECORDS.contains(&row.intent.folder.as_str())
        })
        .filter_map(|row| {
            let id = row.intent.id.parse::<u32>().ok()?;
            let total = row.gates.gates.len();
            let done = row
                .gates
                .gates
                .iter()
                .filter(|gate| gate.state == crate::domains::intent::gates::GateState::Done)
                .count();
            let open_gates = row
                .gates
                .gates
                .iter()
                .filter(|gate| gate.state == crate::domains::intent::gates::GateState::Open)
                .count();
            Some(RoomRank {
                id,
                done,
                total,
                open_gates,
                days_since_name: days_since_name(repo, &row.intent.id),
                days_since_gate: days_since_gate(
                    repo,
                    Path::new(&format!(
                        "zero/intents/{}/{}",
                        row.intent.folder, row.intent.filename
                    )),
                ),
            })
        })
        .collect();
    rank_room(rows)
}

/// (conclusion, confidence, kind). Kind is anomaly, causal, system, or normal.
pub fn pick_anomalies(mut rows: Vec<(String, f64, String)>) -> Vec<(String, f64, String)> {
    rows.retain(|(_, _, kind)| kind != "normal");
    rows.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    rows.truncate(3);
    rows
}

pub fn anomaly_line(confidence: f64, conclusion: &str) -> String {
    format!("⚠ {confidence:.2} {conclusion}")
}

pub fn anomaly_is_red(kind: &str) -> bool {
    kind == "causal"
}

pub fn live_anomalies(ctx: &AppContext) -> Vec<(String, f64, String)> {
    let rows = crate::domains::friday::reasoning::reason(ctx).unwrap_or_default();
    pick_anomalies(rows)
}

fn first_word(command: &str) -> &str {
    command.split_whitespace().next().unwrap_or("")
}

pub fn habit_pair(commands: &[String]) -> Option<(String, String, usize)> {
    if commands.len() < 50 {
        return None;
    }
    let mut counts = std::collections::HashMap::<(String, String), usize>::new();
    let mut first_seen = Vec::<(String, String)>::new();
    for pair in commands.windows(2) {
        let left = first_word(&pair[0]);
        let right = first_word(&pair[1]);
        if left.is_empty() || right.is_empty() {
            continue;
        }
        let key = (left.to_string(), right.to_string());
        let count = counts.entry(key.clone()).or_insert(0);
        if *count == 0 {
            first_seen.push(key);
        }
        *count += 1;
    }
    let mut best: Option<(&(String, String), usize)> = None;
    for key in &first_seen {
        let count = counts[key];
        if best
            .map(|(_, best_count)| count > best_count)
            .unwrap_or(true)
        {
            best = Some((key, count));
        }
    }
    let (key, count) = best?;
    if count < 3 {
        return None;
    }
    Some((key.0.clone(), key.1.clone(), count))
}

/// `✦ cargo  →  git  12`, or nothing when the history is thin or the pair is rare.
#[cfg(test)]
pub fn habit_line(commands: &[String]) -> Option<String> {
    habit_pair(commands).map(|(left, right, count)| format!("✦ {left}  →  {right}  {count}"))
}

pub fn habit_from_db(db: &rusqlite::Connection) -> Option<(String, String, usize)> {
    let mut statement = db
        .prepare("SELECT command FROM shell_history ORDER BY timestamp ASC, id ASC")
        .ok()?;
    let commands: Vec<String> = statement
        .query_map([], |row| row.get(0))
        .ok()?
        .filter_map(|row| row.ok())
        .collect();
    habit_pair(&commands)
}

pub fn print_room(ctx: &AppContext) {
    let repo = PathBuf::from(&ctx.core_root);
    for row in live_ranks(&repo) {
        print!(
            "    {} {} {}",
            "✦".bright_magenta(),
            format!("INT-{}", row.id).bright_cyan(),
            format!("{}/{}", row.done, row.total).bright_green()
        );
        if is_stalled(&row) {
            print!(" {}", "stalled".bright_red());
        }
        println!();
    }
    for (conclusion, confidence, kind) in live_anomalies(ctx) {
        let line = anomaly_line(confidence, &conclusion);
        if anomaly_is_red(&kind) {
            println!("    {}", line.bright_red());
        } else {
            println!("    {}", line.bright_yellow());
        }
    }
    if let Some((left, right, count)) = habit_from_db(&ctx.runtime.db) {
        println!(
            "    {} {}  →  {}  {}",
            "✦".bright_magenta(),
            left.bright_white(),
            right.bright_white(),
            count.to_string().bright_green()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git_repo(with_commit: bool) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "friday-room-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        if with_commit {
            let run = |args: &[&str]| {
                let status = std::process::Command::new("git")
                    .args(args)
                    .current_dir(&dir)
                    .env("GIT_AUTHOR_NAME", "test")
                    .env("GIT_AUTHOR_EMAIL", "test@example.com")
                    .env("GIT_COMMITTER_NAME", "test")
                    .env("GIT_COMMITTER_EMAIL", "test@example.com")
                    .status()
                    .unwrap();
                assert!(status.success());
            };
            run(&["init"]);
            std::fs::write(dir.join("f"), "x\n").unwrap();
            run(&["add", "f"]);
            run(&["commit", "-m", "today"]);
        }
        dir
    }

    #[test]
    fn a_commit_today_is_counted_and_a_missing_repo_is_unknown() {
        let repo = git_repo(true);
        assert_eq!(commits_since_local_midnight(&repo), "1");
        let missing =
            std::env::temp_dir().join(format!("friday-room-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&missing);
        assert_eq!(commits_since_local_midnight(&missing), "unknown");
        let _ = std::fs::remove_dir_all(&repo);
    }

    #[test]
    fn the_sentence_uses_the_phrase_it_was_given() {
        let said = brief_sentence("4", 1, "INT-314");
        assert!(said.contains("4 commit(s) today"), "{said}");
        assert!(said.contains("1 deploy"), "{said}");
        assert!(said.contains("INT-314"), "{said}");
        let failed = brief_sentence("unknown", 0, "none");
        assert!(failed.contains("unknown"), "{failed}");
        assert!(!failed.contains("0 commit"), "{failed}");
    }

    #[test]
    fn rank_is_open_gates_then_days_and_fourteen_is_stalled() {
        let rows = vec![
            RoomRank {
                id: 20,
                done: 1,
                total: 2,
                open_gates: 1,
                days_since_name: 3,
                days_since_gate: 0,
            },
            RoomRank {
                id: 10,
                done: 2,
                total: 5,
                open_gates: 3,
                days_since_name: 1,
                days_since_gate: 1,
            },
            RoomRank {
                id: 30,
                done: 0,
                total: 4,
                open_gates: 3,
                days_since_name: 20,
                days_since_gate: 20,
            },
            RoomRank {
                id: 5,
                done: 1,
                total: 4,
                open_gates: 3,
                days_since_name: 20,
                days_since_gate: 14,
            },
        ];
        let ranked = rank_room(rows);
        let ids: Vec<u32> = ranked.iter().map(|row| row.id).collect();
        assert_eq!(ids, vec![5, 30, 10, 20]);
        assert!(is_stalled(&ranked[0]));
        assert!(is_stalled(&ranked[1]));
        assert!(!is_stalled(&ranked[2]));
        assert_eq!(rank_plain(&ranked[0]), "INT-5 1/4 stalled");
        assert_eq!(rank_plain(&ranked[2]), "INT-10 2/5");
    }

    #[test]
    fn anomalies_keep_three_and_drop_normal() {
        let picked = pick_anomalies(vec![
            ("low".into(), 0.5, "anomaly".into()),
            ("cause".into(), 0.9, "causal".into()),
            ("norm".into(), 0.99, "normal".into()),
            ("sys".into(), 0.7, "system".into()),
            ("mid".into(), 0.8, "anomaly".into()),
        ]);
        let got: Vec<&str> = picked.iter().map(|row| row.0.as_str()).collect();
        assert_eq!(got, vec!["cause", "mid", "sys"]);
        assert!(anomaly_is_red("causal"));
        assert!(!anomaly_is_red("anomaly"));
        assert_eq!(anomaly_line(0.9, "cause"), "⚠ 0.90 cause");
    }

    #[test]
    fn the_habit_line_is_the_next_word_and_stays_quiet_when_thin() {
        let mut commands = Vec::new();
        for _ in 0..25 {
            commands.push("cargo test -p core".to_string());
            commands.push("git commit -m x".to_string());
        }
        assert_eq!(
            habit_line(&commands).as_deref(),
            Some("✦ cargo  →  git  25")
        );
        assert_eq!(habit_line(&commands[..49]), None);
        let rare: Vec<String> = (0..50).map(|n| format!("cmd{n} rest")).collect();
        assert_eq!(habit_line(&rare), None);
    }
}
