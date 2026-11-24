---
id: 331
date: 2026-11-24
type: future
title: "Bug batch 5: closed dependencies count, list is read-only, version fixtures"
status: planned
tags: [bugs, ledger, core]
depends_on: []
format: 3
---

## Vision
Starting an intent that depends on a finished one does not claim that
the dependency names no intent. `core intent list` and `list --json`
run with read access only. The registry test that forbids a version
literal in crate source is green on main, and the nsh-test fixtures
still prove the public index does not embed a release version.

## The Problem
Finding numbers are placeholders until the three findings are filed:
F-0119, F-0120 and F-0121. Swap them for the real ids before work
starts. Read against main 1d56ccb9, core 5.3.0.

1. F-0119. `cistart 330` printed "INT-321 names no intent -- the
   reference is invalid" even though 321 is complete in
   zero/intents/closed.tsv. INT-330's frontmatter was
   `depends_on: [321]`. `start()` (intent/mod.rs:1279) resolves
   deps with `dep_state(&load_all())` (1303-1330). `load_all`
   (188-191) is only `read_open()`. After INT-321, a finished
   intent's file is removed and the row lives in closed.tsv, so
   `dep_state` (92-100) returns Missing. The same false warning
   will fire for any later start whose depends_on names a finished
   intent, including open 288 on 277, 167 on 214, 298 on 293 and
   297 on 292, which are not in the 21-row closed.tsv. The
   reference on 330 was valid; the checker does not read
   closed.tsv or `completed_ids()`.

2. F-0120. `core intent list` requires FilesystemWriteHome even
   for `--json`. `list()` (intent/mod.rs:291-297) requires Read
   and Write, then either builds JSON and prints (298-310) or
   reads `read_open()` and prints. The function does not write.
   `show --json` (show at 457-459) returns `show_json(id)` with
   no write capability check in that function.

3. F-0121. `registry::tests::no_crate_hardcodes_a_version`
   (zero/tools/zero-core/src/registry.rs:651) is red on main.
   The test lives in `-p zero-core`, not `-p core`. A scan that
   matches `version_literals` finds exactly two hits, both
   fixture strings in nsh-test case
   `zero_docs_public_index_states_no_version`
   (zero/shell/nsh-test/src/main.rs:3091): `for ver in ["9.8.7",
   "9.8.8"]`. They write fake zero/meta/VERSION values to prove
   docs/public/index.md does not embed a release version and does
   not change when only the version changes (checks at 3105-3109).
   nsh-test is not under `#[cfg(test)]`, so the scanner flags
   them. They are not crate versions.

## The Solution
- G1. `start()` / `dep_state` consult `read_closed` so an id in
  closed.tsv is Satisfied when status is complete and
  Questionable when status is cancelled, with the existing
  cancelled note, not Missing. They also consult
  `completed_ids()` so an older complete that lives only in the
  git archive is Satisfied. An id in neither open/, closed.tsv
  nor that complete set stays Missing. If a cancelled id is only
  in git and not in closed.tsv, stop and ask rather than invent
  a second walker. Red tests first in a temp ledger.
- G2. `list()` requires only FilesystemReadHome, for both the
  text list and `--json`. Text output of list is unchanged.
- G3. `cargo test -p zero-core no_crate_hardcodes_a_version`
  passes. Keep the nsh-test proof that the public index does not
  contain the fixture versions and does not change when only the
  version changes. Prefer building those strings without a
  single x.y.z literal, or an explicit allowlist of these two
  fixtures by file and value, like the engine-state seeds in
  the same test. Do not skip the nsh-test crate, and do not
  weaken the scanner for real crate versions.

## Limits
- In scope: the three items in The Problem, at the depth in The
  Solution. closed.tsv complete and cancelled rows, and the
  existing `completed_ids()` archive for older completes.
- Out of scope: packages.txt (Christian ruled it out of this
  batch). panel.rs stdio (detached null is intentional; a log
  path is a later design choice). F-0107 (already closed
  without a fix in a1e5e0f5; the record is never deleted).
  F-0103 and F-0108 (still about ghostty). The two open highs
  F-0036 and F-0040 (later nsh batch). The AGENTS.md findings
  sentences drafted in INT-302 (next step after this batch).
- How deep: Job A may touch zero/engine/src/domains/intent/mod.rs
  and its tests, calling existing zero-core ledger readers, no
  new public API unless a reader is missing. Job B may touch
  zero/shell/nsh-test/src/main.rs and/or the allowlist in
  zero/tools/zero-core/src/registry.rs tests. About 5 jobs, one
  production file each. No new crate dependency. core does not
  depend on Intelligence.
- Stop and ask: a job that needs a second production file, a
  cancelled-only git walker, or a list path that actually writes.
  Land what already passes and file a follow-up instead of
  improvising.
- Done means: G1 to G4 ticked with evidence.
- Must not change: closed.tsv contents, live state.db, any file
  outside the repo, root, the nsh prompt path, show(),
  packages.txt, panel.rs, F-0107, format 1 and format 2
  validation for other open intents.

## Success Criteria
- [ ] G1 closed dependencies are found (intent/mod.rs, F-0119).
      In a temp ledger, closed.tsv has a complete id and a
      cancelled id and neither file is in open/. An open intent
      that depends on the complete id is Satisfied, and start
      does not print names no intent. An open intent that
      depends on the cancelled id is Questionable with the
      cancelled note, not Missing. An id in neither open/ nor
      closed.tsv stays Missing. A complete id present only in
      `completed_ids()` is Satisfied.
      ask: a finished dependency in closed.tsv does not read as names no intent
      ruling:
      evidence:
- [ ] G2 list is read-only (intent/mod.rs, F-0120). `core intent
      list` and `list --json` succeed when only
      FilesystemReadHome is granted. Text stdout of list is
      unchanged for a given ledger.
      ask: list and list --json require only read access
      ruling:
      evidence:
- [ ] G3 the version-literal test is green (zero-core registry
      test and nsh-test fixtures, F-0121). `cargo test -p
      zero-core no_crate_hardcodes_a_version` passes. The
      nsh-test case `zero_docs_public_index_states_no_version`
      still fails if the generated index contains the fixture
      versions or changes when only the version changes.
      ask: the scanner is green and the public-index fixtures still prove the rule
      ruling:
      evidence:
- [ ] G4 doors and d green on the laptop.
      ask: doors and d are green
      ruling:
      evidence:

## Job order
- Wave 1, side by side: Job A writes red tests for G1 and G2 in
  intent/mod.rs. Job B writes a red test or a failing
  reproduction for G3 in -p zero-core.
- Wave 2: Job A, after its reds, fixes G1 then G2 in the same
  file, one commit each. Job B, in parallel after its red,
  fixes the fixtures or allowlist.
- G4 on the laptop after both jobs land.
- Each job's first commit is the red tests under this plan's
  Seal. Each fix commit carries Intent: INT-331, Seal:, and
  Fixes: for the finding of that gate.

## Versions
- core patch (5.3.0 to 5.3.1). G1 and G2 are bug fixes in the
  already-shipped list and start commands, not a new surface, so
  patch rather than minor. G3 is a test-only change in nsh-test
  and/or zero-core tests, so no zero-core or nsh-test bump.
