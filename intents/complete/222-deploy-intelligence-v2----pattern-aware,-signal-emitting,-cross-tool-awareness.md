---
id: 222
date: 2026-04-10
type: feature
title: \"Deploy Intelligence v2 -- Pattern-Aware, Signal-Emitting, Cross-Tool Awareness\"
status: complete
tags: [feature, rust, faelight]
version: TBD
---
Deploy v1 does one thing well: build, version, symlink, verify.
It treats every deploy identically. It has no memory.
It emits no signals. It knows nothing about what other tools depend on it.
It does not feed the pattern weight engine.
Every time you deploy, the forest learns nothing.
Every time you deploy during a bad health window, deploy does not warn you.
Every time you deploy a tool that three other tools depend on, deploy does not tell you.
That ends with v2.
Before deploying:
- Reads current health from state.db -- warns if health < 95%
- Reads active intents -- surfaces which intent this deploy belongs to
- Reads dependency graph -- warns if downstream tools will be affected
- Reads deploy history -- warns if this tool has failed 2+ times recently
After deploying:
- Runs targeted health check (not full d -- just affected checks)
- Emits structured signal to engine_signals
- Writes deploy_pattern to state.db
- Updates pattern_weights with deployment outcome
Every deploy writes to deploy_patterns table:
timestamp, tool, version, commit, health_before, health_after,
duration_ms, outcome (success/failed/rolled-back),
active_intents, triggered_by (manual/cistart/auto), downstream_tools.
Pattern engine learns:
- Which tools fail most often after deploy?
- Which health states correlate with deploy failures?
- Which intents produce the most deploys per session?
- What time of day produces the most rollbacks?
Every deploy emits to engine_signals:
source: "deploy", signal_type: "deploy"
payload: tool, outcome, health_after
weight: 1.0 (success), 0.3 (failed), 0.0 (rolled-back)
Deploy maintains a dependency map:
faelight-shell → faelight-term (uses fsh as default shell)
core → all tools that call core commands
faelight-git → fg alias, cistart/cicomplete hooks
When you deploy faelight-shell, deploy warns:
faelight-term depends on faelight-shell -- restart after deploy.
When you deploy core, deploy warns:
17 tools call core commands -- run d after deploy.
fg rollback                   -- show last 5 deploys, pick one to restore
fg rollback faelight-shell    -- rollback specific tool to previous version
fg rollback --dry-run         -- show what would change without doing it
Rollback is safe: deploy already saves versioned binaries in bin/.
The symlink chain is the only thing that changes.
Health check runs after rollback to confirm recovery.
deploy faelight-shell --intent 208
Explicitly links this deploy to an intent.
Friday uses this to estimate future intent complexity.
deploy check-deps faelight-shell
Shows the full dependency graph for a tool before deploying.
INT-208 Tool Intelligence L2 -- pattern logging foundation
engine_signals table (already exists)
Phase 1 -- deploy_patterns table + signal emission
Phase 2 -- health check before/after deploy (targeted)
Phase 3 -- dependency awareness map
Phase 4 -- fg rollback command
Phase 5 -- deploy --intent flag
Phase 6 -- deploy check-deps command
✅ deploy_patterns table created in state.db (2026-04-13)
✅ every deploy writes structured deploy_pattern (2026-04-13)
✅ every deploy emits to engine_signals (2026-04-13)
✅ health check before deploy warns if health < 95% (2026-04-13)
✅ health check after deploy targeted verification (2026-04-13)
✅ dependency map defined for all major tools (2026-04-13)
✅ deploy warns when downstream tools are affected (2026-04-13)
✅ fg rollback shows last 5 deploys, restores selected (2026-04-13)
✅ fg rollback tool-specific rollback works (2026-04-13)
✅ deploy --intent flag links deploy to active intent (2026-04-13)
✅ deploy check-deps shows full dependency graph (2026-04-13)
✅ pattern_weights receiving deploy signals (2026-04-13)
✅ d passes 100% after full implementation (2026-04-13)
"A deploy that does not remember what it changed
cannot warn you before it changes something again.
Deploy v2 is not faster. It is honest." 🌲