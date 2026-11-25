---
id: 352
date: 2026-11-24
type: future
title: "zero-deadwood knows every intent ever filed"
status: planned
tags: [ledger, deadwood, citations]
depends_on: []
format: 3
---

## Vision
A citation of INT-NNN in source resolves against every intent
that was ever filed, including rows in closed.tsv, numbers used
before the June 2026 ledger purge, and the reserved placeholders
INT-000 and INT-999. The numberer that hands out the next id
uses that same set, so it cannot reissue a number that already
has a history.

## The Problem
On 2026-11-24, `zero-deadwood` on main at 3d9ad966 reported 9
dangling citation flags: INT-180, 322, 326 (gap), and INT-333,
334, 341, 342, 346, 999 (forward).

The check is `check_dangling_intent_citations` in
zero/tools/zero-deadwood/src/main.rs:474. It builds a filed set
from filenames one level under directories in zero/intents/
(lines 477-490) and, only when root equals ~/0-core, from
`git_ids_in` of zero/intents/{future,planned,in-progress,
complete,cancelled,open} (lines 492-507, and
zero/tools/zero-core/src/ledger.rs:218-262). It does not read
closed.tsv. That file is a sibling of those directories, so the
read_dir walk skips it. After INT-320, cicomplete removes the
open file and leaves only a closed.tsv row. git_ids_in still
sees recent closes whose files once lived under zero/intents/
open/, which is why 332 is not flagged. A checkout where that
git walk does not run, or a future close whose path git cannot
see, would flag a finished intent as unfiled.

The nine flags are not nine never-filed intents. 180, 322, 326,
333, 334, 341, 342 and 346 were completed in April or May 2026
under intents/complete/ (and later intents/archive/ and
intents/arch-era/complete/). Those files were deleted in
c847e8b7 (2026-06-08). They never had a zero/intents/ path, so
git_ids_in cannot see them, and they have no closed.tsv row.
Current comments cite that older work (for example
novashell commands/mod.rs:6241, semantic.rs:1,
zero-git commit.rs:88). INT-180 is the sway-to-Niri complete
(d45d1b59), not a hole that was never filed. INT-999 is a test
fixture (friday/map.rs:624, 654, 726-727 and intent/mod.rs
tests) that must not exist as a ledger file.

The forward/gap split is an artifact of the visible highest id,
which is 332. So 322 is a "gap" and 333 is "forward" for the
same reason: the old complete is invisible.

`folders::next_number` (zero/engine/src/domains/intent/folders.rs:42)
already counts open/ files, every closed.tsv row, and git history
under zero/intents/ only (highest_git_lifecycle_id, lines 88-113,
same path filter). On this tree it would return 333. Christian
ruled that 333-346 must not be reissued, because those ids are
still cited in code. `cistart` does not allocate ids; `core
intent add` and `new_intent` do, through next_number.

F-0052 remains a different defect: the check asks whether a
number exists, not whether the comment means the filed intent.
233, 234 and 235 are filed for later work, so those comments
already pass. This intent does not close F-0052.

## The Solution
One filed-set reader in zero-core, reused by the dangling check
and by next_number. zero-deadwood already depends on zero-core.
Core must never depend on Intelligence. Do not add a dependency
from core to zero-deadwood, or from zero-core to engine or
zero-deadwood. Reuse `read_closed` (ledger.rs:100). Do not use
`completed_ids_from` alone, because that helper drops cancelled
rows (ledger.rs:200-208), and a cancelled intent is still filed.

G1 is a red test only. In a temp tree, closed.tsv has a complete
row and a cancelled row, neither id has an open/ file, and a .rs
file cites both. Today's check reports both as dangling.

G2 teaches the check to read closed.tsv through `read_closed`
and to treat complete and cancelled ids as filed. An unreadable
closed.tsv is reported as a failure to read the ledger, not as
a silent omission, in the same family as lines 509-517. The G1
test then goes green.

G3 adds the pre-purge ids. Preference: a checked-in list next
to closed.tsv (zero/intents/historical-ids.tsv), produced from
git log of intents/ and faelight/intents/, plus a test that
fails if git can name an id from those prefixes that the list
lacks. The check reads the list on every run, including in a
temp fixture and when root is not ~/0-core. A live-only git
walk of the old prefixes would still be blind in those cases,
which is how closed.tsv was missed. After G3, a laptop run of
`zero-deadwood` no longer flags 180, 322, 326, 333, 334, 341,
342, or 346.

G4 reserves INT-999 the same way INT-000 is skipped at
zero-deadwood main.rs:532-533 (no finding). If the comment's
"summary line" for reserved ids does not exist, add one that
names 000 and 999 rather than leaving them silent. Do not file
an intent named 999; tests require that it does not exist.

G5 is in scope because next_number would hand out 333 today.
It must use the same filed set as the check, including
closed.tsv (already) and the historical list from G3. A red
test: closed.tsv highest is 332, the historical list includes
351, next_number for open returns 352, not 333. This intent
file itself is created with an explicit id, not by
`core intent add`, until G5 has landed.

G6 is the laptop proof: `zero-deadwood` reports 0 dangling
citations, or lists each remaining flag with a written reason;
doors and d are green.

## Limits
- In scope: the dangling citation check in zero-deadwood, its
  tests, a historical-id list and its drift test under
  zero/intents/, a zero-core ledger helper that reads
  closed.tsv and that list, and next_number in
  zero/engine/src/domains/intent/folders.rs so it uses that
  helper. INT-000 and INT-999 as reserved ids in the check.
- Out of scope: closing INT-231 or ticking its G5 (judge that
  after G6, in 231). Closing F-0052 (existence is still not
  meaning). Backfilling new rows into closed.tsv. Rewriting or
  deleting the old comments. Filing or cancelling 169, 186,
  191, or 231. A new prefix for a later restart of the
  sequence. Changing --summary totals or making dangling
  citations fail `d` or --strict. Walking or restoring the
  deleted arch-era intent file bodies in the tree.
- How deep: about six jobs, one gate each, a few minutes each.
  At most the production files named in In scope, plus tests
  and the historical-id list. No new crate. No new dependency
  edge except existing zero-deadwood -> zero-core and core ->
  zero-core. No secrets in the list or in tests.
- Stop and ask: a job that needs a production file outside In
  scope; a drift test that reveals a historically used id this
  file does not already name; a proposal to close F-0052 or to
  tick 231 G5 from this intent; a proposal to invent a second
  numberer; or a request to file this work under a number other
  than 352 before the first commit.
- Done means: G1 to G6 ticked with evidence.
- Must not change: existing closed.tsv rows, live state.db, any
  file outside the repo, root, the nsh prompt path, crate
  source outside In scope, F-0052's record, and any secret or
  confidential value.

## Success Criteria
- [ ] G1 red first: closed.tsv rows are flagged today
      (zero-deadwood citation tests). In a temp ledger,
      closed.tsv has a complete id and a cancelled id, neither
      file is in open/, and a .rs file cites both numbers.
      `check_dangling_intent_citations` reports both as
      dangling (the complete one as a gap if a higher id is
      also filed in the fixture). The test is red on the code
      as it stands.
      ask: a closed.tsv row with no open file is reported dangling before the fix
      ruling:
      evidence:
- [ ] G2 the check reads closed.tsv and treats complete and
      cancelled ids as filed. It calls `read_closed`, not a
      second parser, and not `completed_ids_from` alone. An
      unreadable closed.tsv is named as unreadable, not treated
      as empty of ids. The G1 fixture then reports neither id.
      ask: a complete or cancelled closed.tsv row is filed, and the G1 test is green
      ruling:
      evidence:
- [ ] G3 historical ids are filed. The checked-in list plus its
      drift test are in the tree. The dangling check reads that
      list. After this gate, `zero-deadwood` on the laptop no
      longer flags INT-180, 322, 326, 333, 334, 341, 342, or
      346.
      ask: the pre-purge ids are in the filed set, and those eight numbers are not flagged
      ruling:
      evidence:
- [ ] G4 INT-999 is reserved like INT-000. A cite of INT-999
      produces no dangling finding. A cite of a genuinely
      unfiled number still does. No intent file is created for
      999.
      ask: INT-999 is reserved and does not appear as a dangling citation
      ruling:
      evidence:
- [ ] G5 next_number does not hand out a historically used id.
      It uses the same filed set as the check (open/,
      closed.tsv, git ids under zero/intents/, and the
      historical list). A fixture whose closed.tsv highest id
      is 332 and whose historical list includes 351 returns
      352 for category open, not 333.
      ask: core intent add would not offer 333 on this tree after the fix
      ruling:
      evidence:
- [ ] G6 `zero-deadwood` on the laptop reports 0 dangling
      citations, or lists each remaining flag with a reason
      written in the evidence. doors and d are green.
      ask: zero-deadwood is clean or explained, and doors and d are green
      ruling:
      evidence:

## Job order
- Job 1: G1 red tests in zero-deadwood citation_check_tests.
- Job 2: G2, then the G1 tests go green.
- Job 3: the historical list, the drift test, and G3 in the
  check. Laptop `zero-deadwood` is the evidence for the eight
  numbers.
- Job 4: G4 reserve 999.
- Job 5: G5 in folders.rs, after the G3 reader exists, with
  its own red then green test.
- Job 6: G6 on the laptop after the others land.
- File this intent with id 352 by hand. Do not call
  `core intent add` to allocate the number until G5 has
  landed, because add would currently offer 333.
- Each job's first commit is the red tests under this plan's
  Seal. Each fix commit carries Intent: INT-352 and Seal:.

## Versions
- zero-deadwood patch (1.1.0 to 1.1.1). The check already
  ships; this is a blind spot in the filed set, not a new
  tool surface, so patch rather than minor. The crate is a
  registry row (zero/registry/tools.toml, same 1.1.0).
- core patch (5.3.1 to 5.3.2) if G5 touches folders.rs or
  next_number. The numberer already ships; refusing 333 is a
  fix, not a new command.
- zero-core patch (1.3.0 to 1.3.1) if G2 or G3 adds a ledger
  helper in zero/tools/zero-core/src/ledger.rs.
- Close with: cicomplete 352, and `--bump tool=patch` for each
  of those crates this intent actually touched, plus
  `--skip-bumps` for the rest. cicomplete offers a bump only
  for touched registry rows, so drop a `--bump` that was not
  offered rather than naming a crate that was not touched.

## INT-231 and F-0052
This intent does not close INT-231 and does not tick 231 G5.
After G6, 231 can be judged on a fresh `zero-deadwood` run.
F-0052 stays open. A filed historical id makes an old comment
resolve for existence; it does not teach the check to ask
whether the comment means the filed intent. 233, 234 and 235
remain the exhibited case. Do not add a Fixes: F-0052 trailer
to any 352 commit.

## Note on the number 352
Christian ruled that this work is INT-352. Ids 333-346 are cited
in current source for older work. git history of intents/ and
faelight/intents/ also contains 347-351 (347 study of a bar
shell, 348 study of a widget system, 349 NixOS parity, 350 rust
toolchain, 351 NixOS scripts). No number with a history is
reused. G3's list must include 347-351, or the drift test
should fail. After G5 the next id on this tree is 353. Future
work after the current intents moves to a separate P0- series,
which is out of scope here.
