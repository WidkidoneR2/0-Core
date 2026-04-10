# INT-222 — Deploy Intelligence v2 — Pattern-Aware, Signal-Emitting, Cross-Tool Awareness
Status: [planned]
Date: 2026-04-10
Tags: deploy, intelligence, patterns, signals, cross-tool, pipeline, v2
Deploy v1 does one thing well: build, version, symlink, verify.
It treats every deploy identically.
It has no memory.
It emits no signals.
It knows nothing about what other tools depend on it.
It does not feed the pattern weight engine.
Every time you deploy, the forest learns nothing.
Every time you deploy during a bad health window, deploy does not warn you.
Every time you deploy a tool that three other tools depend on, deploy does not tell you.
That ends with v2.
Deploy v2 is context-aware:
**Before deploying:**
- Reads current health from state.db — warns if health < 95%
- Reads active intents — surfaces which intent this deploy belongs to
- Reads dependency graph — warns if downstream tools will be affected
- Reads deploy history — warns if this tool has failed 2+ times recently
**After deploying:**
- Runs targeted health check (not full d — just affected checks)
- Emits structured signal to engine_signals
- Writes deploy_pattern to state.db
- Updates pattern_weights with deployment outcome
Every deploy writes:
deploy_pattern:
timestamp:        <when>
tool:             faelight-shell
version:          0.6.0
commit:           abc1234
health_before:    100
health_after:     100
duration_ms:      5600
outcome:          success | failed | rolled-back
active_intents:   ["INT-208", "INT-178"]
triggered_by:     manual | cistart | auto
downstream_tools: ["faelight-term"]
Pattern engine learns:
- Which tools fail most often after deploy?
- Which health states correlate with deploy failures?
- Which intents produce the most deploys per session?
- What time of day produces the most rollbacks?
Every deploy emits to engine_signals:
source:      "deploy"
signal_type: "deploy"
payload:     {"tool":"faelight-shell","outcome":"success","health_after":100}
weight:      1.0 (success) | 0.3 (failed) | 0.0 (rolled-back)
Friday will see this signal and know: deployment activity is high today.
The deploy system maintains a dependency map:
faelight-shell → [faelight-term (uses fsh as default shell)]
core           → [all tools that call core commands]
faelight-git   → [fg alias, cistart/cicomplete hooks]
When you deploy faelight-shell, deploy warns:
⚠️  faelight-term depends on faelight-shell
Restart faelight-term after this deploy to pick up changes.
When you deploy core, deploy warns:
⚠️  17 tools call core commands
Run: d after this deploy to verify full system health.
Current state: rollback is manual and painful.
Deploy v2 adds: `fg rollback`
fg rollback              — show last 5 deploys, pick one to restore
fg rollback faelight-shell    — rollback specific tool to previous version
fg rollback --dry-run    — show what would change without doing it
Rollback is safe because:
- deploy already saves versioned binaries in bin/
- The symlink chain is the only thing that changes
- Health check runs after rollback to confirm recovery
deploy faelight-shell --intent 208
Explicitly links this deploy to an intent.
Stored in deploy_pattern.
Friday uses this to understand: INT-208 required 12 deploys to complete.
That becomes a signal for estimating future intent complexity.
deploy check-deps faelight-shell
Shows the full dependency graph for a tool before deploying.
No surprises.
INT-208 Tool Intelligence L2 — pattern logging foundation
state.db deploy_patterns table
engine_signals table (already exists)
Phase 1 — deploy_patterns table + signal emission
Phase 2 — health check before/after (targeted, not full d)
Phase 3 — dependency awareness map
Phase 4 — fg rollback command
Phase 5 — deploy --intent flag
Phase 6 — deploy check-deps command
⬜ deploy_patterns table created in state.db
⬜ every deploy writes structured deploy_pattern
⬜ every deploy emits to engine_signals
⬜ health check before deploy — warns if health < 95%
⬜ health check after deploy — targeted verification
⬜ dependency map defined for all major tools
⬜ deploy warns when downstream tools are affected
⬜ fg rollback — shows last 5 deploys, restores selected
⬜ fg rollback faelight-shell — tool-specific rollback
⬜ deploy --intent flag links deploy to active intent
⬜ deploy check-deps shows full dependency graph
⬜ pattern_weights receiving deploy signals
⬜ d passes 100% after full implementation
"A deploy that does not remember what it changed
cannot warn you before it changes something again.
Deploy v2 is not faster.
It is honest." 🌲
