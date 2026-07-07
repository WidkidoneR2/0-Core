---
id: 110
date: 2026-07-02
type: future
title: "v2 structure (the metal-isolated risk-tiered vision)"
status: planned
tags: [structure, 0-core]
---

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
