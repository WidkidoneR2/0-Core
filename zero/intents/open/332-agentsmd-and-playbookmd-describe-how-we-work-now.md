---
id: 332
date: 2026-11-24
type: future
title: "AGENTS.md and PLAYBOOK.md describe how we work now"
status: planned
tags: [docs, agents]
depends_on: []
format: 3
---

## Vision
AGENTS.md and the orchestration playbook describe how work is done
now: Christian decides and approves everything and runs every
command on his laptop; Eric is the link between Christian and
Cursor and reviews; Cursor cloud agents code on branches and never
push to main; Christian merges and seals. Findings close by a
Fixes: trailer or a closed_without_fix field. No live workflow
doc names Claude or Jarvis as a working agent.

## The Problem
AGENTS.md still names Claude as the author and Jarvis as the
reviewer under "Multi-AI roles". zero/orchestration/PLAYBOOK.md,
written from INT-304, still staffs Jarvis as conductor and Claude
as backup, says a handoff is a file not a paste, and points at
Multi-AI roles. zero/handoffs/README.md still lists claude as a
writer. Christian cancelled Claude as a working agent on
2026-11-22, and on 2026-11-24 described the current triangle and
the copy-and-paste method. INT-302 drafted two AGENTS.md sentences
on how findings close, and those sentences have not been applied;
INT-302 G11 still requires his ruling before AGENTS.md changes,
and G12 assumed the AGENTS.md diff would show only those two
sentences.

## The Solution
Apply the drafted AGENTS.md, PLAYBOOK.md, and handoffs README on
branch int-332, after Christian has seen the diffs. Docs only. The INT-302 sentences land here, so
INT-302 needs a ruling not to re-add them. Historical mentions
stay where they are history. No code and no version bump.

## Limits
- In scope: AGENTS.md, zero/orchestration/PLAYBOOK.md, and
  zero/handoffs/README.md, as drafted and reviewed. The INT-302 AGENTS.md
  sentences. A note on INT-302 so those sentences are not applied
  twice.
- Out of scope: crate source, crate versions, and generated
  indexes. docs/rfc-169-parser-spine.md (historical authorship,
  not a current workflow). Existing handoff files under
  zero/handoffs/2026-11-10-*.md. NovaShell comments that mention
  Claude. Engine code that still uses the name Jarvis for a score
  or gate. The rest of INT-302 (F-0101, F-0108, the NOVASHELL.md
  proposal).
- How deep: three documentation files, applied as drafted. No new
  crate, no new tool, no daemon, and no secrets in the text.
- Stop and ask: a change that needs a fourth file, a code edit, or
  a rewrite of a historical handoff or RFC. Land what was approved
  and file a follow-up instead of improvising.
- Done means: G1 to G4 ticked with evidence, the three files on
  main, and INT-302 ruled not to re-add the two AGENTS.md
  sentences.
- Must not change: any crate, Cargo.lock, closed.tsv contents,
  live state.db, existing 2026-11-10 handoff files,
  docs/rfc-169-parser-spine.md, NovaShell source, and any secret
  or confidential value.

## Success Criteria
- [ ] G1 AGENTS.md describes how we work now, includes the
      INT-302 findings-close sentences, and includes the approved
      improvements A to G. Claude and Jarvis are not named as
      working agents.
      ask: AGENTS.md matches the approved draft
      ruling:
      evidence:
- [ ] G2 PLAYBOOK.md and zero/handoffs/README.md match the same
      triangle. The playbook no longer staffs Jarvis as
      conductor or Claude as backup. The handoffs README lists
      current writers as eric, cursor, or christian, and says
      existing files keep the names they were written under.
      There is no other live handoff doc or template in
      zero/handoffs/ (the 2026-11-10 files are history and stay).
      ask: PLAYBOOK.md and the handoffs README match the triangle, with no Jarvis or Claude as a current writer
      ruling:
      evidence:
- [ ] G3 no Claude or Jarvis as a working agent in live workflow
      docs. Proof: `git grep -i -e claude -e jarvis` on
      AGENTS.md, zero/orchestration/PLAYBOOK.md, and
      zero/handoffs/README.md after the change, with the output
      pasted and each hit classified. Allowed hits in those
      three files: the playbook sentences that record the
      cancelled INT-304 staffing and the path
      zero/handoffs/2026-11-10-jarvis.md. Allowed historical
      mentions left unchanged in other files:
      docs/rfc-169-parser-spine.md line 5 (authorship);
      zero/handoffs/2026-11-10-jarvis.md,
      2026-11-10-cursor.md, and 2026-11-10-zero-digest.md
      (INT-304 first-week files); comments in
      zero/shell/novashell/src/cheatsheet_tui.rs:221 and
      zero/shell/novashell/src/core_integration.rs:307; and the
      word Jarvis as a score or gate name in engine code, which
      is not a workflow doc. A live assignment of Claude or
      Jarvis as a working agent or current writer is a failure.
      ask: live workflow docs do not name Claude or Jarvis as a working agent
      ruling:
      evidence:
- [ ] G4 doors and d green on the laptop.
      ask: doors and d are green
      ruling:
      evidence:

## Versions
- Docs only. No crate bump.
- Close with: cicomplete 332 --skip-bumps
  (no --bump). INT-327 (README, also docs only) stated "No code
  changes and no version bumps." Its complete commit, 2d6da83c,
  did not record the cicomplete flags; that close was before
  cicomplete removed the file and wrote closed.tsv. Current
  cicomplete asks about every touched crate unless each is named
  with --bump or the rest are skipped. This intent touches no
  crate, so --skip-bumps with no --bump is the form.

## INT-302
The two AGENTS.md sentences drafted in INT-302 G10-G12 land in
this intent. INT-302 needs a ruling not to re-add them. F-0101,
F-0108, and the NOVASHELL.md proposal stay in INT-302.
