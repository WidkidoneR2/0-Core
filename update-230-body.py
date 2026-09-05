#!/usr/bin/env python3
"""
INT-230 -- catch the record up to the work.

Ticks G1 and G3 with INT-158 evidence comments, and appends the discovered work
found while doing G2. Nothing here is a code change.

WHY THIS RUNS AS A SCRIPT RATHER THAN AN EDIT BY HAND: the gate text must match
the intent file byte for byte, and three anchors missed today from
reconstructing text instead of matching it. The assertions abort before writing.
"""

import glob
import io
import sys

MATCHES = glob.glob("faelight/intents/in-progress/230-*.md")
if len(MATCHES) != 1:
    print("ABORT: expected exactly one 230 file in in-progress/, found "
          + str(len(MATCHES)), file=sys.stderr)
    sys.exit(1)
PATH = MATCHES[0]


def die(msg):
    print("ABORT: " + msg, file=sys.stderr)
    sys.exit(1)


with io.open(PATH, "r", encoding="utf-8") as fh:
    text = fh.read()


def swap(t, old, new, label):
    n = t.count(old)
    if n != 1:
        die(label + ": matched " + str(n) + " times, need 1")
    return t.replace(old, new)


# ------------------------------------------------------------------ G1 ticked
text = swap(
    text,
    "- [ ] G1 THE 68 SITES ARE CLASSIFIED BY CAPABILITY",
    "- [x] G1 THE 68 SITES ARE CLASSIFIED BY CAPABILITY",
    "tick G1",
)
text = swap(
    text,
    "      it can be re-run and diffed. The classification decides the scope; it is not decoration",
    "      it can be re-run and diffed. The classification decides the scope; it is not decoration\n"
    "<!-- evidence: 9e726f92. census-core-coupling.py + faelight/rust-tools/novashell/CORE-COUPLING.md.\n"
    "     82 classified paths:: calls across 14 functions -- core shell state 31, 0-Core discovery 40,\n"
    "     observability 8, execution 3. The unit is stated three ways because faelight_core lines (80),\n"
    "     classified calls (82) and bare use statements (1) are different numbers and were being confused.\n"
    "     THE PREDICTION IN THIS INTENT HELD, and the reverse of what the raw count suggested: 38% of the\n"
    "     coupling needs no adapter at all, because the XDG move and FAELIGHT_STATE_DB already resolved it.\n"
    "     Re-runnable and diff-gated: exits 2 on an unclassified function, so a new coupling site cannot\n"
    "     land silently. It did exactly that on its first run -- bin_dir was dropped between the histogram\n"
    "     and the classification table and the script refused to pass. It has since corrected Claude four\n"
    "     times: bin_dir, a miscounted six-vs-four migration, and two arithmetic slips. -->",
    "G1 evidence",
)

# ------------------------------------------------------------------ G3 ticked
text = swap(
    text,
    "- [ ] G3 A 0-CORE-ABSENT fsh STARTS, ACCEPTS INPUT, AND RUNS A COMMAND. Demonstrated, not argued",
    "- [x] G3 A 0-CORE-ABSENT fsh STARTS, ACCEPTS INPUT, AND RUNS A COMMAND. Demonstrated, not argued\n"
    "<!-- evidence: demonstrated 2026-09-04 on the deployed binary.\n"
    "     mkdir -p /tmp/g3home; HOME=/tmp/g3home nsh -c \"echo SHELL_ALIVE\" -> SHELL_ALIVE, exit 0.\n"
    "     HOME=/tmp/g3home nsh -c \"pwd\" -> the real launch directory, NOT a phantom -- the startup-cd\n"
    "     fix from 2026-08-21 is holding. /usr/bin/ls -la /tmp/g3home -> EMPTY: nsh -c manufactures no\n"
    "     state at all under a fresh HOME, so the shell cannot build a forest on a machine that has none. -->",
    "G3 evidence",
)

# --------------------------------------------------- discovered work, appended
addition = """

## ⭐ G2 PROGRESS AND WHAT IT FOUND (2026-09-04)

Commits: `fdc174a9` (boundary + four broken readers) - `3c4d7ea6` (rust_tools_dir)
- `e6b290e1` (observability). Census: 0-Core discovery **40 -> 25**.

**THE BOUNDARY IS `novashell/src/core_integration.rs`.** It CALLS
`faelight_core::paths` and never copies it, so `paths` stays the single authority
over WHERE and the adapter owns WHETHER. Runtime only, no `cfg` -- G6 holds.
Accessors so far: `present`, `ledger`, `tools_root`, `tool_manifest`,
`forest_version`, `release_name`, `health`.

### ⚠️⚠️ FOUR SURFACES WERE READING A DIRECTORY THE WORKFLOW EMPTIES
`cistart` MOVES a started intent from `future/` into `in-progress/`, and
`prompt.rs`, `session.rs`, `digest.rs:106` and `health_tui.rs` all scanned
`future/` only. Measured: `future/` held **0** intents with `status: in-progress`,
`in-progress/` held **4**. So the next-intent hint and the banner list were
STRUCTURALLY always empty, and `health_tui` reported **5** by matching the bare
word in prose rather than the frontmatter key. `core` reads the status field and
had the right answer the whole time. **The shell and the engine disagreed and
nothing noticed.** Fixed; the banner now names the real work.

### ⚠️ TWO COPIES OF ONE PATH THAT HAD ALREADY DRIFTED
`dev_cmd` built the same eleven-line manifest expression in its `test` arm and
again in its `watch` arm, byte-identical, forty lines apart -- and `test` checked
the file existed while `watch` did not, so `dev watch nosuchtool` announced cargo
watch on a path that was not there. The existence check now lives INSIDE
`tool_manifest`, where an arm cannot skip it.

Likewise `faelight/meta/VERSION` was read at four sites with **two different
answers for absence**: three said `unknown`, one produced the EMPTY STRING and
printed it as though it were a version.

### ⚠️⚠️ THREE THINGS THE ADAPTER CANNOT FIX, recorded so they are not lost
1. **`cheatsheet_tui.rs:265` reads `rust_tools_dir()/novashell/src/commands/mod.rs`.**
   nsh parses ITS OWN SOURCE at runtime to build the cheatsheet. A packaged
   install has no source tree, so it yields an empty cheatsheet with no error.
   This is a feature that cannot survive the packaging this intent exists to
   enable, and it needs its own answer: generate at build time, ship the parsed
   data, or drop it. **Deliberately not migrated.**
2. **`commands/mod.rs:6776` is `db.health_score().unwrap_or(0)`** -- a FOURTH
   health reader, fabricating a zero, reading the DATABASE rather than a path.
   ⚠️ **The census cannot see it.** G1 measures path coupling, not the defect
   class, and a clean census must never be read as a clean shell.
3. **Two version authorities, a full major apart.** `nsh version` prints
   **1.0.0** from `faelight/meta/VERSION` with a release dated 2026-07-06, while
   `core version` and the banner both say Forest **13.0.0**. Not caused by this
   work and not fixed by it.

### ✅ ONE THING THE AUGUST RECORD HAD WRONG
The note that `prompt.rs` falls back to `"100"` then `unwrap_or(100)` and asserts
PEAK health from a missing file is **STALE**. All three `read_health` callers now
match `Some`/`None` honestly and carry comments recording that the doubled
fallbacks were removed. They were wrapped for the presence check, not repaired.

### ⏭ G2 REMAINING
**25 discovery calls** (21 `intents_dir`, plus `core_root_string`,
`registry_dir`, `tools_registry`, and the cheatsheet), **3 `daemon_socket`**.

### ⏭ G4 HAS A LIST NOW, AND PART OF IT IS SELF-INFLICTED
G4 is NOT satisfied and this pass moved against it in one respect: **five
`.unwrap_or_default()` calls** were introduced to keep the migration mechanical,
each turning an absent forest into an empty `PathBuf`. Named here rather than
discovered later. Add `health_score().unwrap_or(0)` and `health_tui`'s
`unwrap_or(0)`, which survived this pass unchanged.
"""

with io.open(PATH, "w", encoding="utf-8") as fh:
    fh.write(text + addition)

print("updated " + PATH)
print("")
print("Verify: grep -n \"G1 THE 68\\|G3 A 0-CORE\" " + PATH)
