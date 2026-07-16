---
id: 110
date: 2026-07-02
type: future
title: "v2 structure (the metal-isolated risk-tiered vision)"
status: cancelled
tags: [structure, 0-core]
cancelled_date: 2026-07-15
---
## Cancelled
Cancelled 2026-07-15 -- duplicate of INT-112 "0-Core v2 risk-tiered metal-isolated structure",
which is the real intent: 95 lines, 6 gates, and it BUILDS ON INT-061 explicitly. This one is a
title plus an untouched template, filed 2026-07-02.

NOTHING IS LOST. 110's only real content was the POST-RESTRUCTURE CHECKLIST (fsh-test path debt),
and that text already lives in INT-112 at lines 82-93 VERBATIM -- checked before cancelling, key
sentence and all ("INT-061's restructure moved dirs under faelight/ and left 17 fsh-test failures
with stale pre-061 paths"). Someone appended the same checklist to both files and only 112 ever
grew a body.

Found during a 2026-07-15 ledger triage prompted by "we are creating more intents than getting
intents done." The triage disproved the premise (192 total, 134 complete, velocity accelerating at
40.7/month) but surfaced this and three other title-only intents: 145 (11 lines, empty template)
and 010/013/014 (0 gates, 16-19 lines). A title is not an intent.

## Vision
[Describe the goal and desired outcome]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] ...

---


## POST-RESTRUCTURE CHECKLIST -- fsh-test path debt (added 2026-07-07)
Any directory move in this restructure WILL silently break fsh-test, which hardcodes
repo paths in its assertions (faelight/rust-tools/fsh-test/src/main.rs). Precedent:
INT-061's restructure moved dirs under faelight/ and left 17 fsh-test failures with
stale pre-061 paths (rust-tools, engine, intents, runtime, pkgs->packages) -- found
only when the suite was run much later.

After ANY dir move here:
1. Update fsh-test path references AND top-level-structure expectations (e.g. a test
   doing `ls ~/0-core` expecting a dir that moved must expect the new top-level name).
2. Rebuild: nix develop ~/0-core#faelight-forest -c cargo build -p fsh-test
3. DEPLOY -- the `fsh-test` command runs the Nix-DEPLOYED binary, not target/debug.
   A cargo build alone shows green while the live command still fails. Must `dep`.
4. Confirm 82/82 on the deployed binary before considering the move done.
