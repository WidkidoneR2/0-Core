#!/usr/bin/env python3
"""
INT-230 G2, third pass (CORRECTED) -- rust_tools_dir onto the adapter.

⚠️ WHY v1 ABORTED, and it was a script bug not a file surprise: the `dev test`
cut used `            }` as its end marker and searched from the START of the
block -- but the block ITSELF ends with that string, so find() matched inside
the start marker and the cut terminated early. The debris made a later pattern
match twice, and the count assertion caught it before a byte was written.

THE CORRECTION IS ALSO SIMPLER. The eleven-line manifest expression is
BYTE-IDENTICAL in the `test` and `watch` arms, and `sub` is in scope in both --
so one replacement with count=2 serves both and the message writes itself as
"dev {sub}". No overlapping markers anywhere in this version.

TWELVE CALLS, FOUR QUESTIONS, ONE OF THEM DEAD:

  6 -> tool_manifest(tool)   dev_cmd built the same expression in `test` and
                             again in `watch`, forty lines apart -- and the
                             copies HAD ALREADY DRIFTED: `test` checked the
                             manifest existed, `watch` did not, so
                             `dev watch nosuchtool` announced cargo watch on a
                             path that was not there. The check moves INSIDE the
                             accessor so an arm cannot skip it.

  2 -> DELETED               `fn tools` is #[allow(dead_code)] with no reachable
                             caller. It carried an unwrap_or(0) reporting a
                             confident zero tool count on an absent forest, and
                             read core_root/scripts -- the NixOS-era deployment
                             shape the doctor already corrected. Dead code
                             holding two stale assumptions behind an attribute.

  1 -> tools_table           the real enumeration; asks the adapter for the root.

  3 -> tools_root()          the @rust shortcut, the search root, exec.rs:302.

  1 -> NOT MIGRATED          cheatsheet_tui.rs:265 reads
                             rust_tools_dir()/novashell/src/commands/mod.rs --
                             nsh parses ITS OWN SOURCE at runtime. A packaged
                             install has no source tree, so it yields an empty
                             cheatsheet with no error. The adapter cannot fix a
                             feature that cannot survive packaging. Recorded in
                             INT-230, deliberately left alone here.

⚠️ FIVE .unwrap_or_default() CALLS ARE INTRODUCED. They turn an absent forest
into an empty PathBuf, which is the fabricated-answer shape G4 forbids. Traded
deliberately to keep this pass mechanical; they are named G4 work, not done.
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
    """Exact replacement with a count assertion. No span arithmetic."""
    n = text.count(old)
    if n != count:
        die(path + " [" + label + "]: matched " + str(n) + " times, need " + str(count))
    return text.replace(old, new)


def cut_between(text, path, start_marker, end_marker, replacement, label):
    """Span cut where end_marker is searched AFTER the start marker ends."""
    n = text.count(start_marker)
    if n != 1:
        die(path + " [" + label + "]: start matched " + str(n) + " times, need 1")
    i = text.index(start_marker)
    after = i + len(start_marker)
    j = text.find(end_marker, after)
    if j == -1:
        die(path + " [" + label + "]: end marker not found after start")
    return text[:i] + replacement + text[j + len(end_marker):]


edits = []

# ---------------------------------------------------------- core_integration.rs
p = os.path.join(SRC, "core_integration.rs")
t = read(p)

anchor = "/// The status a lifecycle folder carries in frontmatter."
addition = '''/// The 0-Core rust-tools source tree, when 0-Core is present.
///
/// `None` on a machine without a forest -- which is every packaged install.
pub fn tools_root() -> Option<PathBuf> {
    let dir = faelight_core::paths::rust_tools_dir();
    if dir.is_dir() {
        Some(dir)
    } else {
        None
    }
}

/// The Cargo.toml for one tool, or for novashell when `tool` is empty.
///
/// The existence check lives HERE on purpose. `dev_cmd` built this path in its
/// `test` arm and again in its `watch` arm -- byte-identical, forty lines apart
/// -- and the two had drifted: `test` checked the file existed and returned an
/// error, `watch` did not and announced cargo watch on a manifest that was not
/// there. Two owners of one path, agreeing on where it was and disagreeing on
/// whether it was real. One owner now, and an arm cannot skip a check that is
/// not the arm's to skip.
pub fn tool_manifest(tool: &str) -> Option<PathBuf> {
    let root = tools_root()?;
    let manifest = if tool.is_empty() {
        root.join("novashell/Cargo.toml")
    } else {
        root.join(tool).join("Cargo.toml")
    };
    if manifest.is_file() {
        Some(manifest)
    } else {
        None
    }
}

'''
t = swap(t, p, anchor, addition + anchor, 1, "insert accessors")
edits.append((p, t))

# --------------------------------------------------------------- commands/mod.rs
p = os.path.join(SRC, "commands/mod.rs")
t = read(p)

# Both dev_cmd arms at once -- identical text, and `sub` is in scope in both.
t = swap(
    t,
    p,
    """            let manifest = if tool.is_empty() {
                faelight_core::paths::rust_tools_dir()
                    .join("novashell/Cargo.toml")
                    .to_string_lossy()
                    .to_string()
            } else {
                faelight_core::paths::rust_tools_dir()
                    .join(tool)
                    .join("Cargo.toml")
                    .to_string_lossy()
                    .to_string()
            };""",
    """            // INT-230: one accessor, and the existence check lives inside it.
            // The `watch` arm never checked and announced cargo watch on a
            // manifest that was not there; it cannot skip the check now.
            let manifest = match crate::core_integration::tool_manifest(tool) {
                Some(m) => m.to_string_lossy().to_string(),
                None => {
                    return CommandResult::Error(
                        format!("  dev {}: no Cargo.toml found for '{}'", sub, tool).into(),
                        1,
                    );
                }
            };""",
    2,
    "dev test and watch arms",
)

# The test arm's explicit check is now redundant -- the accessor guarantees it.
t = swap(
    t,
    p,
    """            if !std::path::Path::new(&manifest).exists() {
                return CommandResult::Error(
                    format!("  dev test: no Cargo.toml found for '{}'", tool).into(),
                    1,
                );
            }
""",
    "",
    1,
    "redundant test-arm check",
)

# 13943 -- 12-space `let`, 16-space continuation.
t = swap(
    t,
    p,
    """            let manifest = faelight_core::paths::rust_tools_dir()
                .join(tool)
                .join("Cargo.toml")""",
    """            let manifest = crate::core_integration::tools_root()
                .unwrap_or_default()
                .join(tool)
                .join("Cargo.toml")""",
    1,
    "manifest 13943",
)

# 13961 -- 16-space `let`, 20-space continuation.
t = swap(
    t,
    p,
    """                let manifest = faelight_core::paths::rust_tools_dir()
                    .join(tool)
                    .join("Cargo.toml")""",
    """                let manifest = crate::core_integration::tools_root()
                    .unwrap_or_default()
                    .join(tool)
                    .join("Cargo.toml")""",
    1,
    "manifest 13961",
)

# fn tools -- dead, with an unwrap_or(0) and a NixOS-era scripts/ read.
t = cut_between(
    t,
    p,
    "#[allow(dead_code)]\nfn tools(_db: &ForestDb, core_root: &str) -> CommandResult {",
    "\n}\n",
    "",
    "delete dead fn tools",
)

# tools_table -- the real enumeration.
t = swap(
    t,
    p,
    "    let tools_dir = faelight_core::paths::rust_tools_dir();\n    let mut rows = Vec::new();",
    "    // INT-230: asks the adapter for the root, so an absent forest yields an\n"
    "    // empty table rather than a table of a directory that is not there.\n"
    "    let tools_dir = match crate::core_integration::tools_root() {\n"
    "        Some(d) => d,\n"
    "        None => return CommandResult::Output(String::new()),\n"
    "    };\n    let mut rows = Vec::new();",
    1,
    "tools_table",
)

t = swap(
    t,
    p,
    """"@rust" => faelight_core::paths::rust_tools_dir()
                        .to_string_lossy()
                        .to_string(),""",
    """"@rust" => crate::core_integration::tools_root()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),""",
    1,
    "at-rust shortcut",
)

t = swap(
    t,
    p,
    "search_root = faelight_core::paths::rust_tools_dir();",
    "search_root = crate::core_integration::tools_root().unwrap_or_default();",
    1,
    "search root",
)
edits.append((p, t))

# ------------------------------------------------------------------- exec.rs
p = os.path.join(SRC, "exec.rs")
t = read(p)
t = swap(
    t,
    p,
    """let core_src = faelight_core::paths::rust_tools_dir()
                .to_string_lossy()
                .to_string();""",
    """let core_src = crate::core_integration::tools_root()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();""",
    1,
    "exec core_src",
)
edits.append((p, t))

for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("Next: cargo build -p novashell --message-format=short")
