---
id: 192
date: 2026-07-23
type: arch
title: "forest tools cannot express an undetermined outcome so failed checks report clean"
status: in-progress
tags: [arch, telemetry, observation-layer, correctness, cross-tool]
---

## Vision

A check that could not produce a result never reports one. "I did not find
anything" and "I could not look" are different answers, and every forest tool can
tell them apart.

## The Problem

Found 2026-07-23 while checking whether anything needed updating.
`faelight-update --dry-run` printed:

    ⚠️  Failed to check cargo updates: error: no such command: `install-update`
    ...
    🦀 Cargo Tools (up to date)

The check ERRORED and the summary reported UP TO DATE. Not a wrong value -- a
wrong CATEGORY. The tool knew it had failed (it printed a warning) but the warning
went to stderr while the return value went to the summary, and the summary could
not tell the difference.

The cause is four lines in cargo_checker.rs:

    eprintln!("      ⚠️  Failed to check cargo updates: {}", stderr);
    return Vec::new();

`Vec<Update>` has no room for "undetermined", so "could not check" and "nothing to
update" are THE SAME VALUE. Three separate error paths (cargo-update missing,
other error, cargo not available) all return an empty vec.

### Two distinct smells, often on the same line

  1. `return Vec::new()` on an error path -- the outcome collapses.
  2. `Err(_)` -- the REASON is discarded. Even a correct fix later cannot report
     WHY the check could not run.

faelight-update at least warns on stderr. Most other instances do neither: they
swallow the error entirely, so there is no observable difference at all between
"clean" and "the check crashed".

### Evidence (5 tools, 15+ sites, from ONE grep pattern)

    faelight-deadwood/src/main.rs      286, 408, 461, 517   Err(_) => return Vec::new()
    faelight-shell/src/git_tui.rs      132                  Err(_) => return vec![]
    faelight-shell/src/history_tui.rs  134                  Err(_) => return Vec::new()
    faelight-shell/src/triggers.rs     70                   Err(_) => return vec![]
    faelight-shell/src/cheatsheet_tui  774                  Err(_) => return vec![]
    faelight-shell/src/config.rs       207                  Err(_) => return vec![]
    faelight-shell/src/db.rs           185, 353             Err(_) => return vec![]
    faelight-docs/src/toolgen.rs       563                  Err(_) => return vec![]
    faelight-release/src/learning.rs   56                   Err(_) => return vec![]
    faelight-update/src/cargo_checker  24, 30               warn + return Vec::new()
    faelight-update/src/neovim_checker 120                  Err(_) => return Vec::new()
    faelight-update/src/flake_checker  38, 50               warn + empty

⚠️ THIS IS A GREP, NOT AN ENUMERATION. (The INT-191 gate-1 lesson: grep finds
string matches, enumeration follows the code.) The same collapse has other shapes
this pattern did not catch -- `unwrap_or_default()`, `.ok()?`, `unwrap_or(0)`,
and any function returning a count or a bool rather than a collection.

### The clearest case, and the highest-stakes one

    fn check_dead_aliases(root: &Path) -> Vec<Finding> {
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => return Vec::new(),     // config.fsh unreadable -> "no dead aliases"
        };

If config.fsh is missing, renamed, or unreadable, the check reports ZERO findings
-- indistinguishable from a clean result. A path change would silently disable the
check forever. Deadwood feeds the doctor line `Deadwood: 0 low-priority items; no
structural orphans`, which contributes to the HEALTH SCORE. So a broken check
reads as 100% health.

### Severity gradient -- not every instance is equally bad

  HIGH    feeds a score or an automated decision, invisible when wrong.
          deadwood -> health %; faelight-update -> "10 updates available".
  MEDIUM  feeds a display read as authoritative.
  LOW     a TUI list, where empty is visibly empty and the user can tell.

⚠️ AND NOT ALL 15 ARE BUGS. Some `Err(_) => Vec::new()` may be legitimate -- an
optional file that genuinely does not exist means genuinely zero entries. EACH
SITE NEEDS JUDGING, NOT BLANKET-FIXING. A mechanical sweep would be the same class
of error as the bug itself: an answer produced without looking.

## The Solution

A tri-state, missing everywhere today:

    Succeeded
    Failed(reason)
    Undetermined(why)     <- the one that does not exist

Every bug found this session is the third collapsing into one of the first two,
in BOTH directions: `spine exec ll` recorded `ll | 127` (Undetermined became
Failed); `Cargo Tools (up to date)` (Undetermined became Succeeded).

★ THE FIX IS TYPE-LEVEL, NOT LOGIC-LEVEL. `Vec<T>` cannot express doubt, so a
discipline of "remember to handle the error" will drift across 36 tools. Something
like `Result<Vec<T>, CheckSkipped>` makes the collapse a COMPILE ERROR -- the
caller must handle the undetermined case to build.

Same move as INT-169's `IoPlan` being a reserved TYPE rather than fake
stdin/stdout fields, and `lower()` returning `Result` rather than fabricating a
plan for a construct it cannot lower: MAKE THE WRONG STATE UNREPRESENTABLE.

Likely shape: a small shared crate the tools depend on. 36 binaries is too many
for a convention to hold.

### Scope boundary

This intent is the CONTRACT, not the observation layer.

The larger layer -- oversight across faelight-update, core, fsh, faelight-daemon,
faelight-git and others -- belongs with faelightd, whose design notes already list
`telemetry` as a service module alongside an event bus that would naturally carry a
`CheckSkipped` event beside `PackageInstalled`.

That layer CONSUMES this contract; it does not replace it. Tools must be able to
say "undetermined" whether or not the daemon is running.

### Judgment scaffold for gate 2 (fill this in -- do not skip it)

The per-site judgment is the hardest gate, not because it is difficult but because
it is 15+ separate readings of surrounding code. Tedium invites shortcuts, and a
shortcut here REPRODUCES THE ORIGINAL BUG: an answer produced without looking.

Two axes, deliberately separated, because they fail independently:

  A. CAN THIS ERROR FIRE IN A HEALTHY SYSTEM?
     optional file genuinely absent  -> empty is the TRUTH
     file that should always exist   -> empty is a LIE

  B. CAN THE CONSUMER TELL?
     TUI list, empty renders visibly, human is looking  -> VISIBLE
     a number feeding a score or an automated decision  -> INVISIBLE

Four quadrants. Only one is urgent:

                     | VISIBLE            | INVISIBLE
    -----------------+--------------------+---------------------------
    empty is TRUTH   | fine, leave it     | fine, but document why
    empty is a LIE   | low -- user sees   | ★ URGENT -- the lie propagates

So the 15 sites are NOT 15 equal decisions. Sort by quadrant first; the
expectation is that a handful need real care and the rest resolve in a line each.

    site                                    | A: truth or lie? | B: visible? | verdict + reason
    ----------------------------------------+------------------+-------------+------------------
    deadwood/main.rs:286 check_dead_aliases  |                  |             |
    deadwood/main.rs:408                     |                  |             |
    deadwood/main.rs:461                     |                  |             |
    deadwood/main.rs:517                     |                  |             |
    shell/git_tui.rs:132                     |                  |             |
    shell/history_tui.rs:134                 |                  |             |
    shell/triggers.rs:70                     |                  |             |
    shell/cheatsheet_tui.rs:774              |                  |             |
    shell/config.rs:207                      |                  |             |
    shell/db.rs:185                          |                  |             |
    shell/db.rs:353                          |                  |             |
    docs/toolgen.rs:563                      |                  |             |
    release/learning.rs:56                   |                  |             |
    update/cargo_checker.rs:24,30            | LIE (check ran, failed) | INVISIBLE (summary) | ★ the founding case
    update/neovim_checker.rs:120             |                  |             |
    update/flake_checker.rs:38,50            |                  |             |
    (+ whatever enumeration adds)            |                  |             |

⚠️ A "verdict" of TRUTH still needs its REASON recorded. "This file is optional"
is a claim about the system, and an undocumented one will be re-litigated -- or
silently invalidated when the file stops being optional.

### What is genuinely new here

Not the shared-crate mechanics -- those already exist. The AXIS.

The forest measures "is it working" (tests, fsh-test) and "is it healthy" (the
doctor's 34 checks). It has never measured **"do I actually know?"**. Health at
100% with a silently-dead deadwood check is the system being CONFIDENT rather than
CORRECT, and no existing instrument would catch it.

## Success Criteria

- [ ] Every site is ENUMERATED, not grepped -- including the shapes the first
      pattern missed (unwrap_or_default, .ok()?, unwrap_or(0), bool and count returns)
- [ ] Each site is JUDGED individually against the two-axis scaffold above, with the
      REASON recorded (including for sites judged fine). A blanket sweep is explicitly
      rejected -- it would be the same class of error as the bug itself
- [ ] Sites sorted by quadrant BEFORE any are fixed, so effort lands on
      "empty is a lie AND nobody can see it" first
- [x] The tri-state is expressed in the TYPE SYSTEM, so a collapse fails to compile
      rather than relying on discipline
- [ ] HIGH-severity sites first: anything feeding the health score or an automated decision
- [ ] `Err(_)` is retired wherever the reason is needed -- the why survives to the report
- [x] The doctor can distinguish "checked, clean" from "could not check", and says so
- [x] Verified by breaking a check on purpose (rename config.fsh) and confirming the
      report says UNDETERMINED rather than clean
- [x] Decided: shared crate vs per-tool convention, with the adoption surface in mind

## PROGRESS 2026-09-04 -- the contract exists and one tool uses it

THE TYPE LANDED. faelight_core::check carries Skipped {subject, reason} and
Checked<T> = Result<T, Skipped>. Not in error.rs: an error is a failure, this is an
absence of knowledge, and filing it beside FontLoad would encode the confusion the
intent exists to remove.

PROVEN BEFORE FIXING. config.nsh was moved aside and deadwood reported [ok] Dead
aliases: clean, 0|0|0|0 -- it could not read the file and said clean, and that zero
feeds the health score. The charter proposed this as the test; it was run.

The conversion produced SEVEN COMPILE ERRORS in one file, each a place that had been
reporting clean without looking. That is the type doing the enforcement a convention
could not.

END TO END, demonstrated in three states:
  normal          0|0|0|0   [ok] clean          doctor: green
  config hidden   ?|?|0|0   [??] could not check  doctor: Unknown 1 -> 2
  restored        0|0|0|0   back to green

--strict exits 1 on a skip, because a gate that could not run has not passed.
A total containing an unknown is itself unknown -- summing ? as zero would rebuild
the collapse one level up.

AND THE DOCTOR CATCH-ALL WAS ITS OWN INSTANCE: an absent deadwood reported Pass with
not installed, run after deploy. A check that reports clean by being ABSENT.

⚠️ THE ENUMERATION GATE IS THE REMAINING WORK AND IS NOT STARTED. Three sites judged
and converted out of 15+ across five tools, and a new one surfaced while testing:
the doctor Alias Coverage check reads the same config and collapses in the LOUD
direction -- clean became 21 tools missing aliases when the file was hidden. The
July list is a grep, and the two deadwood line numbers in it were already stale.

<!-- 2026-07-23 recon: the forest ALREADY has a shared library. faelight-core/src/lib.rs exists and
28 tools already depend on it via { path = "../faelight-core" }, inside a real Cargo workspace. It
is a clean crate, not a junk drawer: canvas, error, glyph, paths, theme, wayland; 51-line lib.rs of
pure declarations. So this does NOT introduce the forest's first shared contract -- that already
happened. Home: faelight_core::observe (or inside the existing error module). Adoption is one `use`
line for 28 tools, not a Cargo.toml change plus a rebuild-surface argument. The remaining question
is aesthetic (new module vs extend error), not architectural. -->
