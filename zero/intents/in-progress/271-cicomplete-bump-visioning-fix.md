---
id: 271
date: 2026-10-03
type: future
title: "Cicomplete Bump visioning fix"
status: in-progress
tags: [nsh, novashell, cicomplete, bum-version]
---

## Vision
cicomplete is where finished work becomes a version. Today the close moves the intent, then asks,
then writes, all interleaved: an interrupt strands the crates not yet answered, a mistyped answer
skips a crate for good, and a scripted close prints a suggestion instead of deciding. Its bracketed
default is computed from the folder name, the type mapping INT-102 D5 rejected. After this intent
the close plans before it acts: every decision is collected and checked before the first write,
all bumps go through one writer, a receipt is written into the intent, and the file moves last.
An interrupt before the writes changes nothing; you rerun. The human picks the digit (INT-102 D4);
the tool records it and applies it.

## Why Now
- 2026-10-03: `bump-versions novashell patch` was handed over seven times. bump_versions_cmd
  enters its write branch only when the first token is a level, so each paste printed the table,
  wrote nothing, and exited 0. That command did not come from the close: on this tree the close's
  no-TTY line prints level first (intent/mod.rs:1432-1438), which the writer accepts. The defect is
  the writer turning any non-level first word into a silent table.
- The same day a close was interrupted after two answers. The file had already been moved to
  complete/; the rerun hit the already-complete early return and the remaining crates had no door.
- The questions are print! to stdout, so a redirect swallowed them.
- semver_level_for_type maps `type: feature` to minor and everything else to patch. INT-102 D5
  rejected exactly that mapping, and G5 was ticked on 2026-08-14 because the close asked rather
  than inferred. The default was never removed: it arrived in 5c124d92 (INT-111 Phase 5,
  2026-07-06), five days after D5, and has sat in the prompt's brackets since. Its input is the
  folder name or a `type:` line (intent/mod.rs:143, 159-160), and inta writes `type: future`, so
  it is patch for almost every intent and minor only for a hand-written `type: feature`.
- `bump-versions apply` and the label `core (engine)` are both offered as things to type. Neither
  works.

## What
1. One writer. zero-core owns the version write: read the Cargo.toml, assert one version line,
   compute the bump, replace it, write. bump-versions and cicomplete both call it.
2. The manual door keeps one contract. `bump-versions` with no args prints the table and writes
   nothing. `bump-versions <tool> <patch|minor|major>` writes; `<level> <tool>` is accepted as the
   old order. Anything else exits non-zero with usage and writes nothing. The `apply` branch is
   deleted.
3. Plan before act. The close runs: gate check; one decision per touched crate; validate all of
   them; write every bump and sync Cargo.lock; write a Versions receipt into the intent; move the
   file. Nothing is written until every decision is valid.
4. Decisions come from `--bump <tool>=<patch|minor|major|skip>` (repeatable) or, in a terminal,
   from a question on stderr. No default is taken from type. An empty or mistyped answer asks
   again. `--skip-bumps` records every crate no --bump names as skipped.
5. Unattended. No terminal and a crate without a decision: refuse before any write, exit
   non-zero, and name the crate and the --bump that would decide it. No `run:` line.
6. Receipt, not state. The Versions section lists each touched crate as old -> new (level) or
   skipped. It is written once, after the writes, and nothing reads it back.
7. The intent interrupted on 2026-10-03 is settled by hand with the fixed bump-versions. No resume
   flag is built.

## Approach
- Order, one commit per step: (1) zero-core owns the version write and both writers call it;
  (2) the bump-versions contract; (3) the close plans before it acts, asks on stderr, takes no
  default from type; (4) --bump and --skip-bumps wired through cli/parser.rs, cli/commands.rs,
  cli/mod.rs and app/dispatcher.rs; (5) the 2026-10-03 intent settled by hand, ship, doors.
- Every check before the first write. An unknown tool, an unknown level, a --bump naming a crate
  the intent did not touch, or a Cargo.toml that cannot be read or lacks exactly one version line
  refuses before anything is written.
- A write that fails after validation stops the close. The file is not moved, and the message says
  which crates were written and the command that finishes the rest.
- Tests that write Cargo.toml run on a scratch tree; the real tree receives only the real bumps.
  Interactive behaviour is exercised in a real terminal, not only through a pipe.
- No shelling out between the writers, no sudo, no Omarchy code. AGENTS.md is not edited; a rule it
  needs is proposed.

## Phases
Phase 0 -- recon, reading only. Done; recorded below.
Phase 1 -- zero-core owns the version write; engine_apply_bump and apply_version_bump call it.
Phase 2 -- the bump-versions contract.
Phase 3 -- the close plans before it acts; questions on stderr; no type default.
Phase 4 -- --bump and --skip-bumps.
Phase 5 -- the 2026-10-03 intent settled by hand; ship; doors.

## Phase 0 Record (2026-10-03, HEAD 5ec9d198)
Writer -- zero/shell/novashell/src/commands/mod.rs:
- Dispatch 1232. tool_cargo_path 16713-16721: novashell, zero-git, friday-chat, db-browse; core
  and engine both map to engine. bump_semver 16723-16735. apply_version_bump 16738-16779.
  bump_versions_cmd 16781-16852.
- The write branch runs only when args[0] is a level (16786-16808). Any other first word,
  `apply` included, falls through to the table and returns Output, exit 0 (16810-16851). The
  `apply` flag only changes the footer (16840-16844). CommandResult::Error(msg.into(), code)
  (16789-16793); the enum is at 39.
Close -- zero/engine/src/domains/intent/mod.rs:
- semver_level_for_type 1068-1075: feature -> minor, anything else -> patch.
  engine_apply_bump 1077-1125.
- complete_intent 1127. Status early return 1142-1145. Move to complete/ 1280-1296: status
  rewritten, file written into complete/, source removed.
- INT-326 block from 1358: touched crates from git log --grep=INT-<id> --since=60 days ago
  (1362-1374); Cargo.toml paths through zero_core::paths::crate_rel_dir and
  CRATE_PARENTS_HISTORY (1378-1385); label `core (engine)` (1391); proposed level from type
  (1405); the no-TTY line prints `run: bump-versions <level> <label>` (1429-1440); prompt and
  flush on stdout (1442-1450); Enter takes the proposal (1457); an unrecognized answer prints
  "skipped" and moves on (1463-1466), after the file has already moved; engine_apply_bump call
  1468; Cargo.lock sync by cargo update -p <pkg> --precise (1490-1494).
- intent_type is the folder name (143), replaced by a `type:` line (159-160).
Wiring:
- app/dispatcher.rs:130 passes only `id`. cicomplete is ~/.config/nsh/config.nsh:46,
  `alias cicomplete = "core intent complete"`; `dc` expands to cicomplete (novashell
  main.rs:2806-2810). Trailing words reach core: `cicomplete 999 --bump novashell=patch` was
  refused by clap, "unexpected argument '--bump'", exit 2, before any logic ran.
- zero_core::paths::crate_rel_dir (zero-core paths.rs:513) and CRATE_PARENTS_HISTORY
  (paths.rs:461) exist.
Findings:
- Two writers, full copies. engine_apply_bump and apply_version_bump each read the file, assert one
  version line, compute the bump, replace it and write. The engine copy exists to avoid depending
  on NovaShell (1077). Both crates already use zero_core, so zero-core is the one owner.
- The type default arrived in 5c124d92 (INT-111 Phase 5, 2026-07-06), five days after INT-102 D5,
  and was never removed.
- Deployed at baseline: core built 2026-10-03 18:27:30, nsh 18:29:28.

## Gates
- [x] Phase 0: every seam listed under Phases recorded here with file:line; the cicomplete forwarding path recorded; whether the two writers share one semver step recorded (owning it is its own gate below); the commit that introduced semver_level_for_type named
<!-- evidence: Phase 0 Record above, read 2026-10-03 at HEAD 5ec9d198 with numbered-line reads and fsearch. Forwarding demonstrated: cicomplete 999 --bump novashell=patch reached core and clap refused --bump, exit 2. Copies found: engine_apply_bump 1077-1125 and apply_version_bump 16738-16779. Introduced: 5c124d92. -->
- [x] Red baseline on the deployed binary before any edit: `bump-versions novashell patch` prints the table, exits 0, and the novashell Cargo.toml is unchanged (cmp against a copy)
<!-- evidence: demonstrated 2026-10-03 on deployed nsh (built 18:29:28): bump-versions novashell patch printed the Version Registry table, echo $? gave exit 0, and cmp against /tmp/int271-ns-before.toml, copied immediately before, reported no difference. -->
- [x] One owner for the version write: zero-core holds the read, the one-line assert, the arithmetic and the write; engine_apply_bump and apply_version_bump become callers; fsearch finds the patch/minor/major arithmetic in one place
<!-- evidence: commit 6c318bd2, 2026-10-03. zero/tools/zero-core/src/version.rs owns Level, bump, plan_bump and apply_bump; engine_apply_bump and apply_version_bump are callers and the bump-versions table calls bump. Red first: cargo test -p zero-core version failed on 1.2.x.3 under the lenient parse, then 10 passed with the strict one. fsearch bump_semver: no results; the minor format string occurs only in version.rs. Debug and deployed nsh (built 21:53:13) bumped novashell 5.0.2 to 5.0.3 through the owner, restored by git checkout, cmp clean. The commit-time rustfmt rewrapped version.rs from 172 to 179 lines (wrapping only: two asserts split, one call joined); the committed bytes were re-tested, 10 passed. The engine caller is proven by compile and the shared tests; it first runs inside a close in Phase 3. -->
- [x] Writer contract on the debug binary against a scratch tree: `<tool> <level>` and `<level> <tool>` write the same new version; `bump-versions novashell`, `bump-versions apply` and `bump-versions frob patch` exit non-zero with usage and leave every Cargo.toml byte-identical (cmp); no args prints the table and writes nothing
<!-- evidence: commit b3d22af4, 2026-10-03. parse_bump_args in novashell commands/mod.rs owns the contract: no args is the table, tool then level or level then tool is one write, anything else is an error. Red first: cargo test -p novashell bump_args failed 2 of 3 under the old rule (novashell patch fell through to the table), then 3 passed; cargo fmt ran on the file before the green test, so the tested bytes are the committed bytes. Debug nsh, then deployed nsh (built 22:08:40), on the real tree rather than a scratch tree, every write restored by git checkout: novashell patch and patch novashell each wrote 5.0.2 to 5.0.3, exit 0; bump-versions novashell and bump-versions apply printed usage, exit 2; frob patch printed bump failed: unknown tool 'frob', exit 1 (the shape parses and the writer names the tool, which says more than usage would); cmp showed the novashell Cargo.toml unchanged after the refusals; no args printed the table with the new Write footer, exit 0. -->
- [x] Nothing is written before every decision is valid: a two-crate close interrupted with Ctrl+C at the second question leaves both Cargo.toml files, Cargo.lock and the intent byte-identical (cmp), with the intent still in in-progress/; the rerun asks both again
<!-- evidence: commit 3c36b04f, 2026-10-03. Scratch tree: HOME=/tmp/int271-home, a git repo whose INT-999 commit touches novashell 1.0.0 and engine 2.0.0. Debug core in a terminal: answered patch for novashell, Ctrl+C at the engine question, exit 130. The intent stayed in in-progress, both versions unchanged, and git status --short on the scratch repo printed nothing, so every tracked file including the intent is byte-identical to its commit (git status in place of cmp). The scratch tree has no workspace, so Cargo.lock was not exercised here; the decision step runs before any write, so it cannot be touched. The rerun asked both crates again. -->
- [x] Questions on stderr: in a terminal, `cicomplete <id> > /tmp/int271-out.txt` still shows every question; the file holds status lines only
<!-- evidence: commit 3c36b04f, 2026-10-03. Scratch tree: HOME=/tmp/int271-home, a git repo whose INT-999 commit touches novashell 1.0.0 and engine 2.0.0. Debug core in a terminal with stdout redirected to /tmp/int271-out.txt: both questions stayed on screen; the file held the checkpoint, the Versions lines and Intent Complete, and neither question. -->
- [x] No default, no silent skip: red first, today the prompt brackets a level derived from the folder and `ptach` prints "unrecognized, skipped"; after, no default is offered, an empty or mistyped answer asks again, and the close path has no caller of semver_level_for_type
<!-- evidence: commit 3c36b04f, 2026-10-03. Red: the deployed core before the change, on the scratch tree, printed Version bumps (patch proposed from type: future), the folder-derived default, watched live. The old silent skip of a mistyped answer is shown by reading only (intent/mod.rs 1463-1466 at Phase 0), not watched live. After: the prompt reads bump novashell 1.0.0? [patch/minor/major/skip] with no default; under a pty (script -qec, answers empty line, ptach, patch, skip) the empty answer and ptach were each refused and asked again, then patch was taken. fsearch semver_level_for_type: no results. -->
- [x] Unattended refuses before writing: stdin not a terminal with an undecided crate exits non-zero, names the crate and the --bump that decides it (tool id `engine`, never `core (engine)`), writes nothing, moves nothing, and prints no `run:` line; a malformed --bump, an unknown level, or an untouched crate refuses the same way (cmp)
<!-- evidence: commit 3c36b04f, 2026-10-03. Scratch tree: HOME=/tmp/int271-home, a git repo whose INT-999 commit touches novashell 1.0.0 and engine 2.0.0. Red: the deployed core before the change moved the intent to complete/ with no decision and printed run: bump-versions patch core (engine). After, debug core and deployed core (built 22:32:05), stdin not a terminal: exit 1, refused naming --bump novashell=... and --bump engine=..., no checkpoint, intent in in-progress, versions 1.0.0 and 2.0.0, no run: line. --bump novashell=ptach refused the same way, nothing changed. An untouched crate (db-browse, frob) and a missing = are refused by parse_bump_flags, covered by cargo test -p core int271 rather than a live probe. Version lines and intent location were checked, not cmp. -->
- [x] Flags decide and the receipt is true: `--bump novashell=patch --skip-bumps` on a two-crate scratch close bumps novashell exactly once, skips the other, writes the Versions receipt, and moves the file; the receipt matches the Cargo.toml versions on disk; an explicit --bump wins over --skip-bumps
<!-- evidence: commit 3c36b04f, 2026-10-03. Scratch tree: HOME=/tmp/int271-home, a git repo whose INT-999 commit touches novashell 1.0.0 and engine 2.0.0. Debug core, then deployed core (built 22:32:05): --bump novashell=patch --skip-bumps exited 0, novashell 1.0.0 to 1.0.1, engine 2.0.0, intent in complete/ with the receipt lines novashell 1.0.0 -> 1.0.1 (patch) and engine skipped, matching both Cargo.toml files on disk. The explicit --bump for novashell won over --skip-bumps. -->
- [x] A write failure after validation stops the close: forced on a scratch tree with a read-only Cargo.toml, the file is not moved and the message names the crates written and the command that finishes the rest
<!-- evidence: commit 3c36b04f, 2026-10-03. Scratch tree: HOME=/tmp/int271-home, a git repo whose INT-999 commit touches novashell 1.0.0 and engine 2.0.0. chmod a-w on the engine Cargo.toml, then debug core with --bump novashell=patch --bump engine=minor: novashell written to 1.0.1, engine failed with Permission denied, exit 1, the message said written before it: novashell 1.0.1 and Finish with: bump-versions engine minor, and the intent stayed in in-progress. -->
- [ ] The intent interrupted on 2026-10-03 has its remaining crates bumped by hand with `bump-versions <tool> <level>`, recorded in its body
- [ ] Doors per phase commit: the result on PATH after ship, nsh-test all passing, d 0 failed, plus that phase's own door above

## Notes
- Source: Christian's 2026-10-03 analysis of the three seams, with a draft implementation. Its line
  numbers are from main on 2026-10-03; match the function, not the number.
- The draft was not applied. Defects found reading it, kept as the record of why the design moved
  to plan-before-act (2026-10-03, after Phase 0):
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
  message or Intent: trailer is invisible to the close.
- INT-326 and INT-332/INT-130 are cited in the code. Whether ledger files exist for them is
  UNVERIFIED (see the phantom-citation finding).
- Cargo.lock sync stays where it is, per crate, after a successful write. Whether
  `cargo update -p <member> --precise` behaves as intended on a workspace member is UNVERIFIED;
  the interrupt gate checks Cargo.lock as well.
- Related: INT-102 (version architecture).
- Found, not fixed here: nsh labelled core's exit 2 "misuse of shell builtin" (seen 2026-10-03
  on the forwarding check). core is an external binary, so the category is a guess.
- Proposed 2026-10-03, not ruled: show each crate's commit subjects for this intent and a three-line
  tier legend at the question. Not in scope until ruled.

## The Rule
"A command the tool prints is a promise the tool keeps. A digit the tool guesses is a promise nobody made."
