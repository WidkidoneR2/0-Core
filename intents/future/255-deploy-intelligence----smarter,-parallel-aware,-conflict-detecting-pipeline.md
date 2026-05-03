---
id: 255
date: 2026-04-28
type: arch
title: "Deploy Intelligence -- Smarter, Parallel-Aware, Conflict-Detecting Pipeline"
status: in-progress
tags: [arch, deploy, parallel, intelligence, performance, fsh, infrastructure]
version: TBD
---

## Vision

The current `deploy` script (rust-tools/scripts/deploy) is a sequential
workflow: cargo build, install_bin, registry update, signal emit. Each
step blocks the next. For multi-tool deploys (rare today, common as the
forest grows), this becomes the bottleneck.

This intent makes deploy intelligent:
- Aware of which tools can build in parallel (no shared cache contention)
- Aware of dependency graphs between tools (faelight-term depends on faelight-shell)
- Capable of detecting conflicts before starting (lock state, disk pressure, etc.)
- Reporting structured success/failure across parallel batches
- Honest about what it's doing — dry-run plan view available

This is NOT a rewrite. The deploy script's domain logic stays. We add
an orchestration layer that schedules existing deploy steps with awareness.

## Why Now

External architectural input from a senior engineer (saved 2026-04-28)
laid out the design pattern: Task DAG + scheduler + resource model with
explicit user controls. That email is the reference for full implementation.

Current friction is small (deploys are usually one tool at a time) but
will become significant as:
- More tools enter the registry (currently 51, growing toward target 70+)
- Multi-service deploys become common (deploy api / web / worker patterns)
- CI-style workflows need parallel verification

Investing now positions the forest before the friction becomes painful.

## Approach

### Phase 1: Annotate (no behavior change)
- Each tool in registry declares: writes, reads, network, depends_on
- Existing serial deploy ignores annotations (forward-compatible)
- New: `deploy plan <tools...>` shows what WOULD happen

### Phase 2: Explicit parallel
- New syntax: `deploy --parallel core shell term`
- Reads annotations, validates no conflicts
- Spawns each in own task with labeled output stream
- Reports per-tool success/failure

### Phase 3: Auto-parallel detection (conservative)
- `deploy core shell term` (no flag) detects independence from annotations
- Conservative default: if ANY tool lacks annotation, fall back to serial
- User can override: `--strict-serial` or `--force-parallel`

### Phase 4: Trust model
- Successful parallel runs recorded as "safe pair" learned facts
- Failed parallel runs recorded as "needs investigation"
- Friday surfaces patterns: "deploy core + shell ran 12 times, never conflicted"
- Future runs lean on historical signal alongside annotations

### Phase 5: Integration with fsh v9 task graph
- If/when fsh v9 ships a general task DAG executor (Pillar 1 of INT-245),
  deploy's orchestration layer migrates onto it
- Until then, deploy runs its own narrow scheduler

## Architectural Reference

Full design in saved external architectural input (2026-04-28). Key
mental model:
- Task DAG with explicit dependencies
- ResourceSet (writes/reads/network) per task
- Conservative defaults, explicit overrides
- Dry-run plan visualization required before parallel auto-detection

This intent is the deploy-scoped slice of that vision. The full
generalization (across all fsh commands) lives in INT-245 Pillar 1.

## Hard Dependencies

- Existing deploy script and registry (no breaking changes)
- Tool annotations: registry schema needs `deploy_metadata` field
- Task labeling for output multiplexing (similar to INT-249b PTY scanner)
- Optional: tokio runtime in deploy (currently shell script -- may need
  Rust rewrite for proper async orchestration)

## Success Criteria

- [ ] Tool registry entries support deploy metadata (writes, reads, network)
- [ ] `deploy plan <tools>` shows execution plan with parallelization decisions
- [ ] `deploy --parallel <tools>` runs independent tools simultaneously
- [ ] Parallel output streams labeled by tool name, no interleaving
- [ ] Failure in one tool does not cancel other parallel tasks (default)
- [ ] `--fail-fast` flag cancels remaining tasks on first failure
- [ ] Trust model: successful pairs recorded to friday_knowledge
- [ ] Auto-parallel mode (no explicit flag) defaults to safe-when-known,
      serial-when-unknown
- [ ] Deploy script (or its replacement) measurably faster on multi-tool
      deploys (target: 3+ tools deploy in <60% of serial time)
- [ ] No regression in single-tool deploys

## Scope

### In scope
- Deploy orchestration with parallelism awareness
- Tool annotations in registry
- Plan visualization
- Trust learning loop
- Output stream multiplexing for parallel tasks

### Out of scope
- General-purpose parallel command execution (INT-245 Pillar 1)
- Pre-build resource checking (CPU / memory / disk pressure)
- Cross-machine deploy orchestration (single-host only)
- Rollback automation (deploy already maintains versioned binaries
  for manual rollback)
- Deploy graph visualization in TUI (could be a `dt` ratatui tool, separate intent)

## Gate Check
⬜ Not started

---

*"The deploy system orchestrates -- it does not replace the work itself.
Smarter scheduling makes the same work finish faster, with clearer signal."* 🌲

1. **dry-run / plan command** -- `faelight-release plan 12.1.0` shows exactly what
   will happen before committing. Inspired by cargo-release's dry-run-first philosophy.
2. **SIGPIPE fix** -- faelight-release panics on broken pipe when piped to head.
   Same fix as fsh: combine SIG_DFL with panic hook.
3. **Workspace version sync** -- when faelight-shell bumps major, surface which
   downstream tools (faelight-term, faelight-login) should bump minor.
   Friday will eventually make this judgment; for now use intent tag heuristic:
   innovation/architecture/parallel → suggest major
   feature/shell → suggest minor
   fix/chore → suggest patch
faelight-release does more: forest narrative, health at release, Friday insights,
synthesis layer. Borrow dry-run concept only.
