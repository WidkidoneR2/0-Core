# intent v2.0.0

🌲 **Intent Ledger Manager** - Track every system change with intentionality.

## Philosophy

> "Every change should be intentional, documented, and traceable."

The Intent Ledger is the heart of Faelight Forest's philosophy of **Manual Control Over Automation**. Rather than letting the system evolve chaotically, every significant change starts as an intent - a documented decision with clear vision, problem statement, and success criteria.

## Features

- **Add intents** - Document decisions before they're made
- **Status tracking** - planned → in-progress → complete
- **Auto-organization** - Intents move to correct directories based on status
- **Search & filter** - Find intents by content, status, or tags
- **Statistics** - Visual progress bars, success rates, category breakdown
- **Timeline** - Chronological view of system evolution
- **Validation** - Deep integrity checks for malformed intents
- **Health monitoring** - System health checks

## Usage
```bash
# List all active intents
intent list

# Show only planned intents
intent list --planned

# Show specific intent
intent show 052
intent 052  # Shorthand

# Search for intents
intent search rust
intent search --tag v7.0
intent search --status complete

# Statistics with visual progress bars
intent stats

# Chronological timeline
intent timeline

# Validate Intent Ledger integrity
intent validate

# System health check
intent --health

# Add new intent (interactive)
intent add

# Change intent status (auto-moves to correct directory)
intent complete 052
intent start 061
intent defer 054
intent cancel 049
```

## Intent Structure

Every intent is a markdown file with YAML frontmatter:
```markdown
---
id: 052
date: 2025-01-15
type: future
title: "Intent System Enhancements"
status: in-progress
tags: [v8.0, tooling]
---

## Vision
[What are we trying to achieve?]

## The Problem
[What problem does this solve?]

## The Solution
[High-level approach]

## Success Criteria
- [ ] Feature implemented
- [ ] Tests passing
- [ ] Documentation updated
```

## Categories

- **decisions** - Architectural choices and policies
- **experiments** - Trials and explorations
- **philosophy** - Core principles and values
- **future** - Planned work and roadmap
- **incidents** - Issues and their resolutions

## Status Flow
```
planned → in-progress → complete
           ↓
        deferred
           ↓
        cancelled
```

## Integration with 0-Core

The Intent Ledger is checked by:
- `dot-doctor --health` - Validates intent count and structure
- Git commit hooks - Requires intent IDs in commit messages
- Release process - Links releases to completed intents

## Demo for Linus
```bash
# Show system health
intent --health

# Show statistics with visual progress
intent stats

# View chronological timeline
intent timeline

# Deep validation check
intent validate

# Show specific intent
intent 052
```

## Why This Matters

Traditional system administration is reactive - you fix what breaks. Faelight Forest is **intentional** - every change starts with a documented intent that captures:

1. **Why** - The problem being solved
2. **What** - The desired outcome
3. **How** - The high-level approach
4. **When** - Success criteria

This creates an **audit trail of thought** that makes the system explainable, maintainable, and trustworthy.
