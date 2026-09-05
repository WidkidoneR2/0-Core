#!/usr/bin/env python3
"""
INT-230 G2, second pass -- digest.rs onto the adapter, and zero warnings.

TWO SITES IN digest.rs, and only one of them was correct:

  blocked_ready() (line 29)  -- CORRECT today. It wants `status: planned`, which
                                does live in future/. Migrated as CLEANUP: it
                                hand-rolls a complete_ids set and a depends_on
                                frontmatter parser that the adapter already owns.

  render() (line 106)        -- BROKEN, same defect as the other three. Scans
                                future/ for `status: in-progress` to build the
                                banner list. cistart moves started intents into
                                in-progress/, so that list is structurally empty.

⚠️ Claude called this file "fine" after reading only the first site, then found
the second. That is the second time in this intent a whole file was generalised
from one function. The census had all three line numbers the whole time.

WARNINGS: cleared by USE (STATUS_PLANNED, is_planned, complete_ids,
blocked_ready, depends_on all become live here) and by DELETION (total, all,
title -- written speculatively, consumed by nothing). No #[allow(dead_code)]:
an unused item is telling the truth, and suppressing it is how a canonical
function ends up dead with a comment claiming everyone uses it.
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


def cut(text, path, start_marker, end_marker, replacement):
    """Replace start_marker .. end_marker inclusive. Must be unambiguous."""
    n = text.count(start_marker)
    if n != 1:
        die(path + ": start marker matched " + str(n) + " times, need 1")
    i = text.index(start_marker)
    j = text.find(end_marker, i)
    if j == -1:
        die(path + ": end marker not found after start")
    return text[:i] + replacement + text[j + len(end_marker):]


def drop(text, path, snippet):
    """Delete an exact snippet. Must be unambiguous."""
    n = text.count(snippet)
    if n != 1:
        die(path + ": snippet matched " + str(n) + " times, need 1")
    return text.replace(snippet, "", 1)


edits = []

# -------------------------------------------------------------------- digest.rs
p = os.path.join(SRC, "digest.rs")
t = read(p)

# Site 1 -- blocked_ready. Cleanup: the adapter owns the frontmatter parse and
# the complete/ id set. Absence yields (0, 0), which renders as no ready prompt.
t = cut(
    t,
    p,
    "pub fn blocked_ready() -> (usize, usize) {",
    "    (blocked_count, ready_count)\n}",
    "pub fn blocked_ready() -> (usize, usize) {\n"
    "    // INT-230: was a hand-rolled complete/ id set plus its own depends_on\n"
    "    // frontmatter parser -- a second implementation of what the adapter\n"
    "    // owns. Absence yields (0, 0), which renders as no ready prompt.\n"
    "    crate::core_integration::ledger()\n"
    "        .map(|l| l.blocked_ready())\n"
    "        .unwrap_or((0, 0))\n"
    "}",
)

# Site 2 -- the banner list. THE BROKEN ONE.
t = cut(
    t,
    p,
    'let intents_path = faelight_core::paths::intents_dir().join("future");',
    "\n    if !active_intents.is_empty() {",
    "// INT-230: scanned future/ for `status: in-progress`. cistart moves a\n"
    "    // started intent into in-progress/, so this list has been empty for as\n"
    "    // long as that move has existed -- the banner could not name the work\n"
    "    // actually in progress. The adapter reads the lifecycle folders using\n"
    "    // the same frontmatter predicate core does.\n"
    "    let mut active_intents: Vec<String> = match crate::core_integration::ledger() {\n"
    "        Some(l) => l.active().iter().map(|i| format!(\"INT-{}\", i.id)).collect(),\n"
    "        None => vec![],\n"
    "    };\n"
    "    active_intents.sort();\n"
    "\n    if !active_intents.is_empty() {",
)
edits.append((p, t))

# ---------------------------------------------------------- core_integration.rs
# Speculative API with no consumer, deleted rather than allowed.
p = os.path.join(SRC, "core_integration.rs")
t = read(p)

t = drop(
    t,
    p,
    "    pub fn total(&self) -> usize {\n"
    "        self.intents.len()\n"
    "    }\n"
    "\n"
    "    pub fn all(&self) -> &[Intent] {\n"
    "        &self.intents\n"
    "    }\n"
    "\n",
)

t = drop(t, p, "    pub title: String,\n")
t = drop(t, p, "    let mut title = String::new();\n")
t = drop(
    t,
    p,
    '        } else if let Some(v) = trimmed.strip_prefix("title:") {\n'
    "            title = v.trim().to_string();\n",
)
t = drop(t, p, "        title,\n")
edits.append((p, t))

for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("Next: cargo build -p novashell --message-format=short")
