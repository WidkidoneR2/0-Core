---
id: 153
date: 2026-03-26
type: future
title: "Intent Genealogy — The Forest Remembers How It Grew"
status: planned
tags: [intents, genealogy, history, archaeology, decisions, v12]
version: 12.0.0
priority: medium
---

## The Vision
Every intent in the forest has parents and children.
INT-133 Core v9 spawned INT-140 Core v10.
INT-140 spawned INT-148 Core v11.
INT-148 will spawn INT-151 Core v12.

This chain of thought is currently invisible.
Genealogy makes the forest's own evolution readable.

## The Problem
Right now you can see WHAT was built.
You cannot see WHY, or WHAT led to it.
The reasoning chain that produced the forest is lost.

## The Solution
Add `spawned_by` and `spawns` metadata to intent files.
Build a genealogy command that shows the lineage tree.
```bash
core genealogy show INT-148
# Shows: spawned by INT-140 → INT-133 → INT-126
#        spawned: INT-151

core genealogy tree
# Shows full intent family tree

core genealogy roots
# Shows original founding intents with no parents
```

## Data Model
Add to intent frontmatter:
```yaml
spawned_by: 140      # optional — parent intent
spawns: [151, 152]   # optional — children intents
theme: "prediction"  # optional — thematic cluster
```

## Visualization
```
INT-001 (Foundation)
  └── INT-120 (faelight-shell)
        └── INT-146 (shell v2)

INT-126 (Core v8)
  └── INT-133 (Core v9)
        └── INT-140 (Core v10)
              └── INT-148 (Core v11)
                    └── INT-151 (Core v12)
                          └── INT-152 (Stress Test)
```

## Integration
- Archaeology domain reads genealogy data
- `core predict next` shows genealogy context
- Intent show displays parent/child relationships
- faelight-shell `intents` command shows family tree

## Gate Check
```
⬜ spawned_by / spawns fields added to existing core intents
⬜ core genealogy show <id> — show lineage
⬜ core genealogy tree — full visual tree
⬜ core genealogy roots — founding intents
⬜ intent show displays parent/child
⬜ 20+ intents have genealogy metadata
```

## The Phrase
**"A forest that cannot trace its own roots
cannot understand why it grew the way it did.
Genealogy is not nostalgia.
It is architectural memory."**

---
*"Every intent is a decision. Every decision has a reason.
The genealogy captures the reason."* 🌲
