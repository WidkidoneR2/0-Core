---
id: 305
title: "deploy pipeline v2 -- intelligent, dependency-aware, forest-native"
status: complete
date: 2026-05-14
type: arch
tags: [deploy, pipeline, cargo, audit, rollback, cosmic, intelligence]
depends_on: [304, 183]
---
## The Problem
The current deploy pipeline is solid but manual and flat:
  - No dependency ordering (faelight-shell depends on faelight-git,
    deploy does not know this)
  - No rollback command (versions are kept but no `deploy rollback tool`)
  - No functional smoke test after deploy (binary exists check only)
  - No Cosmic-aware bundling (future tools need assets, shaders, fonts)
  - cargo-audit, cargo-deny, cargo-nextest installed but not wired in
  - No cross-tool health check after deploy
  - No parallel deploy for independent tools
  - Friday does not learn from deploy patterns

---
## The Vision
A deploy pipeline that knows the forest.

Not just "build and copy" but:
  - Knows which tools depend on which
  - Builds in the right order automatically
  - Tests before deploying (fsh-test gate)
  - Audits dependencies for vulnerabilities
  - Rolls back cleanly if something breaks
  - Bundles assets for Cosmic-era tools
  - Runs in parallel where safe
  - Friday learns which deploys succeed and which fail

---
## Improvements

### 1. Dependency ordering
Define tool dependency graph in registry:
  faelight-shell depends_on: [faelight-git]
  faelight-term depends_on: []
  faelight-bar depends_on: [faelight-notify]
  
deploy faelight-shell rebuilds faelight-git first if it changed.
deploy --all builds in topological order.

### 2. Rollback command
deploy rollback faelight-shell
  - Finds previous version in bin/
  - Swaps symlinks atomically
  - Verifies health after swap
  - Records rollback in deploy history

deploy rollback faelight-shell 2    -- rollback 2 versions
deploy rollback --all               -- rollback everything to last checkpoint

### 3. Cargo tools integration
Pre-deploy checks (blocking):
  cargo audit -- known vulnerability scan
  cargo deny -- license compliance

Post-build checks (warning):
  cargo bloat -- binary size regression detection
  fsh-test --category Regression -- regression gate

### 4. Smoke tests after deploy
After binary is deployed, run a functional check:
  fsh-test --tool faelight-shell -- runs shell-specific tests
  Not just "binary exists" but "binary works correctly"

### 5. Cosmic-era asset bundling
When tool has assets/ directory:
  Fonts, shaders, icons bundled alongside binary
  Bundle manifest tracks what was deployed with what version
  Rollback includes asset rollback

### 6. Parallel deploy
Tools with no shared dependencies build in parallel:
  deploy --all --parallel
  Dependency graph determines safe parallelism
  Max parallel: number of CPU cores / 2

### 7. Friday deploy intelligence
After each deploy, Friday records:
  build time, binary size, test results
  deploy success/failure rate per tool
  time-of-day patterns (do morning deploys fail more?)
  
Friday can say:
  "faelight-term deploys have failed 30% of the time this week"
  "build time for faelight-shell increased 40% since last week"
  "you usually deploy core after faelight-shell -- do both now?"

### 8. Deploy briefing
deploy --brief shows:
  what changed since last deploy (git diff summary)
  estimated build time (from history)
  which tests will run
  risk level (Friday-calculated from history)

---
## Gates
Phase 1 -- Cargo tools integration:
- [x] cargo audit runs in pre-deploy check -- wired 2026-05-16
- [x] cargo deny runs in pre-deploy check -- licenses ok gate active 2026-05-16
- [x] audit warns on upstream vulns -- no patchable issues, correct behavior
- [x] binary size reported post-build -- baseline recorded

Phase 2 -- Rollback:
- [x] deploy rollback <tool> works -- atomic swap, binary verified 2026-05-16
- [x] deploy rollback <tool> N works -- N versions back supported
- [x] rollback verified -- binary execution check post-swap
- [x] rollback recorded in deploy history via core deploy record

Phase 3 -- Smoke tests:
- [x] fsh-test --category=regression runs after faelight-shell deploy
- [x] regression failures block deploy -- exit 1 on failure (INT-304 gate)
- [x] smoke test results shown in deploy output

Phase 4 -- Dependency ordering:
- [x] tool dependency graph in registry — depends_on field in tools.toml 2026-05-16
- [x] deploy respects build order — deps built before dependent tool
- [x] deploy --all builds in topological order — topo-order command in registry_tools.py

Phase 5 -- Parallel deploy:
- [x] independent tools build in parallel — background jobs with pid tracking
- [x] dependency graph determines safe parallelism — topo-order + core deploys sequentially
- [x] --parallel flag enables parallel mode — deploy all --parallel

Phase 6 -- Cosmic asset bundling:
- [x] assets/ directory detected and bundled — auto-detected, copied to bin/assets/tool/
- [x] bundle manifest created — manifest.toml with tool, version written on deploy
- [x] rollback includes assets — asset dir noted in rollback, manual review flagged

Phase 7 -- Friday intelligence:
- [x] deploy outcomes recorded in state.db via core deploy record
- [x] Friday reports deploy health trends — deploy brief shows per-tool success rates
- [x] Friday suggests related deploys — friday_knowledge updated with deploy health trend
- [x] deploy --brief shows risk level — LOW/MEDIUM/HIGH based on 7-day failure history

Final:
- [x] deploy pipeline is fully automated and intelligent — audit, deny, tests, deps, parallel, assets, brief all active
- [x] no manual steps required — deploy <tool> handles full pipeline automatically
- [x] rollback always available — deploy rollback <tool> [N] tested 2026-05-16
- [x] Friday understands deploy history — deploy_patterns + friday_knowledge + deploy brief

---
"A deploy that does not know the forest
is just copying files.
A deploy that knows the forest
is the forest maintaining itself." 🌲
