//! INT-314. The brief is computed here. run and get_voice both ask this module.
//! The red bodies are the bugs: a missing git says 0, the stored sentence is spoken,
//! rank is unsorted, every observation is shown, and the habit line is silent.
use std::path::Path;

/// `4` or `unknown`. A missing git is not a zero.
pub fn commits_since_local_midnight(repo: &Path) -> String {
    let _ = repo;
    "0".to_string()
}

/// The sentence that is spoken. The stored snapshot is not a source.
pub fn brief_sentence(commit_phrase: &str, deploy_count: i64, active: &str) -> String {
    let _ = (commit_phrase, deploy_count, active);
    "0 commit(s) today".to_string()
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
    let _ = row;
    false
}

pub fn rank_room(rows: Vec<RoomRank>) -> Vec<RoomRank> {
    rows
}

pub fn rank_plain(row: &RoomRank) -> String {
    format!("INT-{} {}/{}", row.id, row.done, row.total)
}

/// (conclusion, confidence, kind). Kind is anomaly, causal, system, or normal.
pub fn pick_anomalies(rows: Vec<(String, f64, String)>) -> Vec<(String, f64, String)> {
    rows
}

pub fn anomaly_line(confidence: f64, conclusion: &str) -> String {
    let _ = (confidence, conclusion);
    String::new()
}

pub fn anomaly_is_red(kind: &str) -> bool {
    let _ = kind;
    false
}

/// `✦ cargo  →  git  12`, or nothing when the history is thin or the pair is rare.
pub fn habit_line(commands: &[String]) -> Option<String> {
    let _ = commands;
    None
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
