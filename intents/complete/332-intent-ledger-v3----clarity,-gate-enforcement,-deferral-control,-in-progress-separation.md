---
id: 332
title: "Intent Ledger v3 -- clarity, gate enforcement, deferral control, in-progress separation"
status: complete
date: 2026-05-25
tags: [intent, ledger, clarity, gates, enforcement, deferral, ux]
---

## The Problem

The intent ledger has grown into a source of confusion:

1. In-progress intents appear mixed into the future/planned list -- no clear separation
2. Gates can be marked complete without demonstration -- no enforcement
3. Deferral happens silently -- Claude defers things without explicit human approval
4. The list command shows planned and in-progress in the same section
5. Integrity drift happens because the ledger does not reflect reality

This intent fixes all of these. The forest's ledger should be the clearest
thing in the system, not the most confusing.

## Design

### Separation
In-progress intents must always appear first, in their own clearly labeled block.
The list command output:

  ACTIVE (in-progress):
    251  Core v23 -- Friday Becomes Central
    308  faelight-compositor v2
    313  faelight-term v3 stabilization

  PLANNED (next up):
    322  fsh v4 ...
    324  faelight-term v4 ...

No mixing. No ambiguity.

### Gate Enforcement
Before any intent can be marked complete (cicomplete), the system must:
1. Read the intent file
2. Count open gates (⬜)
3. If any open gates exist without a formal ⏸ deferral: BLOCK completion
4. Print the open gates and require human to either demonstrate or formally defer

This is a hard block. Not a warning. Not a suggestion.

### Deferral Control
Only the human (Christian) can defer a gate. The format:
  ⏸ gate description -- deferred: [reason] -- approved by: christian [date]

Claude cannot mark a gate deferred. Claude can propose a deferral.
The human must type the approval explicitly.

### Integrity Leak Detection
The doctor check for integrity must detect:
- Intents marked in-progress but file is in future/ directory
- Intents marked complete but file is in in-progress/ or future/
- Gates marked ✅ without a date stamp
- Deferred gates without human approval signature

## Gates

✅ intent list shows ACTIVE section first, separate from PLANNED 2026-05-25
✅ intent list clearly labels ACTIVE / PLANNED / COMPLETE sections 2026-05-25
✅ cicomplete hard-blocked when open gates exist -- demonstrated live 2026-05-25
✅ cicomplete shows all blocking gates with deferral instructions 2026-05-25
✅ Deferral format enforced -- validated in cicomplete, rejects bad format, accepts correct 2026-05-25
✅ Doctor check: detects in-progress intents in future/ -- INT-308 and INT-313 caught and fixed 2026-05-25
⏸ Doctor check: detects ✅ gates without date stamps -- deferred: requires parsing all gate files, high effort for low signal -- approved by: christian 2026-05-25
⏸ Friday checks open gates on every cicomplete attempt -- deferred: requires Friday integration work in INT-251 -- approved by: christian 2026-05-25
✅ Intent list no longer mixes in-progress with planned 2026-05-25
✅ Demonstrated: cicomplete on INT-332 blocked with 11 open gates listed 2026-05-25
✅ Demonstrated: Friday gate deferred with approval signature -- accepted by cicomplete 2026-05-25
✅ Demonstrated: integrity check caught INT-308 and INT-313 in wrong directory, fix applied 2026-05-25
