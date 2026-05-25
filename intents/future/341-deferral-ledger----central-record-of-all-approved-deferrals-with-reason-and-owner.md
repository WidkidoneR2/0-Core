---
id: 341
title: "Deferral Ledger -- central record of all approved deferrals with reason and owner"
status: planned
date: 2026-05-25
tags: [deferral, ledger, accountability, gates, christian, approval, audit]
---

## Why This Exists

On 2026-05-25, christian approved the first formal deferrals in the forest:
- INT-292 gates deferred to INT-324 (faelight-term v4)
- INT-313 gates deferred to INT-324 and NixOS

The deferral format (⏸ gate -- deferred: reason -- approved by: christian date)
exists in the individual intent files. But there is no central place to see
ALL deferrals across the entire forest at once.

This is a problem because:
1. Deferrals can be forgotten -- no visibility means no accountability
2. Before NixOS migration, every deferred gate must be reviewed
3. The human (christian) must be able to see what they approved and why
4. Claude cannot defer gates -- only propose them. This ledger proves it.

## Design

A `core deferral list` command that:
- Scans all intent files (complete/, in-progress/, future/)
- Finds every ⏸ line
- Extracts: gate text, reason, date, approver
- Displays as a sorted, searchable list
- Flags deferrals that are older than 30 days without resolution

The ledger is read-only -- it reflects what is in the intent files.
Changing a deferral means changing the intent file with your approval.

## Current Deferrals (2026-05-25)

### INT-292 → INT-324
- Window resize -- deferred to faelight-term v4 on NixOS
- Path resilience doctor check -- deferred to INT-324
- 1 week daily driver -- deferred to NixOS/INT-324
- v2 source retired -- deferred to INT-324
- foot retirement -- deferred to NixOS/INT-324
Approved by: christian 2026-05-25
Reason: term v4 will be the daily driver on NixOS, v3 served its purpose

### INT-313 → INT-324
- Nested compositor rendering -- deferred to INT-324 v4 rebuild
- EGL/softbuffer fallback -- deferred to INT-324 v4 rebuild
- Sole daily driver -- deferred to NixOS/INT-324
- Runs inside compositor cleanly -- deferred to INT-324 + INT-308 Phase 5
Approved by: christian 2026-05-25
Reason: term v4 on NixOS is the right place to solve these properly

### INT-332 (gate 7)
- Doctor check: detects ✅ gates without date stamps
Approved by: christian 2026-05-25
Reason: high effort for low signal, deferred to future integrity work

### INT-333 (gates)
- Doctor integrity reads from git_operations
- Integrity drift elimination (10 consecutive runs)
- COSMIC notification on push
Approved by: christian 2026-05-25
Reason: long-term proof needed, compositor not stable enough yet

### INT-339 (gates)
- Documentation audit
- Intent file audit (historical references)
- 10 consecutive d runs at 100%
Approved by: christian 2026-05-25
Reason: historical references acceptable, track over time

### INT-251 (gate)
- Friday checks open gates on every cicomplete attempt
Approved by: christian 2026-05-25
Reason: requires Friday integration work in INT-251 itself

## Gates

⬜ core deferral list command implemented
⬜ Scans all intent files for ⏸ lines
⬜ Extracts gate, reason, date, approver for each deferral
⬜ Displays sorted by intent ID
⬜ Flags deferrals older than 30 days without resolution
⬜ Pre-NixOS audit: all deferrals reviewed and confirmed or resolved
⬜ Demonstrated: core deferral list shows all current deferrals accurately
