#!/usr/bin/env python3
"""
INT-230 G2 -- migrate the three broken intent readers onto core_integration.

THE DEFECT, measured 2026-09-04 in faelight/intents/:
    grep -rl "status: in-progress" future/      -> 0
    grep -rl "status: in-progress" in-progress/ -> 4
    grep -rl "in-progress" future/              -> 5

cistart MOVES a started intent from future/ into in-progress/. All three sites
below scan future/ ONLY, so:
  prompt.rs    next-intent hint  -> structurally always None
  session.rs   active intent list -> structurally always empty
  health_tui.rs active count      -> 5, counting prose mentions, not intents

core reads the frontmatter status field across the lifecycle folders and gets 4.
The shell and the engine have been disagreeing.

digest.rs is NOT touched: it wants `status: planned`, which does live in future/.
It is correct today, and its migration is cleanup rather than a fix.

SAFETY: every anchor must match exactly once. Any miss aborts before a single
byte is written, so a failed run leaves the tree untouched.
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


def span(text, path, start_marker, end_marker):
    """Locate start_marker .. end_marker inclusive. Must be unambiguous."""
    n = text.count(start_marker)
    if n != 1:
        die(path + ": start marker matched " + str(n) + " times, need 1")
    i = text.index(start_marker)
    j = text.find(end_marker, i)
    if j == -1:
        die(path + ": end marker not found after start")
    return i, j + len(end_marker)


edits = []

# ---------------------------------------------------------------- health_tui.rs
# The loose match. `content.contains("in-progress")` tests the WHOLE FILE BODY,
# so an intent whose prose mentions the word counts as active. Measured: 5.
p = os.path.join(SRC, "health_tui.rs")
t = read(p)
i, j = span(
    t,
    p,
    'let active_intents: i64 = std::fs::read_dir(faelight_core::paths::intents_dir().join("future"))',
    ".unwrap_or(0);",
)
new = (
    "// INT-230: was a loose content search over future/ only, which counted\n"
    "    // prose mentions and could not see a started intent at all (cistart\n"
    "    // moves the file to in-progress/). Now asks the adapter, which reads\n"
    "    // the frontmatter status field across the lifecycle folders -- the same\n"
    "    // predicate core uses.\n"
    "    let active_intents: i64 = crate::core_integration::ledger()\n"
    "        .map(|l| l.active_count() as i64)\n"
    "        .unwrap_or(0);"
)
edits.append((p, t[:i] + new + t[j:]))

# ------------------------------------------------------------------ session.rs
p = os.path.join(SRC, "session.rs")
t = read(p)
i, j = span(
    t,
    p,
    "fn active_intents(_core_root: &str) -> Vec<String> {",
    "    intents\n}",
)
new = (
    "// INT-230: scanned future/ for `status: in-progress`, but cistart moves a\n"
    "// started intent into in-progress/ -- so this returned an empty list for as\n"
    "// long as that move has existed. The adapter reads the lifecycle folders.\n"
    "fn active_intents(_core_root: &str) -> Vec<String> {\n"
    "    let mut intents: Vec<String> = match crate::core_integration::ledger() {\n"
    "        Some(l) => l.active().iter().map(|i| format!(\"INT-{}\", i.id)).collect(),\n"
    "        None => vec![],\n"
    "    };\n"
    "    intents.sort();\n"
    "    intents\n}"
)
edits.append((p, t[:i] + new + t[j:]))

# ------------------------------------------------------------------- prompt.rs
p = os.path.join(SRC, "prompt.rs")
t = read(p)
i, j = span(
    t,
    p,
    "let next_intent = std::fs::read_dir(faelight_core::paths::intents_dir().join(\"future\"))",
    "in_progress.first().cloned()\n            });",
)
new = (
    "// INT-230: read future/ for `status: in-progress`. cistart moves a started\n"
    "        // intent into in-progress/, so the hint could never fire. The adapter\n"
    "        // reads the lifecycle folders and returns None when 0-Core is absent,\n"
    "        // which renders as no hint rather than a wrong one.\n"
    "        let next_intent = crate::core_integration::ledger().and_then(|l| {\n"
    "            let mut ids: Vec<String> =\n"
    "                l.active().iter().map(|i| format!(\"INT-{}\", i.id)).collect();\n"
    "            ids.sort();\n"
    "            ids.first().cloned()\n"
    "        });"
)
edits.append((p, t[:i] + new + t[j:]))

# Nothing is written until every anchor above has matched exactly once.
for path, text in edits:
    write(path, text)
    print("patched " + path)

print("")
print("3 files patched. Next: cargo build -p novashell")
