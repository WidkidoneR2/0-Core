---
id: 273
date: 2026-10-03
type: future
title: "cicomplete shows evidence at the question: diffstat, commit subjects, tier legend"
status: complete
tags: [cicomplete, versions, evidence]
---

## Vision
The close asks a human for each crate's level, and the human answers better with the work in front
of them. Today the question shows a crate name and its current version, nothing else. After this
intent each question shows, on stderr above the prompt, what this intent did to that crate: its
commits, the files and lines they changed, and a one-line reminder of what the three digits
promise. The close still proposes nothing and the human still types the digit (INT-102 D4).

## Why Now
- INT-271 (complete 2026-10-03) made the close ask honestly: no default, typos asked again,
  questions on stderr. It left the question bare.
- Christian wants the level judged with the size of the work in view: lines of code and files
  involved. INT-102 D5, Decision 6 and the 2026-08-22 ruling found that size cannot choose the
  digit (INT-271's engine change was +391/-191 and was skipped; Phase 2 changed about 80 lines and
  changed a promise). Shown beside the question, size informs the choice without making it.

## What
1. Above each question, on stderr: the number of this intent's commits that touched the crate and
   their subjects (capped, with the count of any not shown); the files changed under the crate and
   the lines added and removed; one legend line: major breaks a promise, minor adds one, patch
   fixes one or touches nothing promised.
2. A note that only commits whose message names INT-<id> are counted, since that is the join key.
3. The prompt itself is unchanged: no default, no proposed level.

## Approach
- Phase 0 reads decide_versions, ask_level and touched_crates as INT-271 left them, and how
  git log --numstat reports a crate path including renames across CRATE_PARENTS_HISTORY.
- The evidence is computed from git at question time and never stored.
- Red first: the bare question captured on the deployed core before any edit.
- Tested on a scratch HOME (the INT-271 method) with commits of known size, so every number shown
  can be checked against git by hand.
- No Omarchy code, no sudo. AGENTS.md is not edited.

## Phases
Phase 0 -- recon, reading only: the question path with file:line; what git log --grep=INT-<id>
  --numstat returns for a crate across its historical parents.
Phase 1 -- the evidence block above each question.
Phase 2 -- ship; doors.

## Gates
- [x] Phase 0: decide_versions, ask_level and touched_crates recorded with file:line, and the numstat behaviour across a crate's historical parents recorded
<!-- evidence: read 2026-10-04 at 62f9e40b; the lines hold at 2a2a09b5 because 146baa1e replaced lines 1317 and 1525 one for one. intent/mod.rs: touched_crates 1093-1131 runs git log --name-only --format= --grep=INT-<id> --since=60 days ago and matches crates by substring (novashell through CRATE_PARENTS_HISTORY, engine by engine/src or engine/Cargo, the rest by name); TouchedCrate 1079 holds tool and rel only. decide_versions 1205-1258 asks at 1229, only for a crate with no --bump, no --skip-bumps and a terminal. ask_level 1177-1200 prompts on stderr. numstat, measured on the real repo: default rename detection prints a moved file once, in brace form, with only its changed lines, the brace at the front ({faelight => zero}/rust-tools/novashell/Cargo.toml, 7b79c725, 0 0) or in the middle (zero/{rust-tools => tools}/novashell/Cargo.toml, bbce22a1, 0 0; zero/{tools => shell}/novashell/Cargo.toml, 183d2f26, 2 2). --no-renames turns a move into a delete plus an add with the whole file counted (INT-267 close c6418be7: 996 and 996 for a one-line edit). -z writes a rename as added TAB removed TAB NUL old NUL new NUL (2a2a09b5, read with od -c). --grep=INT-272 with no --since also matched six older core-protect commits from an earlier INT-272; the 60-day window excludes them. -->
- [x] Red baseline on deployed core: on a scratch HOME the question shows only the crate and its version, captured verbatim
<!-- evidence: demonstrated 2026-10-04 on deployed core (built 01:29:03) in a terminal, scratch HOME /tmp/int273-home: a base commit, three INT-999 commits (novashell a.rs +10; a.rs +3 -2 with engine e.rs +5; a.rs renamed to b.rs +1 -1) and one commit without the id (novashell z.rs +7). The questions read bump novashell 1.0.0? [patch/minor/major/skip] > and bump engine 2.0.0? [patch/minor/major/skip] > with nothing above either. Both were answered skip, so the scratch close completed inside the scratch tree; the real ledger was not touched. -->
- [x] After, on a scratch HOME with commits of known size: each question is preceded on stderr by the commit count and subjects, the files and added/removed lines for that crate, the legend line and the join-key note, and every number matches git log --numstat run by hand
<!-- evidence: commit 6f419ac3, 2026-10-04. Red first: cargo test -p core int273 failed on every_number_matches_the_scratch_tree with evidence_lines empty, then 3 passed. By hand, git log --grep=INT-999 --numstat on the scratch tree gave a13cd9e 1 1 src/{a.rs => b.rs}; 31ebd31 5 0 engine src/e.rs and 3 2 src/a.rs; f064548 10 0 src/a.rs. Debug core, then deployed core (built 01:49:34), in a terminal: above the novashell question, novashell: 3 commit(s) naming INT-999 with the three subjects, 2 file(s) under novashell, +14 -3, a.rs +13 -2 (10+3, 0+2), b.rs +1 -1 (renamed from a.rs), the tiers line and the counted line. Above the engine question, engine: 1 commit(s), 1 file(s) under engine, +5 -0, e.rs only. The commit without the id (z.rs +7) appeared in neither. Every number matches. -->
- [x] No level is proposed: the prompt reads as INT-271 left it, and the empty-answer and typo checks still ask again under a pty
<!-- evidence: demonstrated 2026-10-04 on deployed core (built 01:49:34), scratch HOME: the prompt reads bump novashell 1.0.0? [patch/minor/major/skip] > with no default and no level in the evidence. Under a pty (printf of an empty line, ptach and skip into script -qec): the empty answer printed '' is not patch, minor, major or skip and asked again; ptach was refused and asked again; skip was taken; at the engine question the closed input was refused (no answer for engine: input closed, nothing was written and the intent was not moved), exit 1, scratch git status empty. -->
- [x] The evidence goes to stderr: with stdout redirected the evidence and questions stay on screen and the file holds status lines only
<!-- evidence: demonstrated 2026-10-04 on deployed core (built 01:49:34), scratch HOME, stdout redirected to /tmp/int273-out.txt: both evidence blocks and both questions stayed on screen; the file held the checkpoint, the Versions lines, Intent Complete, the retrospective and the Friday line, and no evidence or question line. The scratch tree was restored by git checkout afterwards. -->
- [x] Doors per commit: the result on PATH after ship, nsh-test all passing, d 0 failed, plus that commit's own door above
<!-- evidence: 6f419ac3 (core): the first nsh-test run was 213 of 215, because the first test fixtures held literal crate directories (crate_directory_has_one_owner) and a retired name (no_live_retired_name_in_any_tracked_file); the fixtures were rebuilt from zero_core::paths before the commit, test-only code. Then ship 1 shipped 0 failed, nsh-test 215 of 215, d 0 failed, Integrity 100%, the gates 3 to 5 probes as recorded above. The ledger commit ran nsh-test and d first with only the intent staged. -->

## Notes
- Christian raised on 2026-10-03 that the evidence should be based on LOC and files involved. This
  intent shows both; it does not turn them into a proposed level. Making size propose the digit
  would reverse INT-102 D5 and the 2026-08-22 ruling, and needs its own decision record first.
- A later possibility, not in scope: nsh-test cases tagged with their NSH-COMPATIBILITY tier, so the
  close could also show which promises an intent's commits changed.
- zero-core is outside the crates the close tracks, because how to version it is undecided (it is
  used by every tool and directory). This intent does not change that.
- Related: INT-271 (the close this builds on), INT-102 (version architecture).
- The join key, measured 2026-10-04: intent ids have been reused, and --grep=INT-<id> matches every era of an id. --grep=INT-272 matched six older core-protect commits; only the 60-day window in touched_crates keeps them out, so the window is load-bearing. --grep is also a substring match, so a short id such as 27 matches 270 to 279. The evidence uses the same commit set as the detection, so the two cannot disagree; changing the join key is its own decision.
- Found, not fixed (2026-10-04, during INT-272's close): notify::desktop (engine notify/mod.rs:142-169) runs busctl with .status(), so busctl's reply prints as u <id> in tool output (seen as u 8, u 10, u 12). The close does not write zero/registry/tools.toml, so every core bump leaves registry drift (the integrity log shows 4.1.2, 4.1.3 and 4.1.4). Pending integrity proposals are not retired when their drift is gone: #17 would set core back to 4.1.4. The registry entry nsh (3.9.0) has no crate directory of that name, and the drift check skips it with continue, saying nothing.

## The Rule
"Show the work beside the question. The digit stays a judgement, made with the evidence in view."

## Versions
- engine skipped
