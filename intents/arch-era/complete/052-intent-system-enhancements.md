---
id: 052
date: 2026-01-14
type: future
title: "Intent System Enhancements - Workflow & Dependencies"
status: complete
tags: [v11, infrastructure, intent, workflow]
version: 11.0.0
---

## Status Note (updated 2026-02-27)

Written for v8.0.0, now at v10.2.0. Workflow states (`intent start`, `intent complete`)
and analytics belong in Core v3 Phase 6 (INT-093). Dependencies and templates remain
valid future work. Deferring until after v3 Phase 1-3 are complete.

## Vision
Make the Intent system production-grade with dependencies, workflows, and analytics.

## Enhancements

### 1. Dependencies & Blocking
```yaml
# In intent frontmatter
dependencies:
  - 047  # Must complete first
blocks:
  - 051  # Cannot start until this done
relates:
  - 036  # Related but not blocking
```

### 2. Workflow States
```
planned → in-progress → testing → complete → archived
```

**Commands:**
```bash
intent start 053      # planned → in-progress
intent test 053       # in-progress → testing  
intent complete 053   # testing → complete
intent block 053      # Check dependencies
```

### 3. Templates
```bash
intent new feature    # Feature template
intent new fix        # Bug fix template
intent new arch       # Architecture template

# Auto-generates proper structure
```

### 4. Analytics
```bash
intent stats

# Shows:
- Avg completion time by type
- Success rate (complete vs abandoned)
- Dependency bottlenecks
- Burndown chart
- Velocity trends
```

### 5. Automatic Branch Naming
```bash
intent start 053
# Creates branch: intent-053-launcher-v2
# Or: git checkout -b $(intent branch 053)
```

## Success Criteria
- [ ] Dependency tracking works
- [ ] Workflow commands functional
- [ ] Templates create proper intents
- [ ] Analytics provide insights
- [ ] Integration with git branches
- [ ] Zero breaking changes to existing intents

## Timeline
**v8.0.0**

---

_"Intents are the nervous system of 0-Core."_ 🌲
