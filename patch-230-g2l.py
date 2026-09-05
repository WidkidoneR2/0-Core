#!/usr/bin/env python3
"""
INT-230 G2 -- the last discovery sites.

NINE CENSUS ENTRIES, SIX REAL MIGRATIONS:

  nl.rs:460      A COMMENT. The census matches TEXT, not syntax, so a `paths::`
                 mention in prose counts as a call. registry_dir is ONE call,
                 not two. Another limit of the artifact worth knowing.

  db.rs:68       core_root_string() sits beside state_db() in the database
                 opener -- that is how the shell finds its OWN database, which
                 is core shell state, not 0-Core discovery. MISCLASSIFIED, same
                 as exec.rs's protected paths.

  main.rs:3126   STAYS BY RULING (the 43-vs-40 category scoping).

MIGRATING:
  3784   @intents in expand_path -- sibling of the @rust fixed last pass, and it
         gets the same treatment: return the shortcut unexpanded rather than the
         empty string.
  8144   fixed list [future, complete, in-progress]
  11691  ⚠️ A SEVENTH INTENT READER. It walks in-progress/complete/future and
         maps each FOLDER to a status label -- the folder-implies-status
         assumption the adapter rejected in favour of reading the frontmatter
         field. It keeps its own logic and takes only the root, because it needs
         complete/ in full and ledger() carries only complete IDS. Widening the
         adapter for one caller would load every completed intent on every
         prompt render.
  11768  count_md over a caller-supplied lifecycle subfolder. ⚠️ THE ONE PLACE A
         ZERO IS ALLOWED, and only because its caller is a version panel where
         "0 intents" prints beside "unknown version" -- absence reading as
         absence rather than as a measurement.
  16517  complete/ as a string.
  nl.rs:477  registry_dir for shell-patterns.toml.
"""

import io
import os
import sys

SRC = "faelight/rust-tools/novashell/src"


def die(msg):
    print("ABORT: " + msg, file=sys.stderr)
    sys.exit(1)


def read(path):
    with io.open(path, "r", encoding="utf-8") as fh:
        return fh.read()


def write(path, text):
    with io.open(path, "w", encoding="utf-8") as fh:
        fh.write(text)


def swap(text, path, old, new, count, label):
    n = text.count(old)
    if n != count:
        die(path + " [" + label + "]: matched " + str(n) + " times, need " + str(count))
    return text.replace(old, new)


edits = []

# ---------------------------------------------------------- core_integration.rs
p = os.path.join(SRC, "core_integration.rs")
t = read(p)
anchor = "/// The status a lifecycle folder carries in frontmatter."
addition = '''/// The 0-Core registry directory, when 0-Core is present.
pub fn registry_root() -> Option<PathBuf> {
    let dir = faelight_core::paths::registry_dir();
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

'''
t = swap(t, p, anchor, addition + anchor, 1, "insert registry_root")
edits.append((p, t))

# --------------------------------------------------------------- commands/mod.rs
p = os.path.join(SRC, "commands/mod.rs")
t = read(p)

t = swap(
    t,
    p,
    """                    "@intents" => faelight_core::paths::intents_dir()
                        .to_string_lossy()
                        .to_string(),""",
    """                    // INT-230: same treatment as @rust -- an absent 0-Core
                    // expanded this to the empty string, silently.
                    "@intents" => crate::core_integration::intents_root()
                        .map(|d| d.to_string_lossy().to_string())
                        .unwrap_or_else(|| "@intents".to_string()),""",
    1,
    "3784 at-intents",
)

t = swap(
    t,
    p,
    """            for dir in &["future", "complete", "in-progress"] {
                let path = faelight_core::paths::intents_dir()
                    .join(dir)
                    .to_string_lossy()""",
    """            // INT-230: no root, no items.
            let intents_root = match crate::core_integration::intents_root() {
                Some(r) => r,
                None => return CommandResult::Output(String::new()),
            };
            for dir in &["future", "complete", "in-progress"] {
                let path = intents_root
                    .join(dir)
                    .to_string_lossy()""",
    1,
    "8144 fzf intents",
)

t = swap(
    t,
    p,
    """    for (dir, dir_status) in &dirs {
        let path = faelight_core::paths::intents_dir().join(dir);""",
    """    // INT-230: a SEVENTH intent reader, kept deliberately. It maps each FOLDER
    // to a status label and needs complete/ in full, while ledger() reads the
    // frontmatter status field and carries only complete ids. Only the root
    // moves; the folder-to-status logic stays where its caller needs it.
    let intents_root = crate::core_integration::intents_root();
    for (dir, dir_status) in &dirs {
        let path = match &intents_root {
            Some(r) => r.join(dir),
            None => continue,
        };""",
    1,
    "11691 folder-status reader",
)

t = swap(
    t,
    p,
    """    let count_md = |sub: &str| -> usize {
        std::fs::read_dir(faelight_core::paths::intents_dir().join(sub))""",
    """    // INT-230: the ONE place a zero is allowed on absence -- this renders in a
    // version panel where "0 intents" prints beside "unknown version", so the
    // zero reads as absence rather than as a count that was taken.
    let count_md = |sub: &str| -> usize {
        let root = match crate::core_integration::intents_root() {
            Some(r) => r,
            None => return 0,
        };
        std::fs::read_dir(root.join(sub))""",
    1,
    "11768 count_md",
)

t = swap(
    t,
    p,
    """    let complete_dir = faelight_core::paths::intents_dir()
        .join("complete")
        .to_string_lossy()
        .to_string();""",
    """    let complete_dir = match crate::core_integration::intents_root() {
        Some(r) => r.join("complete").to_string_lossy().to_string(),
        None => return CommandResult::Output(String::new()),
    };""",
    1,
    "16517 complete dir",
)
edits.append((p, t))

# ------------------------------------------------------------------- nl.rs
p = os.path.join(SRC, "nl.rs")
t = read(p)
t = swap(
    t,
    p,
    """        faelight_core::paths::registry_dir()
            .join("shell-patterns.toml")
            .to_string_lossy()
            .to_string(),""",
    """        crate::core_integration::registry_root()
            .map(|r| r.join("shell-patterns.toml").to_string_lossy().to_string())
            .unwrap_or_default(),""",
    1,
    "nl.rs registry",
)
edits.append((p, t))

for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("⚠️ nl.rs uses unwrap_or_default() on a STRING in a candidate-paths list.")
print("   An empty entry there is a path that simply fails to open, not a")
print("   wildcard -- unlike the PathBuf case that broke the rm guard. Confirm")
print("   the list tolerates an empty entry before accepting this.")
print("")
print("Next: cargo build -p novashell --message-format=short")
