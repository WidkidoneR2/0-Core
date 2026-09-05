#!/usr/bin/env python3
"""
INT-230 G2, fifth pass -- main.rs intent readers onto the adapter.

FOUR SITES MIGRATE. ONE DELIBERATELY DOES NOT.

  3416 -> THE FIFTH BROKEN READER. `intents_dir().join("future")` filtered on
          `status: in-progress`, and future/ holds ZERO of those because cistart
          moves a started intent into in-progress/. Its own comment says "Show
          today's focus from actual in-progress intents only" -- so the focus
          line has never had anything to show.

  3272 -> COUNTED FILES, NOT STATUSES, across ["in-progress", "active"]. It is
          RIGHT TODAY FOR THE WRONG REASON: it agrees with the status count only
          because cistart keeps folder and status in sync. Edit a status without
          moving the file and this number and the ledger disagree. Also "active"
          is a folder that does not exist.

  3041 -> ALREADY CORRECT -- in-progress/ AND `status: in-progress`, the right
          folder and the right predicate. Migrated for one-owner reasons only,
          not as a fix. Sixth site, and the first one that was not broken.

  3127 -> `intents_dir().exists()`. ⚠️ Claude claimed earlier in this intent that
          there was no existence question to hook into. WRONG: paths.rs has none,
          but main.rs already had one, and `present()` duplicates it. Deleting
          the local copy is the point of a boundary.

  3130 -> ⚠️⚠️ NOT MIGRATED, AND THAT IS THE FINDING. Its comment records a
          decision the adapter would have silently undone: counting `planned`
          across all nine categories reported 43 where `intl` reported 40,
          because four decisions and philosophy documents carry `status: planned`
          -- a status those categories have no use for. The `*cat == "future"`
          scope is DELIBERATE and the disagreement is kept VISIBLE as INT-211's
          finding. `ledger()` reads three lifecycle folders and would produce a
          different number for reasons not written down there. It stays in the
          census as an unmigrated site with a stated reason.

📍 MEASURED BEFORE SWAPPING, so the count claim is not an assumption:
`faelight/intents/deferred/` DOES NOT EXIST (zero entries), so
`ledger().active_count()` can only draw from in-progress/ and the swap is
behaviour-preserving. 📍 `deferred` and `active` are both named in code and
absent on disk -- harmless today, a silent behaviour change the day someone
creates one.

⚠️ NOT TOUCHED, AND MISCLASSIFIED IN THE CENSUS: exec.rs:307 and exec.rs:1536
build the intents path as a PROTECTED PATH for the catastrophic-rm guard, and
1536 is its test. That is SAFETY, not discovery. Wrapping it in an Option that
returns None when 0-Core is absent would silently disarm a protection. It is in
the wrong census bucket and it should stay unwrapped either way.
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


def cut_between(text, path, start_marker, end_marker, replacement, label):
    """end_marker is searched strictly AFTER start_marker ends, so nothing
    between them has to be reconstructed. This is the shape that stopped the
    three anchor misses earlier in this intent."""
    n = text.count(start_marker)
    if n != 1:
        die(path + " [" + label + "]: start matched " + str(n) + " times, need 1")
    i = text.index(start_marker)
    after = i + len(start_marker)
    j = text.find(end_marker, after)
    if j == -1:
        die(path + " [" + label + "]: end marker not found after start")
    return text[:i] + replacement + text[j + len(end_marker):]


p = os.path.join(SRC, "main.rs")
t = read(p)

# ---- 3127: the duplicate existence check -------------------------------------
t = swap(
    t,
    p,
    "    let ledger_exists = faelight_core::paths::intents_dir().exists();",
    "    // INT-230: main.rs had its own existence check. The adapter owns that\n"
    "    // question now -- one place asks whether 0-Core is here.\n"
    "    let ledger_exists = crate::core_integration::present();",
    1,
    "3127 present()",
)

# ---- 3272: file count across in-progress and a folder that does not exist ----
t = cut_between(
    t,
    p,
    "    let active_count = {\n        let dir = faelight_core::paths::intents_dir();",
    "        n\n    };",
    "    // INT-230: counted .md FILES across in-progress/ and \"active\" -- a folder\n"
    "    // that does not exist. It agreed with the status count only because cistart\n"
    "    // keeps folder and status in sync; a status edited without moving the file\n"
    "    // would have made the banner and the ledger disagree. Now one definition.\n"
    "    // Measured before swapping: deferred/ does not exist, so this is\n"
    "    // behaviour-preserving.\n"
    "    let active_count = crate::core_integration::ledger()\n"
    "        .map(|l| l.active_count())\n"
    "        .unwrap_or(0);",
    "3272 active_count",
)

# ---- 3041: correct already, migrated for one owner ---------------------------
t = cut_between(
    t,
    p,
    "        let active_intent: String =\n"
    "            std::fs::read_dir(faelight_core::paths::intents_dir().join(\"in-progress\"))",
    "                .join(\", \")\n                })",
    "        // INT-230: this one was ALREADY CORRECT -- right folder, right\n"
    "        // predicate. Migrated so the shell has one definition of an active\n"
    "        // intent, not because it was wrong.\n"
    "        let active_intent: String = crate::core_integration::ledger()\n"
    "            .map(|l| {\n"
    "                l.active()\n"
    "                    .iter()\n"
    "                    .map(|i| format!(\"INT-{}\", i.id))\n"
    "                    .collect::<Vec<_>>()\n"
    "                    .join(\", \")\n"
    "            })",
    "3041 active_intent",
)

# ---- 3416: the fifth broken reader -------------------------------------------
t = cut_between(
    t,
    p,
    "    let focus_intent: Option<String> =\n"
    "        std::fs::read_dir(faelight_core::paths::intents_dir().join(\"future\"))",
    "                if in_progress.is_empty() {\n                    None\n                } else {",
    "    // INT-230: read future/ for `status: in-progress`. cistart moves a started\n"
    "    // intent into in-progress/, so this comment -- \"today's focus from actual\n"
    "    // in-progress intents only\" -- described something that could never fire.\n"
    "    let focus_intent: Option<String> = crate::core_integration::ledger().and_then(|l| {\n"
    "        let mut names: Vec<String> = l\n"
    "            .active()\n"
    "            .iter()\n"
    "            .map(|i| format!(\"INT-{}\", i.id))\n"
    "            .collect();\n"
    "        names.sort();\n"
    "        if names.is_empty() {\n"
    "            None\n"
    "        } else {",
    "3416 focus_intent",
)

write(p, t)
print("patched " + p)
print("")
print("⚠️ 3416's replacement changes the DISPLAY STRING: it was a de-slugged")
print("   filename, it is now INT-nnn. Confirm that reads right before shipping.")
print("")
print("Next: cargo build -p novashell --message-format=short")
