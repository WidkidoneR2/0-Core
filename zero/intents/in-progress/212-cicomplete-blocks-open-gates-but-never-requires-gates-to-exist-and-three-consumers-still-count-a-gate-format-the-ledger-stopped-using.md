---
id: 212
date: 2026-08-09
type: future
title: "cicomplete blocks open gates but never requires gates to exist, and three consumers still count a gate format the ledger stopped using"
status: in-progress
tags: [ledger, intent, gates, enforcement]
depends_on: [294]
---

## Vision
"Complete" means the gates were demonstrated. An intent with no gates cannot reach it, and a
checker that cannot find the file refuses rather than shrugs. One definition of a gate, one owner
of the count, so the ledger cannot disagree with itself about its own central discipline.

## The Problem
The enforcement exists and is half-blind. INT-332 blocks cicomplete when open gates are present --
it reads the markdown format INT-130 established, lists each offender, prints the deferral syntax,
and it works. But it never asks whether a gate exists at all, so a file with zero gate lines walks
straight through. That is how 123 intents came to be marked done without their work being done, and
the empty stubs that remain (145, 010, 013, 014) are the same door still standing open.

Two smaller failures sit beside it. The check is wrapped in `if let Some(ref path)`, so a file the
search cannot locate produces no check rather than a refusal -- and the search uses a hand-written
folder list naming `planned` and `deferred`, neither of which exists in this tree. That is a FOURTH
divergent folder list in the ledger tooling, which is precisely the disease INT-135 gate 7 was
filed to end.

And "gate" has two meanings inside this one file. complete_intent counts markdown; health,
predict_completion and story count the `⬜`/`✅` emoji that INT-130 superseded. The cost is not
cosmetic: a correctly gated intent yields total_gates = 0, so health scores it at zero percent.
The intents following the current convention look like the worst ones in the ledger.

## The Solution
Extract the rule before changing it. The gate check is an inline closure inside complete_intent
today, which is why it has never been tested; lifting it into one pure function over file content
makes G1's red-first demonstration cheap and every later change checkable. Then the three additions
are small: require that gates exist, treat a missing file as a refusal, and derive the folder set
from the same source the rest of the ledger uses rather than a fourth list.

The counting consumers migrate to one owner afterwards, not before -- the enforcer is the site with
teeth, and fixing the reporting first would leave the hole open while making the dashboard prettier.

## Evidence (measured 2026-08-09, not asserted)
- 123 intents were marked done that were not -- the audit that motivates this.
- The check EXISTS: complete_intent, intent/mod.rs:999, INT-332. It reads markdown `- [ ]` / `- [~]`
  and legacy `⬜`, refuses, lists each open gate, prints the deferral syntax. It works.
- THE HOLE: it never requires a gate to EXIST. An intent with zero gate lines passes trivially.
  INT-145 is an empty 11-line template; 010/013/014 are 0-gate stubs; 179's gate was a bare
  placeholder rewritten at completion.
- SECOND HOLE: the whole check sits inside `if let Some(ref path)`. No file found means no check --
  completion proceeds silently.
- THE FOLDER LIST IS PHANTOM: line 1002 searches ["future", "planned", "deferred", "in-progress"].
  `planned` and `deferred` do not exist in this tree. A FOURTH divergent folder list, which is the
  disease INT-135 gate 7 was filed to end.
- TWO DEFINITIONS OF "GATE" IN ONE FILE: the enforcer counts markdown; health (2064),
  predict_completion (2188 and 2197) and story (2445) count `⬜`/`✅` emoji. INT-130 established
  markdown is the real format.
- CONSEQUENCE, not just drift: a fully markdown-gated intent yields total_gates = 0, so health
  computes gate_pct = 0. Correctly gated intents score as the worst ones.

## Non-goals
- Rewriting the deferral mechanism. `⏸ ... approved by: christian <date>` works and stays.
- Removing `override_intent`. A deliberate, recorded escape hatch is not the defect.
- Retroactively gating record dirs (decisions/incidents/philosophy/experiments). They are not
  lifecycle intents and do not carry gates.

## Re-measured when INT-212 started (2026-10-22)

The line numbers in the Evidence above date from 2026-08-09 and are stale. Measured at commit 25fc081d in zero/engine/src/domains/intent/mod.rs:

- Enforcer: complete_intent at 2521; `if let Some(ref path)` at 2542; deferral check 2569-2593; open-gate scan 2594-2606. The file finder is written out at 2544/2553 and again at 2650/2664 in that one function.
- Folder lists, hand-written: complete_intent 2544 and 2650 (future, planned, deferred, in-progress); cancel_intent 4437 (in-progress, future, planned, deferred); defer_intent 4530 and override_intent 4577 (in-progress, future, complete); validate_issues LIFECYCLE 448 and FOLDERS 458; next_id LIFECYCLE 2808. Neither LIFECYCLE copy names planned/, which holds one intent (249) that next_id never scans.
- Finders by id: `starts_with(id)` at 1024, 2553, 2664, 3556, 3666, 3785; 3709 and 3860 test the prefix in another shape. The loose test has no `-` boundary: 22 of the 99 short ids (1 to 99) also match another intent's file by prefix, measured against future, planned, in-progress, complete and cancelled. A measured collision, not yet a demonstrated wrong completion; G5 demonstrates it.
- Two more holes, read from the code and not yet run. The open-gate scan exempts any line that contains `⏸` (2604) while the signature check only inspects lines that START with it (2573), so an open gate with a bare `⏸` appended is exempt unsigned. Both refusal paths, bad deferral at 2592 and open gates at 2628, `return Ok(())`, while the version refusal at 2638 returns an error: a refused cicomplete can read as success by exit status.
- Emoji counting is at three sites, 3464, 3584 and 3837, each `content.matches("⬜") + content.matches("✅")`, not the four named above.
- The intents base path has three owners: intents_dir(ctx) at mod.rs:45, zero_core::paths::intents_dir() at paths.rs:431, and ctx.fpath("intents").

## Gate format (ruled by INT-294 R1, 2026-10-22)

The section is `## Success Criteria`. Each item is one gate: `- [ ] G<n> text`, followed by an INT-158 evidence comment. `[x]` is done, and that includes a gate closed by declining it with numbered reasons. `[~]` is retired for new intents and still reads as open. A deferral keeps its existing form. The emoji form is legacy and is read only until its consumers migrate. Why this name: the scaffold at mod.rs:3020 already writes it, decision 142 names it as file content, and the tooling needs each item to be one independently checkable condition with evidence.

## START HERE -- written 2026-10-22 after the G5 code commit

State: G1 to G4 are ticked with evidence, committed and sealed. G5 CODE is committed as dc65a6da (Seal 26924d7479bb9bbf) but G5 is NOT ticked yet. G6 to G11 are open. Nothing from this stretch has been pushed. INT-280's lifecycle move (deleted in future/, untracked in in-progress/) is uncommitted and is not ours: leave it alone and never stage it.

What G5 changed: find_intent_file in intent/finder.rs, nine tests. The id is a NUMBER: the digits before the first dash of a .md file name, compared numerically, so 21, 021 and 0021 are one id and none matches 212. Folders are searched in the order given, first folder wins, None means missing. start, complete, cancel, defer and override use it. complete finds the file once, reads its gates and moves that same file; a missing file refuses with exit 1 and moves nothing. Narrative (252-266) and predict_completion (596, 659) still carry their own searches.

Do next, in this order:
1. Tick G5 with one Plan on this file (model it on the G4 tick: Plan, dry run, send the real Seal, apply, doors, commit as its own block, no trailers beyond Intent and Seal). The tick must also AMEND the G5 wording: it says exact match with a dash boundary and 22 of 99 short ids; the code does a numeric match. Evidence comment to cite: red run ~/.cache/zero/int212-g5-red.txt (sha256 prefix abda939ae979, 6 of 9 failed); demo before ~/.cache/zero/int212-g5-demo-before.txt (301 skipped the gate check and went on to the registry stage); demo after ~/.cache/zero/int212-g5-demo-after.txt (sha256 prefix a165dbfde18b, 301 refuses, exit 1, nothing moved); commit dc65a6da (Seal 26924d7479bb9bbf); doors: cargo test -p core 126 passed, nsh-test 258 / 258, d 0 failed, core intent validate 334 valid. State plainly that a short-id collision is NOT reachable through complete, because load_all matches the id exactly (complete 21 says Intent 21 not found); the collision is proven in the unit tests and against the real tree only.
2. Optional separate commit: move narrative and predict_completion onto the finder (predict's scope is future only, by design).
3. G6 to G11: G6/G7 health, predict_completion and story consume read_gates (then drop the allow(dead_code) on Gate.id and Gate.evidence); G8 list the 28 grandfathered complete intents without checkbox gates; G9 unmet-gates query; G10 predict_completion take(10); G11 evidence per INT-158 on every gate.
4. Findings to record with core intent find: core intent validate exits 0 even with issues; other crates keep their own folder lists (audit, integrity, zero-git, novashell, intent_tui, core_integration); nsh cannot parse a brace group containing if/fi; zero-gate rustfmt re-stages whole working files and defeats partial staging; cistart moves a file only from future/ so a planned/ intent is not moved (read in code, not demonstrated); validate says 334 now and said 335 at the G4 tick while no committed intent file changed (cause not measured), and d counts differently again.
5. At the very end: push once (git push, then git push origin refs/notes/seals), one ship only if code changed, test the deployed nsh in a new shell, cicomplete 212, clean the cached working .py scripts in ~/.cache/zero. F-0052, F-0053 and the 232 G2 date-versus-git measurement stay open.

Method that works here: payloads go as the pieces transport (each piece checks itself); expected non-zero commands run through python; zero-gate rustfmt-formats staged .rs files, so Plans must produce rustfmt-clean code; no Co-Authored-By or Claude-Session trailers.

## Success Criteria

- [x] G1 RED FIRST: a zero-gate intent completes TODAY, and a stray `- [ ]` outside this section is refused today only by being counted as an open gate, both demonstrated against the extracted checker before any change. Decided: a stray checkbox stays a refusal as its own named reason, never ignored and never reported as a gate. Not by running cicomplete on a real intent
<!-- evidence: 2026-10-22. Red first, run against the faithful copy of the old rule in zero/engine/src/domains/intent/gates.rs before any change: 10 of 18 new tests failed, including zero_gates_refuse, a_missing_section_refuses and an_open_box_outside_the_section_is_named_apart (full run saved at ~/.cache/zero/int212-g1-red.txt, sha256 prefix eb9a9693403d). The decision recorded in this gate: a stray checkbox outside the section stays a refusal as its own named reason, never ignored, never reported as a gate. Nothing was run against a real intent. -->
- [x] G2 ONE pure function over file content owns the gate rule: it reads only the `## Success Criteria` section and returns each gate with its id, its state (open, done, deferred) and whether evidence follows. A gate is deferred only by the full signed form on its own line; a bare `⏸` on an open gate does not exempt it, shown red first against 2604. Unit-tested with no filesystem
<!-- evidence: 2026-10-22. gates::read_gates and gates::check_completion are pure functions over file content in gates.rs, with no filesystem; read_gates returns the id, the state (open, done, deferred) and whether evidence follows for each gate. Red first: a_bare_pause_mark_does_not_exempt_an_open_gate and states_ids_and_evidence_are_read failed against the old rule (same run, eb9a9693403d). Green: cargo test -p core gates passed 22 with 0 failed (~/.cache/zero/int212-g2-green.txt, sha256 prefix 01ef9f13bbed), and the whole crate suite passed 107. -->
- [x] G3 cicomplete REFUSES an intent with no gates, naming that reason distinctly from open gates, and every refusal exits non-zero: the two refusal paths return Ok(()) today (2592, 2628), shown red first against the exit status
<!-- evidence: 2026-10-22. Red first, the deployed old core run against stand-in ledgers (temp HOME, the real INT-212 frontmatter): an open gate printed BLOCKED and exited 0, an unsigned deferral printed the format error and exited 0, and an intent with no Success Criteria section passed the gate check and went on to the version stage (~/.cache/zero/int212-g3-red.txt, sha256 prefix e5e3dd3e54fb). Green, the debug build of this tree: no section, empty section, unsigned deferral, open gate and stray checkbox each exit 1 with their own headline and a refused line on stderr (~/.cache/zero/int212-g3-demo.txt, sha256 prefix f8206868d6c3). Its last line reads WRONG because that script judged the all-done case by any refused line, and a later registry refusal in the stand-in matched; the gate check itself let that case through. -->
- [x] G4 ONE folder source: one module-level LIFECYCLE set (future, planned, in-progress, complete, cancelled) with named subsets for active work and for searching, replacing the hand-written lists (the gate was written for eight; the work found more, each named in the evidence below), so next_id scans planned/. A test reads the real intents/ tree and fails red when a directory is in neither the lifecycle set nor the record set, so a new folder cannot be missed silently again
<!-- evidence: 2026-10-22. Red first: folders.rs written as a skeleton with empty sets, nine tests run against it, six failed, among them every_directory_in_the_real_tree_is_classified (the real intents/ tree against the empty sets), the_numberer_counts_planned and the_lifecycle_shares_one_counter (~/.cache/zero/int212-g4-red.txt, sha256 prefix c9a2c3bc2ace). Green: the real sets, ten folders tests and 117 in the crate all pass, no warnings (~/.cache/zero/int212-g4-green.txt, sha256 prefix 432e19957db1). next_id is folders::next_number, which counts planned/. load_all, validate_issues, start, complete, cancel, defer, override and next_id read the sets; the status-agrees-with-directory check in validate_issues maps folders to statuses and stays. A second inline list, the default list filter, was found by the post-green search: before the fix INT-249 (in planned/) was absent from the default core intent list (~/.cache/zero/int212-list-before.txt, sha256 prefix dd36dc1b3c7c), after it is present as planned (~/.cache/zero/int212-list-after.txt, sha256 prefix 7b082316a2a9). Making the folder names one source made validate flag INT-249, whose status line said in-progress inside planned/; the line was corrected. Commits: 83bba294 (Seal 890640aa9879221a), bf2d3702 (Seal 18875bc6dd411f93), 535088d0 (Seal 0fffe5dd41a8a217). The first attempt at the folders commit also carried the list-filter edit, because zero-gate's rustfmt step re-stages whole working files; it never left this machine and was redone as the three commits above. Doors on the final tree: nsh-test 258 / 258, d 0 failed, core intent validate 335 valid. Not changed here: other crates keep their own folder lists (audit, integrity, zero-git, novashell, intent_tui, core_integration); they are recorded as a finding. -->
- [x] G5 ONE finder, find_intent_file, locates an intent by id in the lifecycle folders and replaces the copies named above. The id is a NUMBER: the digits before the first dash of a .md file name, compared numerically, so 21, 021 and 0021 are one id and none matches 212. Shown red first against the 22 of 99 short ids that match another intent's file by prefix today. A missing file is a REFUSAL, not a skip: completion never proceeds because the check could not find what it checks
<!-- evidence: 2026-10-22. Red first: nine finder tests run before the fix, 6 of 9 failed (~/.cache/zero/int212-g5-red.txt, sha256 prefix abda939ae979). Demo before, the deployed old core: INT-301, a stand-in with no file for the gate check, skipped the check and went on to the registry stage (~/.cache/zero/int212-g5-demo-before.txt, sha256 prefix de9231f440b2). Demo after, the debug build of this tree: 301 refuses, exit 1, nothing moved (~/.cache/zero/int212-g5-demo-after.txt, sha256 prefix a165dbfde18b). Code: find_intent_file in intent/finder.rs, commit dc65a6da (Seal 26924d7479bb9bbf). start, complete, cancel, defer and override use it; complete finds the file once, reads its gates and moves that same file. Doors: cargo test -p core 126 passed, nsh-test 258 / 258, d 0 failed, core intent validate 334 valid. A short-id collision is NOT reachable through complete, because load_all matches the id exactly (complete 21 says Intent 21 not found); the collision is proven in the unit tests and against the real tree only. Not migrated here: narrative (252-266) and predict_completion (596, 659) still carry their own searches. -->
- [x] G6 ONE gate-counting owner. health, predict_completion and story consume the G2 function instead of counting emoji; the three sites are 3464, 3584 and 3837 (measured at 25fc081d)
<!-- evidence: 2026-10-22. Red first: gates::count written as a skeleton that counts nothing, six count tests run against it, 5 of 6 failed (~/.cache/zero/int212-g6-red.txt, sha256 prefix a76ae563a412, plan Seal 87b1e8215975ec24, never committed). Green: gates::count is the one owner. It reads the Success Criteria section through read_gates and falls back to the legacy emoji marks only for a file with no gates there, which the health row labels legacy; 132 tests passed, no warnings, rustfmt clean (~/.cache/zero/int212-g6-green.txt, sha256 prefix cdf62b988caf). The four counting sites in intent/mod.rs (health, predict_completion twice, story) call it, and no emoji .matches call is left in mod.rs. Committed as 42a6301f, Seal 8fe4cd54628ffdf0, with the plan text stored byte for byte as the seal note. Doors: nsh-test 258/258, d 0 failed, core intent validate exit 0 on the debug build. -->
- [x] G7 health reports a REAL gate count for a markdown-gated intent, shown before and after on the same intent (it reads 0/0 today)
<!-- evidence: 2026-10-22. Before and after on the same intents in a stand-in ledger (temp HOME holding a copy of this file and two small markdown-gated intents; ~/.cache/zero/int212-g7-demo.txt, sha256 prefix 50c73f304588). The deployed old core health: this intent read 3/7, counted from emoji marks in its prose and not from its gates; a markdown-only intent in future/ read 0/0; the same intent in in-progress/ was not listed at all. The debug build of this tree: this intent reads 5/11, the future/ intent 2/3, the in-progress/ intent 2/3. On the real ledger the deployed core health printed All active intents healthy while this intent was in progress (~/.cache/zero/int212-g7-before.txt, sha256 prefix 48a662b0242b), and story and predict_completion printed Intent 212 not found; the debug build prints this intent at 5/11 gates, 45 percent, in all three (~/.cache/zero/int212-g7-after-health.txt, sha256 prefix 6c3167bae671). Red first: 3 of 6 finder tests failed against stubs (~/.cache/zero/int212-g7-red.txt, sha256 prefix 1dead32cc80b, plan Seal ff5b909cd5932f2b, never committed). Green: 138 tests passed, no warnings, cargo fmt check clean (~/.cache/zero/int212-g7-green.txt, sha256 prefix f802b1ad2fa2). Committed as 7d8d5dc6, Seal 25d9c09a8795ee63. health now asks git from the file own folder, so a stalled intent can be flagged. Doors: nsh-test 258/258, d 0 failed, core intent validate exit 0. -->
- [x] G8 grandfathering is STATED: 84 complete intents have no gate under the Success Criteria rule today (measured 2026-10-22 with core intent gates --none: 17 have no checkbox line at all, 67 keep only ticked boxes under older Gates headings). The rule applies at cicomplete only, forward-only per INT-158; the 84 are listed by core intent gates --none, not rewritten
<!-- evidence: 2026-10-22. Measured with the tool built for G9, core intent gates --none, debug build of 79e7461c: 84 complete intents have no gate under the Success Criteria rule. Counted from the files themselves, 17 of the 84 hold no [ ] or [x] checkbox line at all and 67 keep ticked boxes only, the first of them under a Gates heading in 44 files and a Gate heading in 6. The 28 first written in this gate was an upper bound from a different measure and is corrected in the gate text, not kept. Grandfathering is STATED here: the rule applies at cicomplete only and forward-only per INT-158; no old intent is rewritten. The list is not copied into any file because a copy goes stale: core intent gates --none prints it, always current, and core intent gates <id> shows any one of them. The only enforcement is G3, cicomplete refusing the intent being completed, never the 84. -->
- [x] G9 a tool answers does this intent have unmet gates, for any intent, and an intent in complete/ with an unticked gate is reportable (196 is one today). Absorbed from INT-232 G3 and G4
<!-- evidence: 2026-10-22. gates::standing (dc3798d8, Seal 710f9f180a49a180), core intent gates (79e7461c, Seal e415f110a8cd2ddb) and the stray-box report (9ab836aa, Seal 4908e63863e5b947), each red first: standing 5 of 6 tests failed against a stub (~/.cache/zero/int212-g9-red.txt, sha256 prefix 11004ea5a777), the command 6 of 6 (int212-g9-gates-red.txt, 53605f0f2bf6), the stray report 2 of 8 (int212-g9-stray-red.txt, 95da39876c07); green each time, 152 tests at the end, no warnings, doors clean. Demonstrated on the real ledger (int212-g9-stray-demo.txt, 1c8f636c21b1): core intent gates 196 reads 6/6 done with 1 open outside the section, the M8 deferral under a second gates heading, and core intent gates --unmet lists complete/196 among 51 intents; before the stray report it listed 41 and not 196. core intent gates 212 prints the standing of this intent and each gate. -->
- [ ] G10 predict_completion take(10) is recorded or fixed: it takes ten files in readdir order, not the ten most recent
- [ ] G11 each gate carries evidence per INT-158

<!-- INT-158 -- EVIDENCE CONVENTION. A ticked box is a promise. Evidence is the receipt.
When you tick a gate, put the proof in an HTML comment on the line after it: a commit
hash, a file:line, a log or artifact path, or "demonstrated: what + how". Prose counts.
FORWARD-ONLY (never retrofit old intents -- busywork, no payoff).
SOFT (a discipline, not gate-police; nothing enforces this).
LIGHT (trivial self-evident gates need no artifact).
Exemplars: INT-133 (the original), INT-161, INT-112, INT-061.
See docs/CONVENTIONS.md. Delete this comment when the intent is written. -->
