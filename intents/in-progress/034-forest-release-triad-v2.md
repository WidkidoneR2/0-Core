---
id: 034
date: 2026-06-04
type: feature
title: "Forest release v2: generation + commit + intent triad tracking"
status: in-progress
tags: [release, generation, triad, friday, versioning]
priority: high
---

## Vision

Every release records the triad in state.db:
  Release version = NixOS generation = Git commit count

Example:
  Faelight NixOS 1.0.0
    generation: 47
    commits: 2984
    intents: INT-001 through INT-025

Friday uses this to:
- Trace any bug to exact generation + commit
- Answer "which generation is stable?"
- Warn before garbage collection removes a release generation
- Cross-reference rollback targets with release history

## Why this matters

NixOS keeps generations but GC removes old ones.
The release triad in state.db survives GC.
Friday becomes the permanent memory when generations are gone.

## Approach

- faelight-release v2 records triad on every bump
- core release show displays triad history
- Friday queries: "what generation is version 1.0.0?"
- GC warning: "generation 47 (release 1.0.0) about to be collected"

## Gate

- [ ] Every release records generation + commit count + intent range
- [ ] core release show displays full triad history
- [ ] Friday can answer "which generation is release X?"
- [ ] GC warning fires before release generation is collected
- [ ] Subsumed into INT-031 or standalone


## MERGED INTO INT-031 (2026-06-27)
INT-034 (triad tracking) is being built as part of INT-031 (faelight-release v2). The triad is
031's core deliverable -- see 031's Phase 0 recon for the combined gate-set. This intent's gates
are tracked there; 034 and 031 will cicomplete together. Kept as a separate ledger entry for
traceability (the triad has its own identity), but the WORK lives in 031.
