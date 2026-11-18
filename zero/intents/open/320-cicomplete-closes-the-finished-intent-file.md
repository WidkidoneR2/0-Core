---
id: 320
date: 2026-11-18
type: future
title: "cicomplete removes the finished intent file, and nothing forgets it"
status: in-progress
tags: [ledger, cicomplete, intents]
format: 2
---

## Vision
A finished intent leaves open/. cicomplete takes the file out in the same
release commit that carries the version bump, so it is gone on the laptop and
on GitHub at once. Nothing that counts, numbers or traces intents loses track
of it: the done count stays the same, a number is never handed out twice, and
`core intent trace INT-NNN` still answers. The full text stays in git.

## The Problem
cicomplete sets `status: complete` and leaves the file in open/ ("The file
stays in open/"). Ten finished intents sit there today (308, 309, 310, 313
to 319). INT-313 already decided that complete and cancelled intents are not
files, but nothing removes them, and three readers would break if they were
removed by hand:

1. `zero_core::ledger::completed_ids()` (zero-core/src/ledger.rs:45) counts
   live files with status complete plus ids git saw under
   `zero/intents/complete/`. 308 to 319 never lived in complete/, so deleting
   them drops the banner from 242 done to 232, and the same set feeds doctor,
   Friday, strategy, predict, stress, narrative, snapshot and the banner's
   blocked count.
2. `folders::next_number()` (engine/src/domains/intent/folders.rs:29) is one
   plus the highest id on disk in open/. With 308 to 319 gone the highest is
   302, so the next intent would be numbered 303, which is already used by a
   finished intent and by its commits, seals and findings.
3. `trace()` (engine/src/domains/intent/trace.rs:184) refuses an id that
   `load_all` does not see: "INT-NNN names no intent in the ledger". That is
   already true for 277, 292 and every other intent removed earlier.

One test also reads a real ledger file: `a_real_v1_intent_on_disk_still_completes`
(gates.rs:821) opens `open/308-...md`, so removing 308 fails `cargo test -p
core`. The test after it, `a_v1_ticked_gate_without_a_receipt_still_completes`,
has no `#[test]` and never runs.

What already copes: `validate_issues()` resolves a `depends_on` against git
history in every lifecycle folder (mod.rs:724), and zero-deadwood's citation
check reads git history too.

## The Solution
`zero/intents/closed.tsv` records every intent that leaves open/, one tab
separated row each: id, status (complete or cancelled), closed date, the last
commit that touched the file, and the title. It sits at the root of
zero/intents, where neither the loader nor validate reads intent files, and
GitHub shows it as a table. zero-core reads it with no new dependency.

- completed_ids adds the rows whose status is complete.
- next_number takes one plus the highest id across open/, closed.tsv and,
  when git can answer, git history of the lifecycle folders.
- trace and show know a closed id and point at `git show <commit>:<path>`.
- cicomplete, after the gate check, the version prompts and a successful
  write_versions, appends the row, runs `git rm` on the file and stages
  closed.tsv, so both land in the release commit Christian makes. The
  suggested commit message says the file was removed.
- If git cannot do it (not a repo, file untracked, git missing), the
  completion still stands: the file stays in open/ with status complete
  exactly as today, and the output says removal UNDETERMINED with the reason.

## Success Criteria
- [x] G1 the gate tests do not read a real ledger file. The v1 test reads an
      embedded copy of 308, and the orphaned v1 test has its #[test] and
      runs. Watched failing first in a scratch tree with 308 moved away.
      ask: cargo test -p core passes with open/308 absent
      ruling:
      evidence: cargo test -p core passed on int-320 (349); v1 test reads the embedded testdata copy, the orphaned v1 test has #[test] and ran
- [x] G2 closed.tsv is read (zero-core ledger.rs). completed_ids_from() on a
      scratch dir with rows 318 complete and 039 cancelled contains 318 and
      not 039. A missing closed.tsv is an empty index. A malformed row is
      named, not dropped. An unreadable file is reported, not empty.
      ask: a closed complete id counts as done, a cancelled one does not
      ruling:
      evidence: cargo test -p zero-core: complete 318 counted, cancelled 039 did not; missing closed.tsv is empty; malformed row named; unreadable file reported
- [x] G3 a number is never handed out twice (folders.rs). With open/302 on
      disk and 319 in closed.tsv, next_number answers 320. Today it answers
      303.
      ask: next_number answers 320 for open/302 plus closed 319
      ruling:
      evidence: cargo test -p core: open/302 plus closed 319, next_number answered 320
- [x] G4 trace knows a closed intent (trace.rs). An id that is only in
      closed.tsv is traced: its findings print, and the row names the commit
      that holds its last text.
      ask: trace INT-318 answers after 318 leaves open/
      ruling:
      evidence: cargo test -p core: trace of a closed-only id printed the finding and named git show <commit>:<path>
- [x] G5 cicomplete closes the file (intent/mod.rs). In a temp git repo,
      close_file() on a committed open/ file appends one row, removes the
      file and stages both. An untracked file and a non-repo directory answer
      UNDETERMINED and leave the file where it is. Nothing is removed unless
      the gate check and write_versions both succeeded.
      ask: the file is staged for deletion and closed.tsv is staged
      ruling:
      evidence: cargo test -p core: close_file staged the deletion and closed.tsv; untracked and non-repo paths were UNDETERMINED and left the file
- [x] G6 doors, nsh-test and d green on the laptop, and a rehearsal in a
      scratch HOME (a clone of 0-core, ZERO_STATE_DB in /tmp) completes a
      throwaway intent: git status shows the file deleted and closed.tsv
      modified, both staged, and core intent stats shows the same done count
      before and after.
      ask: doors, nsh-test and d green; rehearsal shows D and M staged
      ruling:
      evidence: doors 4/4 (nsh-test included) and d 96% with only the uncommitted 320 file after merge 91c61579; scratch HOME rehearsal with the int-320 build showed D and M staged and trace and show answered 399 from closed.tsv

## Job order
- G1 (gates.rs) and G2 (zero-core/src/ledger.rs) run side by side.
- G3 (folders.rs), G4 (trace.rs) and G5 (intent/mod.rs) run side by side
  after G2 lands, because all three ask the closed reader.
- G6 on the laptop.

## Out of scope
- Removing the ten files already finished. That is INT-321, and it waits for
  this intent.
- core intent cancel. INT-321 G5 gives it the same close.
- Reformatting or renumbering any old intent.
- A full archive of every finished intent in closed.tsv. Ids that git saw
  under complete/ are still read from git.

## Versions
- zero-core minor (adds the closed index)
- core minor (cicomplete removes the file)
- nsh patch (relinks zero-core; banner counts unchanged)
