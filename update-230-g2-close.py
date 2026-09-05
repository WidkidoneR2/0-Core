#!/usr/bin/env python3
"""
INT-230 -- tick G2 and G6 with evidence.

G6 is satisfied BY CONSTRUCTION, not by work: there is no `cfg` anywhere in
core_integration.rs or at any migrated call site. Absence is a runtime fact
throughout. Recording that is the gate; it asked for a reason to be written
first if a flag were ever introduced, and none was.
"""

import glob
import io
import sys

MATCHES = glob.glob("faelight/intents/in-progress/230-*.md")
if len(MATCHES) != 1:
    print("ABORT: expected one 230 file, found " + str(len(MATCHES)), file=sys.stderr)
    sys.exit(1)
PATH = MATCHES[0]

with io.open(PATH, "r", encoding="utf-8") as fh:
    text = fh.read()


def swap(t, old, new, label):
    n = t.count(old)
    if n != 1:
        print("ABORT: " + label + " matched " + str(n) + " times, need 1", file=sys.stderr)
        sys.exit(1)
    return t.replace(old, new)


text = swap(
    text,
    "- [ ] G2 THE ADAPTER BOUNDARY IS NAMED",
    "- [x] G2 THE ADAPTER BOUNDARY IS NAMED",
    "tick G2",
)
text = swap(
    text,
    "      reads the answer. No second authority over the layout -- `paths` is not copied",
    """      reads the answer. No second authority over the layout -- `paths` is not copied
<!-- evidence: fdc174a9, 3c4d7ea6, e6b290e1, 540ff38a, a054f3a2, c061cac6, 84d1b696, 7cf40096.
     THE BOUNDARY IS novashell/src/core_integration.rs. It CALLS faelight_core::paths and never
     copies it, so paths stays the single authority over WHERE and the adapter owns WHETHER.
     Accessors: present, ledger, focus, intents_root, tools_root, tool_manifest, registry_root,
     forest_version, release_name, health.
     MEASURED BY THE CENSUS: 51 unmigrated 0-Core calls at the start, 13 at the end, and none of
     the 13 is work the boundary should own -- 4 are the adapter's own calls, 4 are observability
     inside it, 2 are exec.rs's catastrophic-rm protected paths (SAFETY: an Option returning None
     would silently disarm a protection), 1 is main.rs:3126 which STAYS BY RULING (its comment
     records the 43-vs-40 category scoping kept visible as INT-211's finding), 1 is db.rs:68
     core_root_string which is how the shell finds its OWN database and is core shell state
     misclassified, and 1 is cheatsheet_tui.rs:265, which cannot be fixed by an adapter at all.
     daemon_socket was reclassified to core shell state on measurement: it derives from
     runtime_dir(), XDG since 2026-08-21, and its three callers were already correct.
     ⚠️ THE CENSUS NOW STATES ITS OWN LIMITS in its generated output, because it corrected this
     work five times and will be trusted for that reason: it matches text not syntax, it sees only
     paths:: calls (THIRTY hand-built 0-core paths in this shell are invisible to it, twelve
     pointing at a scripts directory that does not exist -- INT-240), it classifies by function so
     one function asked two questions cannot be split, and it measures path coupling rather than
     the defect class. -->""",
    "G2 evidence",
)

text = swap(
    text,
    "- [ ] G6 NO COMPILE-TIME FEATURE FLAG unless a concrete reason to compile two products is recorded",
    "- [x] G6 NO COMPILE-TIME FEATURE FLAG unless a concrete reason to compile two products is recorded",
    "tick G6",
)
text = swap(
    text,
    "      here first",
    """      here first
<!-- evidence: satisfied BY CONSTRUCTION. No `cfg` appears in core_integration.rs or at any
     migrated call site; absence is a runtime fact throughout. The intent rejected the flag on
     the grounds that it PRESERVES the coupling while worsening the build matrix, and nothing in
     the migration produced a reason to revisit that. Verify with: grep -rn cfg( on the adapter. -->""",
    "G6 evidence",
)

addition = """

## ✅ G2 CLOSED, AND WHAT THE MIGRATION FOUND (2026-09-05)

The boundary work found more defects than it created abstractions. Every one is
the same shape -- **an absent or wrong value presented as a real one** -- which
is INT-227's invariant, and is why G4 has been closing site by site rather than
as a gate at the end.

### The defects, all fixed and all proven live

1. **FIVE BLIND INTENT READERS.** `cistart` moves a started intent from
   `future/` to `in-progress/`, and `prompt.rs`, `session.rs`, `digest.rs:106`,
   `health_tui.rs` and `main.rs:3416` all scanned `future/` only. Measured:
   `future/` held **0** intents with `status: in-progress`, `in-progress/` held
   **4**. So the banner list and the focus hint were STRUCTURALLY empty, and
   `health_tui` reported **5** by matching the bare word in prose. `core` had
   the right answer the whole time; the shell and the engine disagreed and
   nothing noticed.

2. **A GUARD THAT BLOCKED EVERY `rm -rf`.** The catastrophic-rm protected list
   tests `expanded.contains(protected)`, and an `unwrap_or_default()` made one
   entry the EMPTY STRING -- which every string contains. On a forestless
   machine it refused every recursive delete while naming `''` as the thing it
   protected. **Introduced by Claude during this intent** and fixed with a
   filter, not a wrap: a protected list holds paths that exist.

3. **`find @rust` SEARCHED AN EMPTY PATH.** Same `unwrap_or_default()`, and
   worse than degradation -- it OVERWROTE a good `core_root` default and handed
   `""` to `fd`. Also Claude's. All four search-root sites now refuse with a
   reason, because each one ends in a tool being pointed at a directory.

4. **FOUR VERSION READERS DISAGREEING ON ABSENCE.** Three said `unknown`, one
   produced the empty string and printed it as though it were a version.

5. **TWO COPIES OF ONE PATH THAT HAD DRIFTED.** `dev_cmd` built the same
   manifest expression in `test` and `watch`, forty lines apart; `test` checked
   the file existed, `watch` did not and announced cargo watch on a path that
   was not there.

6. **A FOCUS WRITE THAT COULD NEVER SUCCEED.** `set_focus_intent` writes
   `shell_state.focus_intent`; `get_focus_intent` reads `focus.toml`. A setter
   and a getter sharing a name and not a storage. And the caller's all-digits
   guard rejected every value the old display produced, so two independent
   failures were stacked, neither visible from the other. Filed as **INT-242**.

7. **A BANNER COMPUTATION THAT WAS NEVER USED AND WAS WRONG.** `_focus` read and
   parsed the tools registry on every render and discarded the result -- and its
   comment claimed "lowest audit score tool" while the code took the first name
   line in file order, reading no score at all.

### Recorded, not fixed here

- **`cheatsheet_tui.rs:265`** reads `rust_tools_dir()/novashell/src/commands/mod.rs`.
  nsh parses ITS OWN SOURCE at runtime, so a packaged install gets an empty
  cheatsheet with no error. ⚠️ **This cannot survive the packaging this intent
  exists to enable** and needs its own answer: generate at build time, ship the
  parsed data, or drop it.
- **THIRTY hand-built `0-core/` paths**, twelve pointing at `~/0-core/scripts`
  which does not exist, plus `completion.rs` offering `cd` to four paths that
  were never right after the tree moved. INT-240, measured.
- **Two frontmatter parsers with different tolerances** -- `intent_tui`'s reads
  a proper `---` fence, the adapter's bounds at twenty lines. Both work today.
  INT-211.
- **`db.health_score().unwrap_or(0)`** at `commands/mod.rs:6776`, a fourth
  health reader fabricating a zero from the database. Invisible to the census.
- **Two version authorities a full major apart**: `nsh version` prints 1.0.0
  from `faelight/meta/VERSION`, `core version` says Forest 13.0.0.

### ⏭ REMAINING ON THIS INTENT

**G4** -- the display-level work. `health_tui`'s `unwrap_or(0)` and
`db.health_score().unwrap_or(0)` remain, and `8144`/`16517` return an empty
success where a refusal may be better. Every `unwrap_or_default()` Claude
introduced has been removed.

**G5** -- a runtime test asserting the shell works and the integrations report
their absence. ⚠️ Not a source-text check.

**G7** -- evidence per gate.
"""

with io.open(PATH, "w", encoding="utf-8") as fh:
    fh.write(text + addition)

print("updated " + PATH)
