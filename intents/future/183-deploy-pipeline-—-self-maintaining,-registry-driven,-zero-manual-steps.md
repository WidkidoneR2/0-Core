---
id: 183
date: 2026-03-31
type: arch
title: "Deploy Pipeline — Self-Maintaining, Registry-Driven, Zero Manual Steps"
status: in-progress
tags: [deploy, registry, automation, tools, pipeline, architecture, self-healing]
version: 11.6.0
---

## The Problem
The deploy script is a hardcoded list of 6 tools out of 53 registered.
When a tool is added: someone must manually edit deploy.
When a tool is retired: someone must manually remove it.
When a tool changes type: nothing updates automatically.
This is the opposite of a self-maintaining system.

## The Vision
The deploy script reads tools.toml.
It knows every tool, its type, and whether it is deployable.
Adding a tool to the registry automatically makes it deployable.
Retiring a tool from the registry automatically removes it from deploy.
Zero manual steps. Zero drift.

## The Registry Contract
Add metadata to each tool entry in tools.toml:
```toml
[[tool]]
name = "faelight-shell"
version = "0.6.0"
category = "shell"
description = "Forest-native shell environment"
expected_usage = "high"
type = "rust"           # rust | script | system | special
deployable = true       # whether deploy script should handle it
retired = false         # if true, skip in all pipeline operations
```

### Tool Types
```
rust     — cargo build --release -p <name>, copy to scripts/
script   — already in scripts/, no build needed, just verify
system   — managed by pacman/paru, not in 0-core
special  — custom deploy logic (e.g. faelight-login needs sudo)
```

### Retirement Flow
When a tool is retired:
1. Set retired = true in tools.toml
2. fg commit
3. deploy all — automatically skips retired tools
4. doctor — retired tools excluded from path resilience check
5. registry — retired tools shown in separate section

## The New Deploy Script Architecture
```bash
# deploy reads tools.toml directly
deploy_from_registry() {
    python3 -c "
import tomllib, subprocess, sys

with open('$ROOT/01-registry/tools.toml', 'rb') as f:
    data = tomllib.load(f)

for tool in data.get('tool', []):
    if tool.get('retired', False): continue
    if not tool.get('deployable', False): continue
    if tool.get('type') != 'rust': continue
    name = tool['name']
    subprocess.run(['$ROOT/scripts/deploy', name])
"
}
```

## Phase 1 — Registry Schema Update
Add `type`, `deployable`, `retired` fields to all 53 tool entries.
Write migration script — adds defaults without breaking existing entries.

## Phase 2 — Deploy Script Rewrite
Replace hardcoded list with registry-driven loop.
Handle each type correctly:
- rust: cargo build + atomic copy
- script: verify exists + executable
- special: custom handler
- system: skip (managed by paru)

## Phase 3 — Retirement Flow
`core tool retire <name>` — sets retired=true, removes from active pipeline.
`core tool add <name>` — scaffolds new registry entry with correct type.
`deploy all` — reads registry, skips retired, deploys rest.

## Phase 4 — Doctor Integration
Doctor path resilience reads `deployable=true, retired=false` tools only.
Retired tools show in a separate `retired tools` section.
Adding a tool to registry automatically adds it to doctor checks.

## Phase 5 — Self-Healing
If a deployable rust tool is missing from scripts/:
`deploy` detects gap and rebuilds automatically.
`d` shows warning: "faelight-X missing — run: deploy faelight-X"

## The Future (toward AI self-maintenance)
This intent is the foundation for the system rebuilding itself.
When the forest detects a tool is outdated or broken:
- v12 Strategy: "faelight-X needs rebuild" → proposes deploy action
- v13 Autonomy: executes deploy with human confirmation
- v14 Partnership: maintains the full tool ecosystem autonomously

## Gate Check
```
⬜ tools.toml — all 53 tools have type/deployable/retired fields
⬜ deploy script reads from tools.toml — no hardcoded list
⬜ deploy all — deploys all rust deployable non-retired tools
⬜ Retiring a tool — set retired=true, deploy skips it automatically
⬜ Adding a tool — add to registry, deploy picks it up automatically
⬜ Doctor path resilience — reads deployable/retired from registry
⬜ core tool retire <name> — marks tool as retired
⬜ Self-healing — deploy detects and reports missing tools
⬜ Zero manual deploy script edits ever again
```

## The Phrase
**"A system that requires manual steps to maintain itself
is not a system — it is a job.
The forest maintains itself.
The registry is the source of truth.
Everything else reads from it."**

---
*"When you retire a tool, the forest forgets it cleanly.
When you add a tool, the forest knows it immediately.
No manual steps. No drift. No surprises."* 🌲
