# Intent Ledger - The Memory of 0-Core

**Purpose:** Capture the WHY behind every decision, experiment, and vision.

## Philosophy

> "Code shows HOW. Intent shows WHY."

The Intent Ledger is 0-Core's institutional memory. It records:

- **Decisions** - What we chose and why
- **Experiments** - What we tried (success or failure)
- **Philosophy** - Core beliefs and principles
- **Future** - Vision for what's coming

## Structure

```
INTENT/
├── decisions/    # Major architectural choices
├── experiments/  # Things we tried (worked or not)
├── philosophy/   # Core beliefs and principles
└── future/       # Planned features and vision
```

## File Format

Each intent is a numbered Markdown file with YAML frontmatter:

```yaml
---
id: 001
date: 2025-12-31
type: decision
title: "Complete Rust Transition - v5.0 onwards"
status: planned
tags: [rust, architecture, security]
---

## Why

[Explanation of reasoning]

## What

[What we're doing]

## Alternatives Considered

[What we didn't choose and why]

## Impact

[Expected outcomes]
```

## Usage

```bash
# Add new intent
intent add

# List all intents
intent list

# Show specific intent
intent show 001

# Search by tag
intent search rust
```

## Principles

1. **Capture everything** - Ideas, decisions, failures, successes
2. **Write for future you** - You'll forget why you made choices
3. **Be honest** - Record failures and learnings
4. **Date everything** - Context matters

---

_The forest remembers._ 🌲
