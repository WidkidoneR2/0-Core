//! INT-330. The intent ledger as JSON schema 1.
//!
//! Reads the open rows, their gates, closed.tsv, the completed-id set and
//! dep_state. Writes nothing. A value that could not be read is null.

use std::path::Path;

/// JSON for `core intent list --json`, plus the warnings that belong on stderr.
pub struct Built {
    pub value: serde_json::Value,
    pub warnings: Vec<String>,
}

/// The ledger under `dir`, with git answers from `repo`.
///
/// The body is filled in after the red tests. Until then it returns an empty
/// object so the tests compile and fail.
pub fn build_at(dir: &Path, repo: &Path) -> Built {
    let _ = (dir, repo);
    Built {
        value: serde_json::json!({}),
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::build_at;
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
            counts["cancelled"], 2,
            "one live cancelled and one closed row"
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
}
