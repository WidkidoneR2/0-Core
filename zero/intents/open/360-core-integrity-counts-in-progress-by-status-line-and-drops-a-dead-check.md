---
id: 360
date: 2026-11-27
type: future
title: "core integrity counts in-progress by status line and drops a dead check"
status: in-progress
tags: [core, integrity, intent]
depends_on: []
format: 3
---

## Vision
Two integrity checks in the Intent group tell the truth
about what they look at. `intent_inprogress_count`
counts an intent as in progress only when its
frontmatter `status:` line says in-progress, not when
the body quotes those words. `intent_status_directory`
does nothing today, so it is removed instead of
sitting in the suite as coverage that does not exist.
Nothing else about integrity changes.

## The Problem
Measured 2026-11-27 on a box checkout of origin/main
cd8a5d04 (core 5.5.0). 393 of 393 `cargo +1.88.0 test
-p core` tests passed. The laptop's tree and state were
not touched. All 35 files in zero/intents/open/ are
`status: planned`, and none of them contains the text
`status: in-progress`.

The report that both checks use body `contains`
matching is half true. Read from code:

1. `intent_inprogress_count` reads the whole file.
   IntentInProgressCountCheck
   (integrity/mod.rs:1230-1274) lists open/ (1240) and
   for each .md file asks
   `content.contains("status: in-progress")` (1249).
   That is a substring test on the entire file, so an
   intent whose frontmatter says planned but whose body
   quotes `status: in-progress` is counted as in
   progress. It raises an alert only when more than 7
   files match (1260), with weight 2 (1269). So the
   wrong answer needs 8 matching files. Today no file
   matches, so it is a latent bug, not a live one. It
   is also the kind of text an intent about intents
   writes: INT-359's own file quotes the phrase.

   Reproduced on a scratch copy of cd8a5d04 (cargo
   test, two throwaway tests, not committed). Eight
   files in open/, each `status: planned` with a body
   line `Body quote: status: in-progress`:

       REPRO count issue: 8 intents marked in-progress
       (expected 7 or fewer): 905-planned.md, 908-...
       REPRO count issues: 1

   Reproduced end to end with the debug binary built
   from that scratch copy, in a scratch HOME under
   /tmp with a stub state.db, command
   `core integrity run`. The same tree with no scratch
   files: "Integrity: 100%" and "No integrity issues
   detected". With the eight planned-but-quoting files
   added: "Integrity: 84%" and "Alerts: 1 issues
   requiring attention". With the eight files made
   really in-progress in the frontmatter: also 84% and
   1 alert, which is the correct answer.

2. `intent_status_directory` is not a body match, and
   it is a no-op. IntentStatusDirectoryCheck
   (1115-1177) already compares the exact frontmatter
   line, through frontmatter_status (577-585, INT-275).
   The bug is different. Its folder list holds one
   entry, `("open", ...)` (1125). The match on the
   folder name at 1127-1130 handles only "complete"
   and "future" and has `_ => continue`, so the "open"
   entry is skipped before any file is read. The only
   alert in the body (the `*dir_type == "complete"`
   test, 1155-1169) needs dir_type "complete", which
   can never be set. `let _ = expected_status;`
   (1170) is the leftover. So the check cannot return
   an issue for any input.

   Reproduced on the same scratch copy: four files in
   open/ with status complete, planned, deferred and
   in-progress:

       REPRO status_directory issues: 0

   How it got that way: INT-313 stage 3 (23c1fce4,
   2026-11-14) removed the move proposals when status
   changed in place. INT-313 stage 4 (f0415b3e,
   2026-11-14) made open/ the only live folder and cut
   the folder list to ("open", ...). Neither commit
   removed the check, so it stays in the suite
   (build_check_suite, 1090) and runs on every scan.
   Its one test, a_status_that_does_not_match_its_folder_
   is_not_a_move (2390-2418), runs the check and
   asserts it files no move, which a no-op passes.

3. The reader this needs already exists in the same
   file. frontmatter_status (577-585) returns the first
   line starting with `status:` within the first twenty
   lines, trimmed, or "". Its test is
   status_is_the_frontmatter_line_not_a_mention
   (2420-2428). The checks module sees it (check 1
   calls it at 1145). Two other status readers exist
   in the intent domain: status_of (intent/finder.rs:67,
   private) and frontmatter_status_value
   (intent/mod.rs:5858, private, added by INT-359 G2).
   Neither can be reached from integrity without
   changing visibility.

4. There is precedent for removing a dead check.
   INT-358 removed docs_count_consistency from the
   suite and tests it is gone
   (int358_docs_count_check_is_gone_and_100_percent_
   is_not_a_count, 2826). A pending row of a removed
   check is retired as "check removed" by the existing
   reconcile (integrity/mod.rs:639-663); nothing new is
   needed for that.

Related, found and left alone: frontmatter_status
accepts a `status:` line anywhere in the first twenty
lines, so a file with no frontmatter status and a
quoted line near the top would still match; it also
compares exactly, so `status: "in-progress"` in quotes
does not match. All 35 open files use the plain form.
The alert text for the count check contains one
non-ASCII character, a less-than-or-equal sign (1265).
A run of `core integrity status` after a clean run
showed "Active alerts: 1" in the scratch HOME; that
was not investigated. None of these is in this intent.

## The Solution
All work is in
zero/engine/src/domains/integrity/mod.rs. No other file
changes, and no new function or helper.

Decision: reuse integrity's own frontmatter_status, do
not reuse the intent-domain readers and do not write a
new one. Why: it is already in this file, tested, and
used by the sibling check. status_of and
frontmatter_status_value are private to the intent
domain, so using either means changing visibility or
adding a cross-domain call, which is a design choice.
Folding the three readers into one is foundation item 4
("one intent reader"). This intent stays minimal and
leaves that to item 4.

Decision: remove IntentStatusDirectoryCheck, do not
revive it. Why: reviving it means deciding what a wrong
status in open/ is (complete? cancelled?), which is a
new rule, not a bug fix. cicomplete and cancel remove
the file from open/ when they finish (INT-320, INT-359
close), so no rule is obvious from the code read here.
If Christian wants such a rule, it is its own intent.
Removal follows INT-358.

G1 adds four red-or-guard tests in the integrity tests
module, next to
status_is_the_frontmatter_line_not_a_mention. They use
stand_in(tag) (2137), ctx.fpath("intents/open"), and
`checks::IntentInProgressCountCheck.run(
&IntegrityContext::new(&ctx))`. No seam is needed; the
check already exists. Tests are named int360_ so one
filter runs them:
`cargo +1.88.0 test -p core int360_`.

- `int360_inprogress_count_ignores_a_body_quote`
  (red). Eight files in open/, each frontmatter
  `status: planned`, each body quoting
  `status: in-progress`. The run returns no issues.
  Today it returns one.
- `int360_inprogress_count_ignores_quotes_beside_real_ones`
  (red). Seven files with frontmatter `status:
  in-progress`, plus three planned files that quote the
  phrase in the body. The run returns no issues (seven
  is allowed). Today it returns one.
- `int360_inprogress_count_alerts_on_eight_real`
  (guard, passes today and must still pass). Eight
  files with frontmatter `status: in-progress`. The run
  returns exactly one issue, its description starts
  with `8 intents marked in-progress`, and its check
  name is `intent_inprogress_count`.
- `int360_inprogress_count_allows_seven_real`
  (guard, passes today). Seven files with frontmatter
  `status: in-progress`. The run returns no issues.

G2 changes IntentInProgressCountCheck only. Replace
the test at 1249 with
`frontmatter_status(&content) == "status: in-progress"`.
Do not change the threshold, the weight, the message,
the directory it lists, or any other check. All four
int360_ tests pass. The full suite is 397 passed.
Proof on the box, in a scratch HOME under /tmp with a
copy of the tree and a stub state.db (the same method
as the repro above): eight planned files that quote
the phrase give `core integrity run` -> "Integrity:
100%"; eight really in-progress files give an alert
and 84%. Paste both outputs.

G3 removes IntentStatusDirectoryCheck: the struct and
its impl (1115-1177), its line in build_check_suite
(1090), and its old test (2390-2418). It adds one test,
`int360_status_directory_check_is_gone`, next to
int358_docs_count_check_is_gone_and_100_percent_is_not_a_count:
the names in build_check_suite() do not contain
`intent_status_directory`, and an open/ file with
status complete makes no suite check report that name.
frontmatter_status stays (the count check uses it).
The full suite is 397 passed (one test removed, one
added).

G4 is the laptop proof.

## Limits
- In scope: integrity/mod.rs only. The `contains` line
  in IntentInProgressCountCheck; removal of
  IntentStatusDirectoryCheck, its suite line and its old
  test; five int360_ tests; a core patch bump at close.
- Out of scope: merging or changing the three status
  readers (foundation item 4); making the readers
  agree on quoted values or on the twenty-line window;
  a new rule for a wrong status in open/; changing the
  count threshold, weight or message (including the
  non-ASCII sign in it); intent_duplicate_id; the
  integrity status "Active alerts" count; the intent
  domain; zero-core path helpers.
- How deep: four gates, a short sitting. One line
  changed in one check, one check and one test removed,
  five tests added. No new crate, dependency, function
  or visibility change. Core does not depend on
  Intelligence and this does not change that.
- Stop and ask: an existing test other than the one
  removed in G3 fails or needs its assertion changed; the
  fix needs a new reader or a visibility change; removing
  the check needs any change outside integrity/mod.rs; a
  case seems to need a change outside In scope.
- Done means: G1 to G4 ticked with evidence.
- Must not change: live state.db and the laptop's tree
  (agents use scratch HOME and scratch repos only),
  the behaviour and printed text of every other check,
  frontmatter_status itself (reuse it, do not edit it),
  the intent domain, closed.tsv, any file outside the
  repo, and any secret or confidential value.

## Success Criteria
- [x] G1 red first: the four int360_ tests are in the
      integrity tests module. The two body-quote tests
      fail on their assertions (none errors), and the
      two guard tests pass. The 393 existing tests stay
      green.
      ask: today a body quote of the phrase can make the count alert, and nothing tests it
      ruling: confirmed on int-360 59ee6f72
      evidence: 59ee6f72 -- four int360_ tests; the two body-quote tests failed on their assertions (8 and 10 intents marked in-progress), none errored; the two guard tests passed; 395 passed, 2 failed
- [x] G2 IntentInProgressCountCheck counts a file only
      when frontmatter_status says `status: in-progress`.
      All four int360_ tests pass and the full suite is
      397 passed. Box scratch proof pasted: planned files
      that quote the phrase give 100%, really
      in-progress files give the alert.
      ask: only a real in-progress status counts, and eight real ones still alert
      ruling: confirmed on int-360 b1608a20
      evidence: b1608a20 -- one line, contains replaced by frontmatter_status equals status: in-progress; four int360_ pass, 397 of 397; box scratch run 100 pct clean, 100 pct with eight planned quoting files, 84 pct and one alert with eight real
- [x] G3 IntentStatusDirectoryCheck, its suite line and
      its old test are removed, and
      int360_status_directory_check_is_gone passes. The
      full suite is 397 passed. Nothing else in the
      suite changes.
      ask: the suite no longer carries a check that cannot report anything
      ruling: confirmed on int-360 660aa3db
      evidence: 660aa3db -- struct, impl, suite line and old test removed; int360_status_directory_check_is_gone added; five int360_ pass, 397 of 397; build has no warning
- [x] G4 laptop: after merge and ship, `core integrity
      run` on a scratch copy of the tree with eight
      planned files that quote the phrase reports 100%
      and no intent issue; with eight really
      in-progress files it alerts; `d` on the real tree
      is green with integrity 100%; the laptop's real
      open intents and state are untouched.
      ask: the shipped binary counts by the status line, and the real tree is untouched
      ruling: confirmed on laptop
      evidence: Block T 5 int360_ passed, 397 of 397, tree clean, doors 4 of 4; merge, 4 seals intact, ship core, d 96 pct with only the cistart status line uncommitted, Integrity 100 pct; scratch clone with eight planned quoting files 100 pct no issues; with eight real in-progress files one alert at 82 pct, not the 84 pct of G2, because G3 removed one check and the score is 100 minus 200 over 2 plus the number of checks

## Job order
Jobs run on branch int-360 only. Agents never push to
main. Job I files the intent. G1 first, alone. Then G2,
then G3 (each builds on the one before; G2 first keeps
frontmatter_status in use when G3 deletes check 1). Each
job pulls with rebase before it pushes. G4 is the
laptop. The seal is the first 16 hex of sha256 of
int360-plan.txt: c2dcba6715e730d0.

## Versions
- core patch (a bug fix and a dead check removed, no
  new user-facing command; core is 5.5.0, so the close
  bumps to 5.5.1). Earlier bug batches closed as patch
  (INT-331: 5.3.1).
