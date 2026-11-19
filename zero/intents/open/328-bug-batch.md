---
id: 328
date: 2026-11-18
type: future
title: "Bug batch 4: merged work still counts, refusals say safe or not, nsh for and subshell, the temp-dir window"
status: planned
tags: [bugs, cicomplete, nsh, core]
format: 2
---

## Vision
A close after a --no-ff merge still sees the work that landed. A refusal
says in one first line whether it was safe (green) or partway (red). A
`for` loop in an `&&` chain, and a grouping `( ... )`, run as one construct.
The temp-dir process-id check on main is green.

## The Problem
1. `cicomplete 320 --bump zero-core=minor` refused with "this intent's
   commits did not touch zero-core (touched: none)", even though the
   INT-320 branch commits (4d53fd6d..b2d28a1e, all with `Intent: INT-320`)
   touched `zero/tools/zero-core` and `zero/engine`, and arrived through
   --no-ff merge 91c61579. `touched_crates` (intent/mod.rs:1383) calls
   `intent_git_log` (1356) with `--name-only --format=` and
   `intent_log_args` (1324): `-E --grep=INT-{id}([^0-9]|$)` and
   `--since={frontmatter date} 00:00:00` (F-0049, F-0050). The 320 file
   is dated 2026-11-18; the six branch commits are 2026-11-17T13:46Z to
   13:51Z, so `--since` drops them. The merge is in the window, but
   `git log --name-only` emits no paths for a merge by default, so the
   only later commit (44392ce0, the intent file) touches no crate.
   `parse_bump_flags` (1438) then prints "touched: none". Plain `-m`
   would list the incoming files and also the other parent's files
   (README, INT-327); the right switch is `--diff-merges=first-parent`.
   The pre-push line "no commit in the range carries Intent:"
   (trace.rs:1197) is a different bug: trailer parse worked (the seven
   commits printed intact). `.githooks/pre-push` `check_pushed_refs`
   also runs `core intent seals check` on `refs/notes/seals`, whose
   notes commits have no Intent trailer, so the hook prints that line
   on every combined push of main and the notes.

2. Refusals do not say, in one first line, whether anything changed.
   cicomplete prints "refused:" then "nothing was written and the
   intent was not moved" (mod.rs:2934, 2953, 2963). `write_versions`
   already has a partway case (a bump failed after earlier Cargo.toml
   writes, :2297 and :2323) but it leads with "bump failed" rather than
   a clear safe/partway line. fingerprint record and doctor check_one
   say "REFUSED --" in a third shape. Christian wants green for a safe
   refusal and red for a partway one, with NO_COLOR and a non-tty
   staying plain.

3. nsh: `( ... )` grouping is not supported. `FlowError::CommandNotFound`
   (error.rs:36) special-cases a command word starting with `(` and
   prints "subshells are not supported yet". `split_logical`
   (expand.rs:565) also cuts the group at a mid-paren `&&`.

4. nsh: `for x in a b; do ...; done` in an `&&` chain is torn at
   `; do`. `split_semicolons_marked` (main.rs:310) keeps a for/while/if
   whole only when the *line* starts with `for ` / `if ` (INT-285). A
   chain that starts with `git commit && for sha in ...; do ...; done`
   is split into `for sha in ...`, `do ...`, and `done`, then printed
   as separate steps of "○ N commands" (main.rs:1474). `for` still
   reaches `try_shell_construct` (engine.rs:1752), which hands a whole
   for to `sh -c` -- so the incomplete fragment is what produced
   "syntax error: unexpected end of file from `for'".

5. `paths::tests::every_temp_dir_path_carries_the_process_id`
   (paths.rs:1228) is red on main. It looks two lines past each
   `temp_dir()` (take(2) at 1288). It flags intent/mod.rs:4006
   `core-recent-no-such-ledger` (a planted-absent path, no process
   id, same class as the existing ALLOWED `int267-no-such-directory`),
   and two sites that already carry `process::id()` one line past that
   window: nsh-test/src/main.rs:5223 and
   zero-doctor/src/probes/hardening.rs:523, after rustfmt wrapped
   `format!`.

6. Notes only, no gate: `cargo test -p core` does not refresh
   `target/debug/core`, so a later hook or hand run can use a stale
   binary; and core refuses to start without an existing state.db
   (zero-core/src/state_db.rs:3, INT-247: open never creates the
   file). Rehearsals use `ZERO_STATE_DB` and a db nsh already created.

## The Solution
- `intent_git_log` passes `--diff-merges=first-parent` so a merge in
  the window lists the incoming paths and not the other parent's.
  The `--since` window opens at the earlier of the frontmatter `date:`
  and the author date of the first commit that added
  `zero/intents/open/{id}-*.md`, still floored by a previous
  generation of the same id (F-0050 / INT-267).
- pre-push skips `refs/notes/*` and does not run seals check on them.
- zero-core gains a small `refuse` helper (no `colored` crate; color
  only when stderr is a tty and NO_COLOR is unset): first line
  "Stopped safely — nothing changed" in green, or "Stopped partway —
  something changed" in red, then the reason, the list of what
  changed (partway only), and the one command to finish. cicomplete,
  write_versions, fingerprint record and doctor check_one start with
  it. An UNDETERMINED close after a successful complete stays as
  today, not recast as a refusal.
- The splitter treats `for`/`while`/`until`/`if` and `( ... )` as
  atomic regions anywhere in the line. A mid-chain for stays one
  segment and still goes to `sh` through `try_shell_construct`. A
  grouping subshell runs the inner list in the engine against a saved
  cwd and env and restores both, including on failure; `exit` in the
  child does not exit the parent.
- The temp-dir test reads to the end of the statement. The planted
  `core-recent-no-such-ledger` path is ALLOWED with the same reason
  as `int267-no-such-directory`. The two rustfmt-wrapped sites are
  left alone.

## Success Criteria
- [ ] G1 cicomplete sees merged work (intent/mod.rs). In a temp git
      repo, an intent dated 2026-11-18, a 2026-11-17 commit with
      Intent: INT-N touching zero/tools/zero-core, and a --no-ff merge
      on 2026-11-18 with the same trailer, touched_crates includes
      zero-core, not none. A merge-only window lists the incoming
      (first-parent) paths and not the other parent's files.
      ask: a --no-ff merge of earlier Intent commits still offers those crates
      ruling:
      evidence:
- [ ] G2 pre-push does not scan notes ( .githooks/pre-push ). A push of
      a branch plus refs/notes/seals prints the Intent commits intact
      and does not also print "no commit in the range carries Intent:".
      ask: pushing main and refs/notes/seals does not claim the range has no Intent
      ruling:
      evidence:
- [ ] G3 refusals say safe or partway (zero-core refuse, then the core
      call sites). A safe stop's first line is "Stopped safely — nothing
      changed" (green on a tty, plain under NO_COLOR or a non-tty). A
      partway stop's first line is "Stopped partway — something changed"
      (red), names what changed, then the reason and the one command to
      finish. cicomplete's bump refusal and write_versions' mid-bump
      failure use this helper.
      ask: first line is green-safe or red-partway; NO_COLOR is plain
      ruling:
      evidence:
- [ ] G4 a mid-chain for loop stays whole (main.rs splitter).
      `true && for x in a b; do echo $x; done && echo z` is three
      segments, the for construct is one of them, both echoes run, and
      there is no syntax error. A line that starts with for still
      stays whole, as INT-285 already required.
      ask: for in an && chain runs, and is not split at "; do"
      ruling:
      evidence:
- [ ] G5 a grouping subshell is isolated (engine.rs).
      `(cd DIR && export FOO=1 && pwd)` runs, prints DIR, and leaves
      the parent's cwd and FOO unchanged. A leading paren is not a
      command name. `exit` inside the group does not exit the parent
      shell.
      ask: ( cd DIR && export FOO=1 && pwd ) does not leak cwd or FOO
      ruling:
      evidence:
- [ ] G6 the temp-dir process-id check passes (paths.rs).
      every_temp_dir_path_carries_the_process_id is green on main.
      The two rustfmt-wrapped sites still count as carrying the id.
      core-recent-no-such-ledger is an allowed planted-absent path.
      ask: cargo test -p zero-core every_temp_dir_path_carries_the_process_id passes
      ruling:
      evidence:
- [ ] G7 doors, nsh-test and d green on the laptop. A scratch-HOME
      rehearsal of a throwaway intent whose work arrived through a
      --no-ff merge offers the touched crates to cicomplete --bump.
      A safe cicomplete refusal starts with the green safe line. A
      mid-chain for loop runs.
      ask: doors, nsh-test and d green; the three hand checks pass
      ruling:
      evidence:

## Job order
- Wave 1, side by side: red tests for G1 (intent/mod.rs), G2
  (.githooks/test-pre-push.sh), G3 (new zero-core refuse.rs), G4
  (main.rs splitter tests), G5 (nsh-test or engine tests). G6 has
  no new red test; the existing one is already red on main.
- Wave 2, side by side after their reds: G1 fix in intent/mod.rs,
  G2 fix in pre-push, G3 helper body in refuse.rs, G4 splitter in
  main.rs, G6 window and ALLOWED in paths.rs.
- Wave 3, after the files they need: G3 call sites in cicomplete /
  write_versions / fingerprint / doctor (after the helper and after
  the G1 fix, because cicomplete shares intent/mod.rs with G1); G5
  subshell execution in engine.rs (after the splitter, because a torn
  `( ... )` never reaches the engine).
- G7 on the laptop.

## Limits
This section is a trial of a format Christian asked for: an intent
states its limits up front.

- In scope: the six items in The Problem, at the depth in The
  Solution. Mid-chain while/until/if ride with the for-loop splitter
  fix (same INT-285 rule). Grouping `( ... )` is one level, isolated
  cwd and env, `exit` stays in the child.
- Out of scope: applying INT-320's deferred version numbers (main is
  still core 5.1.2, zero-core 1.2.0, nsh 6.0.1; this close bumps from
  those for 328's own changes). A spine AST For / Subshell variant.
  Brace grouping `{ ... }` (shares cwd and env). `( ) &`, nested
  subshells beyond one recursive level, process substitution (INT-238).
  Recoloring every tool and hook (commit-msg, sandbox, zero-doctor
  definition load). Recasting an UNDETERMINED close as a refusal.
  Creating state.db from core, or making `cargo test -p core` refresh
  `target/debug/core` (notes only). Sweeping finished files out of
  open/ (INT-321). Native for-loop execution inside nsh (sh remains
  the runner, once the construct stays whole).
- How deep: live path only (intent_git_log / intent_log_args,
  pre-push, a new zero-core helper and its named call sites, the
  main.rs splitter, engine subshell, the paths.rs test). No parser
  rewrite, no new crate dependency on zero-core.
- Budget: 12 jobs, about 9 files (intent/mod.rs, pre-push and its
  test, refuse.rs and lib.rs, main.rs, engine.rs, maybe error.rs,
  paths.rs, fingerprint/mod.rs, doctor/mod.rs, maybe nsh-test). One
  production file per job. A job that would need a second production
  file, or more than about 5 minutes, stops and files a follow-up.
- Must not change: the nsh prompt path, live state.db, any file
  outside the repo, anything requiring root or sudo, the
  Core-must-not-depend-on-Intelligence rule, F-0049's anchored grep,
  F-0050's floor against a previous generation of the same id.
- When to stop: if a correct mid-chain for or `( ... )` needs a new
  parser rather than an atomic region in the existing splitter, land
  what already passes and file a follow-up. If `--diff-merges=first-parent`
  is missing on a target git, stop and file a follow-up rather than
  inventing a second path walker. If a refusal site is not in the
  named list, leave it and file it.

## Out of scope
See Limits. The older batches (INT-317, 318, drafted 322) and
INT-321's sweep stay out.

## Versions
- zero-core minor (adds the refuse helper)
- core patch (cicomplete sees merged work; refusals use the helper)
- nsh minor (grouping subshell; mid-chain for stays whole)
