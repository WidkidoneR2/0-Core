---
id: 002
date: 2025-12-31
type: decisions
title: "Versioning Strategy Clarification"
status: planned
tags: [versioning, semver, documentation]
---

## Why

Current versioning is inconsistent and potentially confusing:

- v3.5.0 = Major feature (Intent Ledger)
- v3.5.1 = Major feature (Git Guardrails)
- v4.0.0 = Major milestone
- v5.0.0 = Complete rewrite (Rust)

Using 0.5 increments for majors blurs meaning.
Package managers expect semantic versioning.

## The Problem

When users see 3.5.0:

- Assumes minor update to v3.x
- Creates sorting confusion (3.10 > 3.5?)
- Doesn't signal scope of changes

## Options

### Option A: Adopt Proper SemVer (Recommended)

```
MAJOR.MINOR.PATCH
5.0.0 = Breaking changes (Rust rewrite)
4.1.0 = New feature (Intent Ledger)
4.1.1 = Bug fix
```

**Pros:** Industry standard, clear meaning, tool support  
**Cons:** Current v3.5.0 becomes v4.1.0 (retroactive confusion)

### Option B: Keep Current, Document Better

```
3.5.0 = Major milestone
3.5.1 = Feature release
3.5.2 = Feature release
4.0.0 = Next major
```

**Pros:** No breaking change to existing versions  
**Cons:** Still confusing, not industry standard

### Option C: Hybrid - Fix Going Forward

```
Keep: v3.5.0, v3.5.1, v3.5.2 (already released)
Switch: v4.0.0 becomes proper SemVer
Future: 4.0.0, 4.1.0, 4.2.0, 5.0.0
```

**Pros:** Clean break at v4.0, honors existing releases  
**Cons:** Inconsistency in history

## Recommendation

**Option C - Hybrid Approach**

Rationale:

- v3.x is already out, changing it creates more confusion
- v4.0.0 is the perfect time to adopt proper SemVer
- Document the change clearly in v4.0.0 release notes
- Going forward: strict SemVer compliance

## Implementation

**v4.0.0 Release Notes:**

```markdown
# Version Numbering Change

Starting with v4.0.0, 0-Core adopts Semantic Versioning (SemVer):

MAJOR.MINOR.PATCH

- MAJOR: Breaking changes, architecture shifts
- MINOR: New features, backward compatible
- PATCH: Bug fixes only

Previous versions (v3.x) used a different scheme.
All future versions follow SemVer strictly.
```

**README.md Section:**

```markdown
## Versioning

0-Core uses Semantic Versioning (SemVer) starting from v4.0.0:

- **MAJOR** (4.x.x): Breaking changes, major rewrites
- **MINOR** (x.1.x): New features, backward compatible
- **PATCH** (x.x.1): Bug fixes, no new features

Examples:

- v4.0.0: GPU Profiles (new major feature set)
- v4.1.0: Additional GPU features
- v4.1.1: Bug fix in GPU profiles
- v5.0.0: Complete Rust rewrite (breaking change)

**Note:** Versions prior to v4.0.0 used a different numbering scheme.
```

## Decision

Adopt Option C at v4.0.0.
Create intent to document this in README and CHANGELOG.

## Impact

- Clearer version meanings going forward
- Industry-standard compliance
- Better tooling support
- Easier for users to understand scope of updates

## Status Updates

### 2025-12-31

Initial decision captured.
Will implement documentation in v4.0.0 release.

### 2025-12-31

[Initial creation]

---

_Part of the 0-Core Intent Ledger_ 🌲
