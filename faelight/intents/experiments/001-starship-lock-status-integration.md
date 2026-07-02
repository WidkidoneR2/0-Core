---
id: 001
date: 2025-12-31
type: experiments
title: "Starship Lock Status Integration"
status: complete
tags: [starship, core-guard, shell]
---

## Why

Wanted cleaner lock status indication than verbose text warnings.

Current approach (core_guard function):

```
⚠️  Protected zone (UNLOCKED) - lock-core when done
```

Idea: Integrate into Starship prompt instead.
Inline icon vs separate line.

## What We Tried

Custom Starship module to show 🔒/🔓 in prompt:

```toml
[custom.core_lock]
command = "check if locked, output icon"
when = '[[ $PWD == ~/0-core* ]]'
```

## What Happened

Attempted integration, deleted it during testing.
Realized it conflicted with experiment mindset.

Decided to save for v3.5.2 (Shell Safety & Polish).

## What We Learned

1. Don't experiment with infrastructure during foundation building
2. Save polishing ideas for appropriate releases
3. Document the idea even if not implemented yet
4. Momentum matters - ship foundation first, polish later

## Outcome

- Experiment abandoned temporarily
- Idea preserved for v3.5.2
- Will implement as part of Shell Polish theme
- This is how we learn: try, evaluate, decide

## Status

Complete (as an experiment).
Will implement properly in v3.5.2.

### 2025-12-31

[Initial creation]

---

_Part of the 0-Core Intent Ledger_ 🌲
