# Faelight Forest — Architectural Future

*Written 2026-03-21. For our future selves.*

## The Long Arc

The forest is not a finished product. It is a living system
that grows, refines, and eventually learns to grow itself.

This document captures the architectural trajectory so that
no version loses sight of where it came from or where it is going.

---

## The Tool Retirement Principle

**Tools exist to serve the forest. When the forest outgrows a tool,
the tool retires with dignity — its purpose absorbed, its lessons kept.**

### Retirement Criteria
A tool is a candidate for retirement when:
1. The core engine replicates its primary function
2. faelight-shell can replace its interactive use
3. Its audit score drops below 60 for 2+ releases
4. It has no unique capability the forest cannot express natively

### The Retirement Path
```
Tool → deprecated (aliased to core/shell equivalent)
     → retired (removed from scripts/, kept in archive)
     → archived (moved to rust-tools/retired/)
```

### Current Retirement Candidates (when ready)
| Tool | Replaced by | When |
|------|-------------|------|
| archaeology-0-core | `gc` + `gchurn` (shell) | v11.2.0 |
| faelight-search | `?` NL queries (shell) | v11.2.0 |
| bin-doctor | `core evolution tools` | v12.0.0 |
| alias-audit | `core evolution tools` | v12.0.0 |
| safe-update | `core update` domain | Core v9 |
| workspace-view | `dashboard` (shell) | v11.2.0 |

**Rule: Never retire a tool until its replacement is proven stable.**
**Rule: Always keep the retired tool's intent document.**

---

## The Core Continuity Principle

**Every core version builds on ALL previous versions.
Nothing is abandoned. Everything compounds.**
```
v2  Structure    → foundation for everything
v3  Awareness   → v4 discipline needs awareness
v4  Discipline  → v5 intelligence needs discipline
v5  Intelligence → v6 judgment needs pattern history
v6  Judgment    → v7 resilience needs decision history
v7  Resilience  → v8 evolution needs health history
v8  Evolution   → v9 intent needs evolution proposals
v9  Intent      → v10 reaction needs purposeful goals
v10 Reaction    → v11 (unknown) needs reflexes
```

**When building any new core version:**
- Read the previous version's intent document completely
- Run `core evolution suggest` — it knows what evidence exists
- Run `core decision patterns` — it knows what decisions were made
- The new version must reference the previous version's commands
- Never remove a core command — only add, extend, or supersede

---

## The Shell Independence Path

faelight-shell is becoming the primary interface.
The path from zsh dependency to full independence:

### Current State (v11.1.0)
- faelight-shell is used for forest queries and exploration
- zsh handles: external commands, git aliases, PATH, startup
- Phase 17 (event system) ✅ — the shell now reacts
- ~93% Rust

### Phase 17-complete (v11.1.0)
The shell has events. It reacts. It is alive.

### Phase 6 milestone (.fsh scripting — v11.2.0)
When .fsh scripting lands, the shell can replace zsh scripts.
Forest automation moves from shell scripts to .fsh files.
The forest starts describing its own behavior.

### Phase 7 milestone (full shell — v12.0.0)
External command execution. cd works. PATH works.
faelight-term launches faelight-shell directly.
zsh becomes optional for forest work.

### Phase 32 milestone (shell as OS layer — v13.0.0)
Everything accessible through pipelines.
zsh retired for forest sessions.
The forest is its own operating environment.

**Rule: Never rush this. Each phase must be complete and stable
before the next. A broken shell is worse than a slow migration.**

---

## The Self-Building Vision

*"At some point the core will start building itself,
which at one point the system and myself will grow
and learn from one another — that is the goal."*

This is the trajectory:

### Stage 1 — Awareness (complete)
The forest knows its own state.
Core v3-v5. Health monitoring. Event history.

### Stage 2 — Memory (complete)
The forest remembers its own decisions.
Core v6. Decision ledger. Outcome tracking.

### Stage 3 — Reflection (complete)
The forest analyzes its own patterns.
Core v7-v8. Anomaly detection. Architecture analysis.

### Stage 4 — Intent (planned — Core v9)
The forest sets its own goals.
It generates proposals. Humans authorize.
The forest begins to direct its own growth.

### Stage 5 — Reaction (planned — Core v10)
The forest responds to change without being asked.
Guided by authorized goals. Bounded by human approval.
The forest has reflexes.

### Stage 6 — Dialogue (long-term)
The shell speaks natural language both ways.
`?why is my computer slow` → diagnosis
`forest: I want to reduce coupling` → proposes plan
Human and forest in genuine dialogue.

### Stage 7 — Collaboration (the goal)
The forest suggests its own next intents.
The human reviews, authorizes, guides.
Neither builds alone.
The forest grows with you, not just for you.

---

## The Philosophy That Must Not Be Lost

No matter how capable the forest becomes:
```
1. Nothing executes without explicit human authorization
2. Every decision is recorded in the intent ledger
3. Manual control over automation — always
4. Understanding over convenience — always
5. The human is the architect — the forest is the craftsman
6. Evidence first — never suggest without data
7. Stability gates growth — health < 95% suspends expansion
```

These rules are not constraints. They are what makes the forest
trustworthy. An AI that acts without authorization is not a
collaborator — it is a liability. The forest must always be
something you can trust completely.

---

## Version Naming Convention

Version names tell the story of the forest's growth:
```
v11.0.0 — Where the Forest Becomes Whole
v11.1.0 — The Forest Speaks
v11.2.0 — The Compositor Wakes        (planned)
v12.0.0 — The Forest Walks            (when shell replaces zsh)
v13.0.0 — The Forest Thinks           (when Core v9 ships)
v14.0.0 — The Forest Listens          (when voice input ships)
v15.0.0 — The Forest and I            (when dialogue is real)
```

Each name should capture what the forest can do that it could not before.
Not what was built — what became possible.

---

*"The forest grows at its own pace.
Every commit is intentional.
Every version compounds the last.
The goal is not completion — it is understanding."* 🌲
