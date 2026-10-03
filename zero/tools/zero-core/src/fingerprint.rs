//! INT-269 -- the fingerprint collector. The ONE place Project 0 asks whether this machine and
//! this tree are the ones it thinks they are. docs/FINGERPRINT.md is the flow; AGENTS.md holds
//! the rules.
//!
//! Nine declared inputs, read from plain files: no subprocess, no privilege. An input that cannot
//! be read is MISSING, never a default, and a record with anything missing compares as
//! UNDETERMINED. The digest covers the declared inputs only, with a hash written here (FNV-1a),
//! because std's DefaultHasher may change between compiler versions.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The declared inputs, in the order they are digested. Adding one is a contract change: an
/// intent, a red-first test, and an update to docs/FINGERPRINT.md.
pub const DECLARED: [&str; 9] = [
    "identity.machine_id",
    "identity.hostname",
    "machine.arch",
    "machine.os_id",
    "machine.board",
    "machine.cpu",
    "process.uid",
    "tree.repo",
    "tree.dirs",
];

/// The record's schema. A record written under another schema answers a different question:
/// compare is UNDETERMINED naming `schema`, never FAIL (INT-270). Raise it in the same change
/// that alters what a declared input means. A record with no schema line is schema 1, the
/// INT-269 shape.
pub const SCHEMA: &str = "2";
pub const SCHEMA_KEY: &str = "schema";

/// The five directories that must be real directories, relative to home.
pub const REAL_DIRS: [&str; 5] = [
    ".local/state/zero",
    ".config/zero",
    ".cache/zero",
    ".local/share/zero",
    ".config/nsh",
];

/// Where each input is read from. `host()` is this machine; tests point it at a fake one.
#[derive(Debug, Clone)]
pub struct Sources {
    pub etc: PathBuf,
    pub proc_root: PathBuf,
    pub sys: PathBuf,
    pub home: PathBuf,
}

impl Sources {
    pub fn host() -> Self {
        Sources {
            etc: PathBuf::from("/etc"),
            proc_root: PathBuf::from("/proc"),
            sys: PathBuf::from("/sys"),
            home: std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_default(),
        }
    }
}

/// Every declared input as a value, or None when it could not be read (the missing-set).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub facts: BTreeMap<String, Option<String>>,
}

/// PASS: all read, all equal. FAIL: all read, some differ (named). UNDETERMINED: any unread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail(Vec<String>),
    Undetermined(Vec<String>),
}

impl Record {
    /// The declared inputs this record could not read, in declared order.
    pub fn missing(&self) -> Vec<String> {
        let mut out = Vec::new();
        for k in DECLARED {
            match self.facts.get(k) {
                Some(Some(_)) => {}
                _ => out.push(k.to_string()),
            }
        }
        out
    }

    /// The digest over the declared inputs only. None while anything is missing: a digest of a
    /// partial read would look like an answer.
    pub fn digest(&self) -> Option<String> {
        if !self.missing().is_empty() {
            return None;
        }
        let mut text = String::new();
        for k in DECLARED {
            let v = self.facts.get(k).and_then(|v| v.as_deref()).unwrap_or("");
            text.push_str(k);
            text.push('=');
            text.push_str(v);
            text.push('\n');
        }
        Some(format!("{:016x}", fnv1a64(text.as_bytes())))
    }

    /// The schema this record was written under. No schema line is schema 1, the INT-269 shape.
    pub fn schema(&self) -> String {
        match self.facts.get(SCHEMA_KEY) {
            Some(Some(v)) => v.clone(),
            _ => "1".to_string(),
        }
    }

    /// One line per fact: `key=value`, or the bare key for a missing one.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for (k, v) in &self.facts {
            match v {
                Some(v) => out.push_str(&format!("{k}={v}\n")),
                None => out.push_str(&format!("{k}\n")),
            }
        }
        out
    }

    pub fn from_text(text: &str) -> Record {
        let mut facts = BTreeMap::new();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            match line.split_once('=') {
                Some((k, v)) => facts.insert(k.to_string(), Some(v.to_string())),
                None => facts.insert(line.to_string(), None),
            };
        }
        Record { facts }
    }
}

/// FNV-1a, 64-bit. Written here so the digest is the same on every compiler, forever.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Missing on either side is UNDETERMINED -- a gap is never filled with a default. Otherwise
/// every declared input is compared, and the ones that differ are named.
/// A record from another schema answers a different question: UNDETERMINED naming `schema`.
pub fn compare(live: &Record, expected: &Record) -> Outcome {
    if expected.schema() != SCHEMA {
        return Outcome::Undetermined(vec![SCHEMA_KEY.to_string()]);
    }
    let mut missing = live.missing();
    for k in expected.missing() {
        if !missing.contains(&k) {
            missing.push(k);
        }
    }
    if !missing.is_empty() {
        return Outcome::Undetermined(missing);
    }
    let mut differ = Vec::new();
    for k in DECLARED {
        if live.facts.get(k) != expected.facts.get(k) {
            differ.push(k.to_string());
        }
    }
    if differ.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail(differ)
    }
}

/// One line per input that keeps the answer from PASS: its record and live value, what could not
/// be read, or why the record answers a different question. Empty on PASS.
pub fn explain(live: &Record, expected: &Record) -> Vec<String> {
    let schema = expected.schema();
    if schema != SCHEMA {
        return vec![format!(
            "the record was written by fingerprint schema {schema}; this collector is schema {SCHEMA}, so its fields answer a different question -- run: core fingerprint record"
        )];
    }
    let mut out = Vec::new();
    for k in DECLARED {
        let was = expected.facts.get(k).cloned().flatten();
        let now = live.facts.get(k).cloned().flatten();
        match (was, now) {
            (_, None) => out.push(format!("{k}: could not be read now")),
            (None, Some(_)) => out.push(format!("{k}: not in the record")),
            (Some(was), Some(now)) if was != now => {
                out.push(format!("{k}: record {was}, now {now}"))
            }
            _ => {}
        }
    }
    out
}

pub fn collect() -> Record {
    collect_from(&Sources::host())
}

pub fn collect_from(s: &Sources) -> Record {
    let uid = read_trim(&s.proc_root.join("self/status"))
        .and_then(|t| field(&t, "Uid"))
        .and_then(|v| v.split_whitespace().next().map(str::to_string));
    let entries: [(&str, Option<String>); 9] = [
        ("identity.machine_id", read_trim(&s.etc.join("machine-id"))),
        (
            "identity.hostname",
            read_trim(&s.proc_root.join("sys/kernel/hostname")),
        ),
        ("machine.arch", Some(std::env::consts::ARCH.to_string())),
        (
            "machine.os_id",
            read_trim(&s.etc.join("os-release")).and_then(|t| os_id(&t)),
        ),
        ("machine.board", board(&s.sys)),
        (
            "machine.cpu",
            read_trim(&s.proc_root.join("cpuinfo")).and_then(|t| field(&t, "model name")),
        ),
        ("process.uid", uid),
        ("tree.repo", repo(&s.home)),
        ("tree.dirs", dirs(&s.home)),
    ];
    Record {
        facts: entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .chain(std::iter::once((
                SCHEMA_KEY.to_string(),
                Some(SCHEMA.to_string()),
            )))
            .collect(),
    }
}

fn read_trim(p: &Path) -> Option<String> {
    let t = std::fs::read_to_string(p).ok()?;
    let t = t.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// The value after the first `key:` line, as in /proc/cpuinfo and /proc/self/status.
fn field(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim() == key {
                let v = v.trim();
                return if v.is_empty() {
                    None
                } else {
                    Some(v.to_string())
                };
            }
        }
    }
    None
}

fn os_id(os_release: &str) -> Option<String> {
    for line in os_release.lines() {
        if let Some(v) = line.strip_prefix("ID=") {
            let v = v.trim().trim_matches('"');
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn board(sys: &Path) -> Option<String> {
    let vendor = read_trim(&sys.join("class/dmi/id/board_vendor"))?;
    let product = read_trim(&sys.join("class/dmi/id/product_name"))?;
    Some(format!("{vendor} / {product}"))
}

/// The repository's identity: the inode of ~/0-core and its origin URL. Not its device number:
/// btrfs hands a subvolume a new one at every mount (58, then 59, across one boot on
/// 2026-10-01), and the fingerprint moved with nothing changed (INT-270). The URL is read from
/// .git/config. A home that is not an absolute path is unread, never the current directory.
fn repo(home: &Path) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    if !home.is_absolute() {
        return None;
    }
    let root = home.join("0-core");
    let meta = std::fs::metadata(&root).ok()?;
    let config = std::fs::read_to_string(root.join(".git/config")).ok()?;
    let mut in_origin = false;
    for line in config.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_origin = t == "[remote \"origin\"]";
            continue;
        }
        if in_origin {
            if let Some((k, v)) = t.split_once('=') {
                if k.trim() == "url" && !v.trim().is_empty() {
                    return Some(format!("{} {}", meta.ino(), v.trim()));
                }
            }
        }
    }
    None
}

/// Each of the five real directories as dir, link, other or absent. Absent is a read FACT (and a
/// FAIL against a record that had it); a directory that cannot be examined is unread.
fn dirs(home: &Path) -> Option<String> {
    if !home.is_absolute() {
        return None;
    }
    let mut parts = Vec::new();
    for d in REAL_DIRS {
        let state = match std::fs::symlink_metadata(home.join(d)) {
            Ok(m) if m.file_type().is_symlink() => "link",
            Ok(m) if m.is_dir() => "dir",
            Ok(_) => "other",
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => "absent",
            Err(_) => return None,
        };
        parts.push(format!("{d}:{state}"));
    }
    Some(parts.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A whole fake machine in a temp directory: nothing here reads the real one.
    fn fake(name: &str) -> Sources {
        let root = std::env::temp_dir().join(format!("zero-fp-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&root);
        let files = [
            ("etc/machine-id", "0123456789abcdef0123456789abcdef\n"),
            ("etc/os-release", "NAME=\"Omarchy\"\nID=omarchy\n"),
            ("proc/sys/kernel/hostname", "testhost\n"),
            ("proc/cpuinfo", "processor\t: 0\nmodel name\t: Test CPU 9000\n"),
            ("proc/self/status", "Name:\ttest\nUid:\t1000\t1000\t1000\t1000\n"),
            ("sys/class/dmi/id/board_vendor", "Framework\n"),
            ("sys/class/dmi/id/product_name", "Laptop 16\n"),
            (
                "home/0-core/.git/config",
                "[core]\n\tbare = false\n[remote \"origin\"]\n\turl = git@github.com:WidkidoneR2/0-Core.git\n",
            ),
        ];
        for (rel, text) in files {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
        for d in REAL_DIRS {
            std::fs::create_dir_all(root.join("home").join(d)).unwrap();
        }
        Sources {
            etc: root.join("etc"),
            proc_root: root.join("proc"),
            sys: root.join("sys"),
            home: root.join("home"),
        }
    }

    /// INT-270: no input may read a number the kernel hands out per mount or per boot. btrfs gave
    /// ~/0-core device 58, then 59, across one boot on 2026-10-01, and tree.repo went red.
    #[test]
    fn no_input_reads_a_number_the_kernel_assigns_per_mount_or_boot() {
        let src = include_str!("fingerprint.rs");
        let collector = src.split("#[cfg(test)]").next().unwrap_or(src);
        for banned in [
            ".dev()",
            ".rdev()",
            "st_dev",
            "boot_id",
            "SystemTime",
            "Instant",
            "/proc/uptime",
            "btime",
        ] {
            assert!(
                !collector.contains(banned),
                "the collector reads `{banned}` -- a value that moves across a boot or a mount is not identity"
            );
        }
    }

    #[test]
    fn the_repo_is_its_inode_and_origin_url() {
        use std::os::unix::fs::MetadataExt;
        let s = fake("repo");
        let ino = std::fs::metadata(s.home.join("0-core")).unwrap().ino();
        assert_eq!(
            fact(&collect_from(&s), "tree.repo"),
            Some(format!("{ino} git@github.com:WidkidoneR2/0-Core.git"))
        );
    }

    #[test]
    fn a_record_from_another_schema_is_undetermined_never_fail() {
        let s = fake("schema");
        let live = collect_from(&s);
        assert_eq!(fact(&live, SCHEMA_KEY).as_deref(), Some(SCHEMA));
        let mut old = live.clone();
        old.facts.remove(SCHEMA_KEY);
        old.facts.insert(
            "tree.repo".to_string(),
            Some("58:1 git@github.com:WidkidoneR2/0-Core.git".to_string()),
        );
        let old = Record::from_text(&old.to_text());
        assert_eq!(
            compare(&live, &old),
            Outcome::Undetermined(vec![SCHEMA_KEY.to_string()])
        );
        let why = explain(&live, &old).join("\n");
        assert!(
            why.contains("schema 1") && why.contains("core fingerprint record"),
            "{why}"
        );
    }

    #[test]
    fn a_fail_shows_the_record_and_the_live_value() {
        let s = fake("explain");
        let expected = collect_from(&s);
        assert!(explain(&expected, &expected).is_empty());
        std::fs::write(s.proc_root.join("sys/kernel/hostname"), "otherhost\n").unwrap();
        let live = collect_from(&s);
        assert_eq!(
            explain(&live, &expected),
            vec!["identity.hostname: record testhost, now otherhost".to_string()]
        );
        std::fs::remove_file(s.etc.join("machine-id")).unwrap();
        let gap = collect_from(&s);
        assert!(explain(&gap, &expected)
            .contains(&"identity.machine_id: could not be read now".to_string()));
    }

    fn fact(r: &Record, k: &str) -> Option<String> {
        r.facts.get(k).cloned().flatten()
    }

    #[test]
    fn every_declared_input_is_read_on_a_whole_machine() {
        let r = collect_from(&fake("whole"));
        assert!(r.missing().is_empty(), "missing: {:?}", r.missing());
        assert!(r.digest().is_some());
        assert_eq!(fact(&r, "identity.hostname").as_deref(), Some("testhost"));
        assert_eq!(fact(&r, "process.uid").as_deref(), Some("1000"));
        assert_eq!(
            fact(&r, "machine.board").as_deref(),
            Some("Framework / Laptop 16")
        );
    }

    #[test]
    fn an_unreadable_input_is_missing_and_the_outcome_undetermined() {
        let s = fake("unreadable");
        let expected = collect_from(&s);
        std::fs::remove_file(s.etc.join("machine-id")).unwrap();
        let live = collect_from(&s);
        let gap = vec!["identity.machine_id".to_string()];
        assert_eq!(live.missing(), gap);
        assert_eq!(live.digest(), None);
        assert_eq!(compare(&live, &expected), Outcome::Undetermined(gap));
    }

    #[test]
    fn one_changed_input_is_fail_naming_it() {
        let s = fake("changed");
        let expected = collect_from(&s);
        std::fs::write(s.proc_root.join("sys/kernel/hostname"), "otherhost\n").unwrap();
        let live = collect_from(&s);
        assert_eq!(
            compare(&live, &expected),
            Outcome::Fail(vec!["identity.hostname".to_string()])
        );
    }

    #[test]
    fn a_link_where_a_real_directory_belongs_is_fail_on_the_tree() {
        let s = fake("link");
        let expected = collect_from(&s);
        let d = s.home.join(".config/nsh");
        std::fs::remove_dir(&d).unwrap();
        std::os::unix::fs::symlink(s.home.join(".config/zero"), &d).unwrap();
        let live = collect_from(&s);
        assert_eq!(
            compare(&live, &expected),
            Outcome::Fail(vec!["tree.dirs".to_string()])
        );
    }

    #[test]
    fn a_field_outside_the_declared_list_does_not_move_the_digest() {
        let r = collect_from(&fake("extra"));
        let mut named = r.clone();
        named
            .facts
            .insert("display.name".to_string(), Some("Project 0".to_string()));
        assert_eq!(r.digest(), named.digest());
        assert_eq!(compare(&named, &r), Outcome::Pass);
    }

    #[test]
    fn two_reads_of_one_machine_are_one_record() {
        let s = fake("twice");
        assert_eq!(collect_from(&s), collect_from(&s));
    }

    #[test]
    fn a_record_survives_being_written_and_read() {
        let s = fake("roundtrip");
        std::fs::remove_file(s.etc.join("machine-id")).unwrap();
        let r = collect_from(&s);
        assert_eq!(Record::from_text(&r.to_text()), r);
    }

    #[test]
    fn the_digest_function_is_pinned() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"project 0"), 0x2501_e42b_1869_9b7e);
    }
}
