#!/usr/bin/env python3
"""
INT-230 G2, fourth pass -- the observability bucket onto the adapter.

EIGHT CALLS, THREE QUESTIONS:

  4 -> forest_version()   The SAME file (faelight/meta/VERSION) read four times,
                          and THE FOUR ALREADY DISAGREE ABOUT ABSENCE:
                            6777  -> unwrap_or_default()      -> ""
                            11437 -> unwrap_or_else("unknown")
                            11718 -> unwrap_or_else("unknown")
                            12198 -> unwrap_or_else("unknown")
                          One question, one file, two different answers. 6777
                          would print an EMPTY version as though it were a
                          version. The accessor returns Option and each caller
                          chooses its own display, which is where that choice
                          belongs.

  1 -> release_name()     changelog_file() at 12201 does unwrap_or_default()
                          then searches for "## [", so an absent changelog
                          silently yields no release name and no signal.

  3 -> health()           ⚠️ THESE THREE ARE ALREADY CORRECT and are wrapped
                          only for the 0-Core presence check. commands/mod.rs
                          12461, main.rs 3106 and prompt.rs 463 all match
                          Some/None honestly, with comments recording that the
                          doubled-95 and doubled-100 fallbacks were removed.
                          Claude was working from an August note that said this
                          was broken; the code had already moved past it. Wrap,
                          do not repair.

📍 NOT IN THE CENSUS, FOUND WHILE READING: commands/mod.rs:6776 is
`db.health_score().unwrap_or(0)` -- a FOURTH health reader, from the database
rather than the cache file, fabricating a zero. It does not touch paths:: so the
census cannot see it. The census measures PATH COUPLING, not the defect class,
and that is a real limit of the artifact worth recording.

⚠️ THE VERSION IS THE FOREST'S, NOT THE SHELL'S. `nsh --version` answers 3.9.0
from CARGO_PKG_VERSION with no file involved. These four are correctly classified
as 0-Core observability.
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
addition = '''/// The forest version string, when 0-Core is present.
///
/// ⚠️ THIS IS THE FOREST'S VERSION, NOT THE SHELL'S. `nsh --version` answers
/// from CARGO_PKG_VERSION with no file involved, and must keep doing so -- a
/// shell that read its own version off disk would report nothing when packaged.
///
/// Four callers read faelight/meta/VERSION independently and had already
/// drifted on what absence means: three fell back to "unknown", one to the
/// EMPTY STRING, which printed as though it were a version. Returning Option
/// moves that choice to the display, where it belongs.
pub fn forest_version() -> Option<String> {
    let v = std::fs::read_to_string(faelight_core::paths::version_file()).ok()?;
    let v = v.trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

/// The current release name from the changelog, when 0-Core is present.
///
/// The caller did `unwrap_or_default()` then searched for a `## [` heading, so
/// an absent changelog produced no release name and no signal that anything was
/// missing.
pub fn release_name() -> Option<String> {
    let changelog = std::fs::read_to_string(faelight_core::paths::changelog_file()).ok()?;
    changelog
        .lines()
        .find(|l| l.starts_with("## ["))
        .map(|l| l.to_string())
}

/// The last health percentage the doctor recorded, when it has ever run.
///
/// ⚠️ THE THREE CALLERS OF THIS WERE ALREADY CORRECT. `paths::read_health()`
/// returns Option and all three match it honestly; the doubled-95 and
/// doubled-100 fallbacks that once made a machine assert peak health from a
/// missing file are gone. This wraps for the 0-Core presence check only, and
/// deliberately does not change what any caller displays.
pub fn health() -> Option<u8> {
    faelight_core::paths::read_health()
}

'''
t = swap(t, p, anchor, addition + anchor, 1, "insert observability accessors")
edits.append((p, t))

# --------------------------------------------------------------- commands/mod.rs
p = os.path.join(SRC, "commands/mod.rs")
t = read(p)

# 6777 -- the outlier that produced an EMPTY version.
t = swap(
    t,
    p,
    """                let version = std::fs::read_to_string(faelight_core::paths::version_file())
                    .unwrap_or_default()
                    .trim()
                    .to_string();""",
    """                // INT-230: was unwrap_or_default(), so an absent VERSION file
                // printed as an empty version. The other three readers of this
                // same file said "unknown"; now they all do.
                let version = crate::core_integration::forest_version()
                    .unwrap_or_else(|| "unknown".to_string());""",
    1,
    "version 6777",
)

# 11437 and 11718 -- identical shape, both already said "unknown".
t = swap(
    t,
    p,
    """    let version = std::fs::read_to_string(faelight_core::paths::version_file())
        .unwrap_or_else(|_| "unknown".into())
        .trim()
        .to_string();""",
    """    let version =
        crate::core_integration::forest_version().unwrap_or_else(|| "unknown".to_string());""",
    2,
    "version 11437 and 11718",
)

# 12198 -- no .trim(), so kept separate.
t = swap(
    t,
    p,
    """    let version = std::fs::read_to_string(faelight_core::paths::version_file())
        .unwrap_or_else(|_| "unknown".into());""",
    """    let version =
        crate::core_integration::forest_version().unwrap_or_else(|| "unknown".to_string());""",
    1,
    "version 12198",
)

# 12461 -- health, already correct, wrapped for the presence check.
t = swap(
    t,
    p,
    "    let health: String = match faelight_core::paths::read_health() {",
    "    let health: String = match crate::core_integration::health() {",
    1,
    "health 12461",
)
edits.append((p, t))

# ------------------------------------------------------------------- main.rs
p = os.path.join(SRC, "main.rs")
t = read(p)
t = swap(
    t,
    p,
    "    let health_opt = faelight_core::paths::read_health();",
    "    let health_opt = crate::core_integration::health();",
    1,
    "health main.rs",
)
edits.append((p, t))

# ------------------------------------------------------------------ prompt.rs
p = os.path.join(SRC, "prompt.rs")
t = read(p)
t = swap(
    t,
    p,
    "match faelight_core::paths::read_health().map(u32::from) {",
    "match crate::core_integration::health().map(u32::from) {",
    1,
    "health prompt.rs",
)
edits.append((p, t))

for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("⚠️ changelog_file at commands/mod.rs:12201 is NOT patched here -- its")
print("   caller parses the changelog inline and needs reading first.")
print("")
print("Next: cargo build -p novashell --message-format=short")
