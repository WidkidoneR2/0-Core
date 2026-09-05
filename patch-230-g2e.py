#!/usr/bin/env python3
"""
INT-230 G2, fourth pass part b (CORRECTED) -- release_name and its caller.

⚠️ WHY v1 ABORTED: a BLANK LINE sits between the changelog read and
`let release_name`, and Claude's pattern reconstructed the region without it.
The em-dash was correct (verified U+2014 as M-bM-^@M-^T with cat -A); the
whitespace was not. THIRD anchor miss this session, all three from rebuilding
surrounding text instead of matching what the file holds.

THE FIX: a span cut between two SHORT, UNAMBIGUOUS markers, so nothing between
them has to be reconstructed at all.

⚠️ CLAUDE'S ACCESSOR ALSO RETURNED THE WRONG VALUE. It returned the whole `## [`
heading. The caller splits that heading on an em-dash and takes the SECOND half,
which is the release NAME.

⚠️ THE EM-DASH (U+2014) IS LOAD-BEARING. It now appears in ONE place -- inside
the accessor -- and is built here from chr(8212) so this script cannot mangle
the character in transit.
"""

import io
import os
import sys

SRC = "faelight/rust-tools/novashell/src"
EMDASH = chr(8212)


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


def cut_between(text, path, start_marker, end_marker, replacement, label):
    """Span cut. end_marker is searched strictly AFTER start_marker ends, so
    nothing between them needs to be known or reconstructed."""
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

old_body = '''pub fn release_name() -> Option<String> {
    let changelog = std::fs::read_to_string(faelight_core::paths::changelog_file()).ok()?;
    changelog
        .lines()
        .find(|l| l.starts_with("## ["))
        .map(|l| l.to_string())
}'''

new_body = '''pub fn release_name() -> Option<String> {
    let changelog = std::fs::read_to_string(faelight_core::paths::changelog_file()).ok()?;
    changelog
        .lines()
        .find(|l| l.starts_with("## ["))
        .and_then(|l| l.split(RELEASE_SEPARATOR).nth(1))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// The em-dash separating a changelog version from its release name.
///
/// U+2014, and it is LOAD-BEARING: the split depends on this exact character.
/// It is the one that has broken patch anchors in this repo before, so it lives
/// HERE, once, rather than being re-typed at a call site.
const RELEASE_SEPARATOR: char = \'''' + EMDASH + '''\';'''

t = swap(t, p, old_body, new_body, 1, "correct release_name")

t = swap(
    t,
    p,
    """/// The caller did `unwrap_or_default()` then searched for a `## [` heading, so
/// an absent changelog produced no release name and no signal that anything was
/// missing.""",
    """/// Returns the text AFTER the em-dash in the `## [` heading -- the release
/// name, not the whole heading. The caller previously did `unwrap_or_default()`
/// on the file, so an absent changelog produced no release name and no signal
/// that anything was missing. The display fallback stays with the caller, which
/// is whose choice it is.""",
    1,
    "fix doc comment",
)
edits.append((p, t))

# --------------------------------------------------------------- commands/mod.rs
# Span cut: everything from the changelog read through the fallback line,
# whatever whitespace sits between them.
p = os.path.join(SRC, "commands/mod.rs")
t = read(p)
t = cut_between(
    t,
    p,
    "    let changelog =",
    '.unwrap_or_else(|| "The Forest Remembers".to_string());',
    "    // INT-230: the changelog read and its em-dash split live in the adapter.\n"
    "    let release_name = crate::core_integration::release_name()\n"
    '        .unwrap_or_else(|| "The Forest Remembers".to_string());',
    "wire release_name caller",
)
edits.append((p, t))

for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("Next: cargo build -p novashell --message-format=short")
