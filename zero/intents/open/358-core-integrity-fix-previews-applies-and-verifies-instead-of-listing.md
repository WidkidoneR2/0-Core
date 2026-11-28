---
id: 358
date: 2026-11-27
type: future
title: "core integrity fix previews, applies and verifies instead of listing"
status: planned
tags: [core, integrity, doctor, proposals]
depends_on: []
format: 3
---

## Vision
`core integrity fix` does what its name says. It shows each
pending proposal with the exact change it would make.
`--dry-run` shows the same and writes nothing. Without it,
every proposal the engine can carry out is applied, checked
again afterward, and reported as applied, refused or needs
hand. `core integrity status` counts what `d` counts. A
proposal nobody can apply, the README tools count, is no
longer made, so `d` stops reporting "1 integrity
proposal(s) pending" for a change that does not exist.

## The Problem
Measured 2026-11-27 on a box checkout of origin/main
c37e3bc8 (core 5.3.2), built with cargo +1.88.0, run on a
scratch HOME copy of the tree and a scratch state.db. The
laptop's state was not touched.

1. `fix` lists and fixes nothing. cmd_fix
   (integrity/mod.rs:718-768) runs the whole pipeline
   (722), which applies any AutoFix silently and drops the
   count, then prints each pending row as `#id [category]
   description`, a check name and "Apply this fix? (run:
   core integrity apply N)". It shows no change and
   applies nothing. The flag list is empty
   (cli/parser.rs:1754-1755): `core integrity fix --help`
   prints only "Usage: core integrity fix", and
   `fix --dry-run` is an unknown-argument error.
2. `apply` carries out two kinds of fix. apply_row
   (mod.rs:1522) handles UpdateRegistryVersion (1578) and
   MoveFile (1593) and nothing else; any other type
   returns "has no stored X fix to carry out; apply it by
   hand" (1611). It marks the row applied right after the
   write (1618-1624) and never checks that the check
   stops finding it.
3. `status` counts retired rows. cmd_status (604-612)
   counts `applied_at IS NULL`. The one definition of
   pending is PENDING (mod.rs:303), `applied_at IS NULL
   AND retired_at IS NULL`, and fix, apply, heal and the
   reconcile all use it. Reproduced on the scratch db: with
   one real proposal and one retired row, status says 2
   and fix lists 1. Christian's laptop showed the same
   gap: status 5, `d` 1.
4. The one proposal is false, and unappliable.
   docs_count_consistency (mod.rs:1286-1342) takes the
   first README line that has "tools deployed", "tools
   installed", "key tools" or "(middle dot) tools:" and
   not "install" or "pipeline", then the first word of it
   that parses as a number once non-digits are trimmed.
   README.md:150 is a sample doctor line,
   `Path Resilience  17/17 tools deployed (100%)`.
   "17/17" does not parse and "(100%)" does, so the
   check reads 100. It compares that to every
   `name = "` line in registry/tools.toml, 37, of which 9
   are retired. Scratch run: pending #1 [documentation]
   "README shows 100 tools but registry has 37", action
   SyncDocs. `core integrity apply 1` answers "apply it by
   hand". The fix could not work even if it ran:
   `zero-docs sync` writes only the tools index and
   docs/public, and never README.md (zero-docs
   main.rs:4-6 ownership rule, and the sync arm at
   100-104), so the same line is found again. The result
   is `d` showing "1 integrity proposal(s) pending" and
   Integrity at 92% on every run (box: one weight-1
   issue over 12 units of weight, 92%).
5. Registry drift is the only proposal that can be real.
   registry_version_drift (mod.rs:1134) proposes
   UpdateRegistryVersion, which apply can carry out through
   zero_core::registry::write_version (253-264 and 1578).
   Only these two checks ever call IntegrityIssue::propose
   (1134, 1326).

Related, found and left alone: `core integrity heal
--dry-run` runs run_pipeline before it looks at dry_run
(mod.rs:1724), so it persists proposals and applies
AutoFix fixes on a "dry" run; and run_pipeline's
`safe_only` flag guards an empty block (486-488). Neither
is in this intent.

## The Solution
All work is in zero/engine/src/domains/integrity/mod.rs,
plus the four files that carry the new flag. The tests
share the stand_in() stand-in repo already in mod.rs
(a two-tool registry one version behind its crates, over an
in-memory database).

G1 adds the red tests and the seams they need, no behaviour
change. Seams (all in integrity/mod.rs, public to the
crate):
`fn pending_count(ctx: &AppContext) -> i64`, holding
today's cmd_status query unchanged;
`enum FixVerdict { WouldApply, Applied, Refused(String),
NeedsHand(String) }`;
`struct FixLine { id: Option<i64>, category: String,
check: String, description: String, change: Vec<String>,
verdict: FixVerdict }`;
`struct FixReport { dry_run: bool, auto_fixed: usize,
lines: Vec<FixLine> }`; and a stub
`fn fix_run(ctx: &AppContext, checks:
&[Box<dyn IntegrityCheck>], dry_run: bool) -> FixReport`
that returns an empty report. Tests are named int358_ so
one filter runs them: `cargo +1.88.0 test -p core
int358_`. All six are red today and fail on an assertion,
none errors. The sixth also holds two parse guards that
already pass.

- `int358_status_counts_only_pending` (red). Insert three
  rows: pending, retired, applied.
  pending_count(&ctx) is 1. Today's query gives 2.
- `int358_dry_run_previews_and_writes_nothing` (red).
  stand_in with the two drifts. Take the bytes of
  registry/tools.toml and the row counts of pending_fixes
  and integrity_log. fix_run(&ctx, &drift_only(), true)
  returns 2 lines; the alpha line's change text contains
  `1.0.0` and `2.0.0`, the core line's contains `3.0.0`
  and `4.0.0`; verdicts are WouldApply; tools.toml is
  byte-identical; both row counts are 0.
- `int358_fix_applies_and_verifies_every_drift` (red).
  fix_run(.., false) on the same stand_in: two Applied
  lines; tools.toml now has alpha 2.0.0 and core 4.0.0 and
  nothing else changed; a fresh RegistryVersionDriftCheck
  finds 0; both rows have applied_at set.
- `int358_fix_sorts_refused_needs_hand_and_unverified`
  (red). A test-local check named
  `registry_version_drift` (so the reconcile keeps its
  rows) proposes three things: a SyncDocs fix, a MoveFile
  whose target already exists, and an UpdateRegistryVersion
  for alpha to 9.9.9 that the check keeps reporting after
  the write. The report has one NeedsHand, one Refused
  (`would overwrite`) and one Refused whose text says the
  check still finds it; the two files are untouched; the
  unverified row is still pending, not applied.
- `int358_docs_count_check_is_gone_and_100_percent_is_not_a_count`
  (red). build_check_suite() names do not include
  docs_count_consistency; a stand_in README holding
  `17/17 tools deployed (100%)` produces no issue from the
  full suite.
- `int358_fix_dry_run_flag_parses_and_fix_alone_still_does`
  (red, in cli/parser.rs next to the INT-330 parse tests).
  `Cli::try_parse_from(["core", "integrity", "fix",
  "--dry-run"])` is Ok, `["core", "integrity", "fix"]` is
  Ok (already true), and `["core", "integrity", "apply",
  "7"]` is Ok. Only the `--dry-run` assertion is red.

G2 is the count. cmd_status calls pending_count, and
pending_count uses PENDING (mod.rs:303). The status line
text does not change. `int358_status_counts_only_pending`
goes green. status reads the table as of the last scan;
`d`, `fix` and `run` reconcile first. That is stated, not
changed.

G3 removes docs_count_consistency. Decision: remove, do
not repair. Why: the check measures a sample screen in the
README and a count of registry lines, so no edit to the
check makes "the README is wrong" mean anything; the
README is person-owned and zero-docs refuses to write it,
so SyncDocs cannot ever fix what it flags; and zero-docs
already measures what it owns (`zero-docs check` exits 1 on
a stale tools index, `zero-docs status`). Delete
DocsCountConsistencyCheck (mod.rs:1286-1342) and its line
in build_check_suite (822) and the "Phase 6: Documentation"
comment. The existing pending row is retired by the
reconcile as `check removed` at the next run, so nothing
migrates by hand. The FixAction::SyncDocs variant and its
apply_safe_fix arm (253) stay: removing a variant is a
larger change than this intent, and after G5 a SyncDocs row
is reported as needs hand, never as applied. Expected on
the laptop after G7: `d` has no "integrity proposal(s)
pending" line and Integrity reads 100% (11 checks to 10,
one weight-1 issue gone).

G4 is the preview and the dry run, in fix_run. It adds
`fn preview_stored(ctx, action_type, action_data) ->
(Vec<String>, Preflight)` where Preflight is Apply,
Refuse(reason) or Hand(reason), decided from the same
stored data apply carries out, so what is shown is what is
done.
- UpdateRegistryVersion: read registry/tools.toml, find the
  row's current version with zero_core::registry::parse,
  build the new text with zero_core::registry::set_version
  (the function the write uses), and show the one changed
  line as `zero/registry/tools.toml:N`, a `- version =
  "old"` line and a `+ version = "new"` line. A set_version
  refusal is Refuse with its reason.
- MoveFile: `move FROM -> TO`; Refuse when FROM is absent
  or TO exists (apply_row's own two tests).
- anything else: the stored description, and Hand.
Dry run (dry_run true) runs every check in `checks` with
`check.run()` only, keeps issues that are proposals with a
fix, and builds lines from them. It does not call
ensure_tables, run_pipeline, persist_proposal, reconcile or
log_issue, and it writes no file. `id` is the matching
pending row's id when one matches on check, subject and
data (read-only), else None. Verdict is WouldApply, Refused
or NeedsHand from Preflight. Dry-run lines for proposals
whose check is not in `checks` do not appear.

G5 is the apply, in fix_run (dry_run false) and apply_row.
1. fix_run runs run_pipeline(.., true) on `checks` as
   cmd_fix does now, keeps the PipelineResult's auto_fixed
   count in the report, reads the PENDING rows of those
   checks, orders them by created_at, and for each builds
   the preview lines then applies the row.
2. apply_row takes the check list it re-runs, instead of
   building build_check_suite() inside (mod.rs:1546-1549);
   cmd_apply and heal pass build_check_suite(), fix_run
   passes its own `checks`. Its error becomes `enum
   ApplyError { Refused(String), NeedsHand(String) }`.
   Display is the same text as today, so `core integrity
   apply` and `heal` print what they printed before. The
   "has no stored X fix" branch is NeedsHand; every other
   Err branch is Refused.
3. Verification, inside apply_row after the write and
   before the row is marked applied: re-run only this
   row's check with `check.run()`; if any issue it returns
   has the row's check name and fix subject, return
   Refused("carried out, but check NAME still finds
   SUBJECT; it stays pending") and do not set applied_at.
   Otherwise set applied_at and return Ok. `apply` and
   `heal` get this check too, because it is one apply
   path.
4. Each row yields one FixLine with verdict Applied,
   Refused or NeedsHand. A refusal on one row never stops
   the others.

G6 wires the command. IntegrityCommands::Fix becomes
`Fix { #[arg(long)] dry_run: bool }` with the doc line
"Preview (--dry-run) or apply every pending proposal, and
check each afterward" (cli/parser.rs:1754-1755); the same
field on IntegrityCommand::Fix (cli/commands.rs:867), the
mapping (cli/mod.rs:612) and the dispatcher
(app/dispatcher.rs:894). cmd_fix(ctx, dry_run) calls
fix_run with build_check_suite() and prints the report:
- a header with the count and, when auto_fixed > 0, an
  "Auto-fixed: N" line;
- per proposal: `#id [category] description`, the check,
  the change lines indented under it, and one verdict line:
  `would apply`, `applied, verified`, `refused: WHY` or
  `needs hand: WHY`;
- a last line: `N applied, M refused, K need hand` or, on a
  dry run, `dry run: nothing written. N would apply, M would
  be refused, K need hand`;
- with nothing pending, the existing "No pending proposals"
  line and retired history (print_retired_history);
- exit non-zero (CoreError::Reported, as apply does) when
  any line is Refused or NeedsHand and the run was not
  dry; a dry run exits 0.
Proof on the box, on a scratch HOME copy with a scratch
db: stand_in-like drift made by editing a Cargo.toml
version in the copy; paste `fix --dry-run`, `fix`, then
`fix` again (No pending proposals) and `status` (0).

G7 is the laptop proof.

## Limits
- In scope: integrity/mod.rs (pending_count, the
  FixVerdict, FixLine and FixReport types, fix_run,
  preview_stored, ApplyError and the verification in
  apply_row, cmd_fix, cmd_status, removing
  DocsCountConsistencyCheck and its suite line, the
  int358_ tests); one flag through cli/parser.rs,
  cli/commands.rs, cli/mod.rs and app/dispatcher.rs; one
  parse test in cli/parser.rs; a core bump at close.
- Out of scope: `core intent park` (its own intent, in the
  intent domain, no shared code); a replacement docs check
  such as one that turns `zero-docs check` into an alert;
  `heal --dry-run` writing on a dry run; run_pipeline's
  dead `safe_only` flag; removing FixAction::SyncDocs or
  the apply_safe_fix SyncDocs arm; a confirmation prompt
  or a per-id selection on `fix`; applying MoveFile
  proposals from a new check (none exist); the registry
  drift check itself; zero-core, zero-docs and
  registry/tools.toml; the table schema of pending_fixes
  and integrity_log; any change to `core integrity run`,
  `log`, `trend`, or `apply`'s printed text.
- How deep: seven gates, about an evening. One new
  function (fix_run) and one helper (preview_stored); two
  existing functions change (apply_row, cmd_fix); one
  check is deleted. No new crate or dependency.
- Stop and ask: an existing integrity test fails or needs
  its assertion changed; `fix` would write any file other
  than registry/tools.toml; a dry run would need a write to
  produce its output; set_version cannot produce the
  preview for a real registry row; the laptop shows a
  proposal other than the README one before G3; `d` shows a
  new alert or Integrity below 100% after G3 for a reason
  other than a new finding; a fix seems to need a change
  outside In scope.
- Done means: G1 to G7 ticked with evidence.
- Must not change: live state.db and the laptop's tree
  (agents use scratch HOME and scratch db only), the
  README and every other person-owned doc, zero-docs,
  zero-core, the pending_fixes schema, the text `core
  integrity apply` and `heal` print today, any file
  outside the repo, and any secret or confidential value.

## Success Criteria
- [ ] G1 red first: the six int358_ tests and the seams
      (pending_count, FixVerdict, FixLine, FixReport and
      the empty fix_run) are in integrity/mod.rs and
      cli/parser.rs. All six fail on their assertions, none
      errors, and the fix-alone and apply-7 parses inside
      the sixth already pass. The 380 existing tests stay
      green.
      ask: today status counts a retired row, fix writes nothing, and the README line reads as 100 tools
      ruling:
      evidence:
- [ ] G2 pending_count uses PENDING and cmd_status calls
      it. int358_status_counts_only_pending passes and
      nothing else changes.
      ask: status counts exactly the proposals fix would list
      ruling:
      evidence:
- [ ] G3 DocsCountConsistencyCheck and its suite line are
      gone. The docs-check test passes; a stand_in with the
      old README line gives no issue; the old row retires
      as `check removed` on the next run.
      ask: the README count proposal is removed rather than repaired, because zero-docs never writes the README
      ruling:
      evidence:
- [ ] G4 fix_run with dry_run true returns a preview line
      per proposal and writes no file and no row.
      int358_dry_run_previews_and_writes_nothing passes.
      ask: the preview shows the exact line apply would write, and a dry run changes nothing
      ruling:
      evidence:
- [ ] G5 fix_run with dry_run false applies each row,
      verifies each afterward and sorts the outcomes;
      ApplyError and the verification are in apply_row;
      `apply` and `heal` print what they printed before.
      The two apply tests pass and cargo test -p core is
      green.
      ask: every proposal the engine can apply is applied and checked again, and one failure does not stop the rest
      ruling:
      evidence:
- [ ] G6 `core integrity fix [--dry-run]` is wired and
      prints the report in the shape above, with a non-zero
      exit on a refusal or needs-hand. All six int358_
      tests pass. Box scratch proof pasted.
      ask: the command reads the way it should and exits right
      ruling:
      evidence:
- [ ] G7 laptop: after merge and ship, `core integrity
      fix --dry-run` and `core integrity status` agree (both
      0 or both the same N); `d` has no "integrity
      proposal(s) pending" line and Integrity is 100%;
      `core integrity apply` and `heal` print as before;
      a scratch-copy proof of fix on one real drift is
      pasted; d is green.
      ask: the laptop agrees, and d is clean and shows no phantom proposal
      ruling:
      evidence:

## Job order
Jobs run on branch int-358 only. Agents never push to
main. Job I files the intent. G1 first, alone (it fixes the
shared type names). After G1, two lanes may run side by
side, because they edit different regions or files of
integrity/mod.rs: lane A is G4 then G5 (both inside
fix_run and apply_row); lane B is G2, G3 and G6 (G2 is
cmd_status and pending_count, G3 deletes the docs check
and its suite line, G6 is cmd_fix plus the four CLI
files). Each job pulls with rebase before it pushes. G6's
box proof needs G5, so run it last. G7 is the laptop. The
seal is the first 16 hex of sha256 of int358-plan.txt:
fc3334046a5f3dc5.

## Versions
- core minor (`core integrity fix` gains --dry-run and
  does what it names; the status count changes)
