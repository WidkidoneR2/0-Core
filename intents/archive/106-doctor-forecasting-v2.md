---
id: 106
date: 2026-03-02
type: future
title: "doctor forecasting v2 — Predictive Health Intelligence"
status: complete
tags: [doctor, forecasting, health, rust, intelligence, glow]
version: TBD
priority: medium
---

## Vision

The doctor knows the past. v2 makes it predict the future.

Not "health is 95%" but "at current trajectory, health will drop
in 3 days unless you update libxml2."

The forest stops being reactive and becomes anticipatory.

## Approach

- Analyze health-history.jsonl for patterns
- Correlate: security findings age → health drop timeline
- Correlate: package staleness → predicted warnings
- Correlate: intent drift duration → health instability
- Output: `core doctor forecast` with confidence intervals
```
core doctor forecast
  📊 7-day health projection
  Today:    95%
  +2 days:  95% (stable)
  +5 days:  90% ⚠️  (libxml2 CVE age threshold)
  +7 days:  85% ⚠️  (3 packages approaching staleness)

  → Recommended: update within 4 days
```

## Success Criteria

- [x] 7-day health projection
- [x] CVE age correlation
- [x] Package staleness prediction
- [x] Intent drift correlation
- [x] Confidence intervals shown

---

*"The forest sees what is coming."* 🌲
