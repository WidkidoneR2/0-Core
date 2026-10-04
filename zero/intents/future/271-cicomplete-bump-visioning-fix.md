---
id: 271
date: 2026-10-03
type: future
title: "Cicomplete Bump visioning fix"
status: planned
tags: [nsh, novashell, cicomplete, bum-version]
---

## Vision
cicomplete is where finished work becomes a version. Today that step tells three lies. It prints a
bump-versions command the writer ignores. It moves the intent to complete/ before asking, so an
interrupt strands every crate not yet answered. And it proposes a level from intent type, which
INT-102 Decision 5 rejected: type answers why a change was made, semver answers what it did to the
contract. After this intent, every command the close prints is one the writer runs, the versions
owed are written on the intent before the move, an interrupted close has a way back in, an
unattended close either receives an explicit decision or refuses, and no level is ever proposed
from intent type. The human picks the digit (INT-102 D4); the tool records it and applies it.

## Why Now
- 2026-10-03: the close handed `bump-versions novashell patch` seven times. bump_versions_cmd
  enters its write branch only when the first token is a level, so each paste printed the table,
  wrote nothing, and raised no error.
- The same day a close was interrupted after two answers. The file had already been moved to
  complete/; the rerun hit the already-complete early return and the remaining crates had no door.
- The questions are print! to stdout, so a redirect swallowed them.
- semver_level_for_type maps `type: feature` to minor and everything else to patch. INT-102 D5
  rejected exactly that mapping, and G5 was ticked on 2026-08-14 because the close asked rather
  than inferred. A type-derived default is that mapping back.
- `bump-versions apply` and the label `core (engine)` are both offered as things to type. Neither
  works.

## What
1. One writer contract. `bump-versions` with no args prints the table and writes nothing.
   `bump-versions <tool> <patch|minor|major>` writes; `<level> <tool>` is accepted as the old
   order. Anything else exits non-zero with usage and writes nothing. The `apply` branch is deleted.
2. Every printed command parses. Suggestions name the tool id (`engine`), never the label.
3. Versions recorded before the move. cicomplete writes a Versions section into the intent while
   it is still in in-progress/, one open box per touched crate, settles each box, and moves the
   file only when no box is open.
4. Resume door. An already-complete intent with an open box settles it. `cicomplete <id>
   --bumps-only` is the door for a file moved before the section existed: no gate check, no move,
   touched crates recomputed from git log --grep=INT-<id>.
5. Prompts on stderr, flushed before the read. Status lines stay on stdout.
6. Unattended close. `--bump <tool>=<patch|minor|major|skip>` (repeatable) and `--skip-bumps`. No
   terminal and no decision: boxes stay open, the file is not moved, exit is non-zero, and no
   `run:` line is printed.
7. No type-derived level. The prompt shows the current version and the choices with no default
   taken from type; an empty answer leaves the box open. The draft `--accept-proposed` is not
   built: there is nothing honest for it to accept.

## Approach
- Order, one commit per step: (1) writer parser and suggestion string, so the printed command
  becomes true; (2) the Versions section, settle-before-move, stderr prompts and the resume door,
  so an interrupt is recoverable; (3) the unattended flags, which tick the same boxes without a
  human; (4) the type-derived default removed.
- Two writers, one semver step. bump-versions calls apply_version_bump; cicomplete calls
  engine_apply_bump. cicomplete never shells out to bump-versions. Phase 0 records whether the two
  share one semver function; if they are copies, they get one owner in this intent before step 1.
- One row parser. A version row is read and written by one function, used by the gate filter, the
  section writer, the settler and the rewriter. The draft parsed the same row three ways.
- Every check before the first write. An unknown tool, an unknown level, or a --bump naming a crate
  the intent did not touch refuses before any Cargo.toml or intent file is written. An explicit
  --bump wins over --skip-bumps for its own crate.
- Tests that write Cargo.toml run on a scratch tree; the real tree receives only the real bumps.
  Interactive behaviour is exercised in a real terminal, not only through a pipe.
- No shelling out between the writers, no sudo, no Omarchy code. AGENTS.md is not edited; a rule it
  needs is proposed.

## Phases
Phase 0 -- recon, reading only. Record file:line here for: bump_versions_cmd, tool_cargo_path,
  bump_semver, apply_version_bump (novashell commands/mod.rs); complete_intent's status early
  return, the move block, the INT-326 question block, the no-TTY "would bump" print,
  engine_apply_bump, semver_level_for_type (engine domains/intent/mod.rs); the Complete arms in
  cli/parser.rs, cli/commands.rs, cli/mod.rs and app/dispatcher.rs; how cicomplete reaches
  core intent complete and whether it forwards trailing arguments; the commit that introduced
  semver_level_for_type; whether zero_core::paths::crate_rel_dir and CRATE_PARENTS_HISTORY exist;
  the shape of CommandResult::Error.
Phase 1 -- writer contract and suggestion string.
Phase 2 -- Versions section, settle-before-move, stderr prompts, resume door, --bumps-only.
Phase 3 -- unattended flags.
Phase 4 -- type-derived default removed.
Phase 5 -- the intent interrupted 2026-10-03 settled through the new door; ship; doors.

## Gates
- [ ] Phase 0: every seam listed under Phases recorded here with file:line; the cicomplete forwarding path recorded; whether the two writers share one semver step recorded (one owner made in this intent if not); the commit that introduced semver_level_for_type named
- [ ] Red baseline on the deployed binary before any edit: `bump-versions novashell patch` prints the table, exits 0, and the novashell Cargo.toml is unchanged (cmp against a copy)
- [ ] Writer contract on the debug binary against a scratch tree: `<tool> <level>` and `<level> <tool>` write the same new version; `bump-versions novashell`, `bump-versions apply` and `bump-versions frob patch` exit non-zero with usage and leave every Cargo.toml byte-identical (cmp); no args prints the table and writes nothing
- [ ] Every command cicomplete prints is accepted by bump-versions: a test feeds each suggested line through the writer parser; the engine crate is suggested as `engine`, never `core (engine)`
- [ ] One version-row parser, class-tested red first against the draft parse: an open row, a settled bump row, a settled skip row, and a gate line that merely contains a dotted number (draft defect: on an open row the whitespace split puts the tool at index 3, so nth(2) returns the closing bracket)
- [ ] An open version row never blocks its own close: an intent still in in-progress/ with one open row and no open gates passes the gate check (red first: the draft exemption read index 3, the tool name, for a dot and counted the row as an open gate)
- [ ] Settle before move: a two-crate close interrupted with Ctrl+C after the first answer leaves the file in in-progress/ with one box ticked and one open; the first Cargo.toml moved exactly once; the rerun asks only for the open crate, then moves the file
- [ ] No double bump: rerunning the close, or --bumps-only, after a row is settled leaves Cargo.toml and Cargo.lock byte-identical (cmp)
- [ ] Questions on stderr: in a terminal, `cicomplete <id> > /tmp/int271-out.txt` still shows every question; the file holds status lines only
- [ ] Resume door: an already-complete intent with an open box settles it; one with no Versions section and no flag prints the --bumps-only hint and writes nothing; --bumps-only on a moved file builds the section from git log --grep=INT-<id> and settles it
- [ ] Unattended refuses honestly: stdin not a terminal and no decision flag exits non-zero, leaves every box open, does not move the file, and prints no `run:` line; a malformed --bump, an unknown level, or an untouched crate refuses before any write (cmp on every Cargo.toml and the intent file)
- [ ] No type-derived level: red first, a `type: feature` close today offers minor as its default; after, the prompt offers no default, an empty answer leaves the box open, and the close path has no caller of semver_level_for_type
- [ ] The intent interrupted on 2026-10-03 has its remaining crates settled through --bumps-only; every row of its Versions section is ticked
- [ ] Doors per phase commit: the result on PATH after ship, nsh-test all passing, d 0 failed, plus that phase's own door above

## Notes
- Source: Christian's 2026-10-03 analysis of the three seams, with a draft implementation. Its line
  numbers are from main on 2026-10-03; match the function, not the number.
- The draft is a starting point, not the patch. Defects found reading it, each fixed before it is
  applied:
  - Row parsing by whitespace index misreads every open row. rewrite_version_line never settles an
    open box, so a rerun would bump the same crate again; ensure_version_section re-appends open
    rows on every pass.
  - The gate-filter exemption checks the wrong index and blocks the very resume path it exists for.
  - A --bump naming an unknown level or an untouched crate is silently left open instead of refused.
  - find_intent_file falls back to starts_with(id), so id 27 matches 271-...; reuse the finder
    load_all already uses, or match "<id>-".
  - touched_crates looks back only 60 days; an older intent loses its commits.
  - The accept-proposed path can return an error mid-loop after earlier crates were written: a
    partial close. Moot once that flag is not built.
- INT-102 is the authority: D4 suggest, never auto-decide; D5 no type-to-semver mapping; Decision 6
  the ledger answers whether, the human answers which. The 2026-08-22 ruling also declined
  weight-based suggestions. This intent restores D5. It does not invent a measure.
- The level is the human's judgement against the tiers in docs/NSH-COMPATIBILITY.md: breaking a
  CONTRACTUAL promise is major, fixing an ERRONEOUS one is patch. Not in scope: a field letting an
  intent declare its tier. No such field is added here.
- Join key: touched crates come from git log --grep=INT-<id>. A commit without INT-<id> in its
  message or Intent: trailer is invisible to the close and to --bumps-only.
- INT-326 and INT-332/INT-130 are cited in the code. Whether ledger files exist for them is
  UNVERIFIED (see the phantom-citation finding).
- Cargo.lock sync stays where it is, per crate, after a successful write. Whether
  `cargo update -p <member> --precise` behaves as intended on a workspace member is UNVERIFIED;
  the no-double-bump gate checks Cargo.lock as well.
- Related: INT-102 (version architecture).

## The Rule
"A command the tool prints is a promise the tool keeps. A digit the tool guesses is a promise nobody made."
