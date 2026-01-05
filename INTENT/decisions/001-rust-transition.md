---
id: 001
date: 2025-12-31
type: decision
title: "Complete Rust Transition - v5.0 onwards"
status: planned
tags: [rust, architecture, security, philosophy]
---

## The Insight

**We are not outgrowing Bash because it's bad.**  
**We outgrew it because the system demands guarantees.**  
**We are not just switching languages—we are formalizing a philosophy into code.**

## Why Rust

### The Need

0-Core evolved from dotfiles → configuration framework → philosophy-driven system.

As the system matured, we needed:

- **Memory safety** (security-critical tools like core-protect)
- **Type safety** (catch bugs at compile-time, not runtime)
- **Guarantees** (immutability, thread safety, error handling)
- **Performance** (compiled efficiency without C's danger)

Bash served us perfectly for rapid prototyping and experimentation.  
Rust serves us for production-grade, security-critical systems.

### The Philosophy Alignment

Rust embodies 0-Core principles:

- **Manual control** (explicit is better than implicit)
- **Intentionality** (no hidden behavior, everything is declared)
- **Understanding** (type system teaches you, doesn't hide complexity)
- **Safety** (prevents entire classes of bugs at compile-time)

## The Shared Core (v5.1.0+)

Once 2-3 Rust tools exist, consolidate shared logic:

```
0-core/
├── corelib/              # Shared Rust crate
│   ├── src/
│   │   ├── lib.rs
│   │   ├── fs.rs         # File system operations
│   │   ├── exec.rs       # Command execution
│   │   ├── config.rs     # Config parsing
│   │   ├── errors.rs     # Error types
│   │   └── output.rs     # Colored output
│   └── Cargo.toml
├── core-diff/
│   └── main.rs → uses corelib
├── dot-doctor/
│   └── main.rs → uses corelib
└── ...
```

**Benefits:**

- Consistent behavior across all tools
- Less duplicated logic
- Easier auditing for system-critical tools
- Mirrors ripgrep/fd structure

## The Psychological Win

I am not learning Rust to "be modern."
I am learning Rust to **delete Bash with confidence.**

Replacing:

- Implicit behavior → explicit models
- Stringly-typed logic → enums
- Runtime surprises → compile-time errors

A system getting stronger over time.

### Timeline

**v3.5.x - v4.x (Q4 2025 - Q1 2026): Foundation (Bash)**

- v3.5.0 - Intent Ledger
- v3.5.1 - Git Guardrails
- v3.5.2 - Shell Safety & Polish
- v4.0.0 - GPU Profiles (Omega legacy)

**v5.0.0 (Q2 2026): First Rust**

- Start with small CLI rewrites (bump-system-version, core-status)
- Learn Rust fundamentals on real tools
- faelight-bar starts development (parallel project)
- Coexists with bar-waybar during development

**v5.1.0 (Q3 2026): THE CUT-OVER**

- **COMPLETE REWRITE** - All scripts in Rust
- **NO BASH FALLBACKS** - Clean break
- **ONE SYSTEM** - Rust only

### What Gets Rewritten in v5.1.0

```
scripts/
├── bump-system-version → Rust (first - easiest, learn basics)
├── core-status       → Rust (simple)
├── dotctl            → Rust (medium)
├── intent            → Rust (medium)
├── profile           → Rust (medium, created in v4.0.0)
├── core-diff         → Rust (medium)
├── dot-doctor        → Rust (medium-hard, most checks)
├── core-protect      → Rust (hard, security-critical)
├── safe-update       → Rust (hard, system-critical)

Bash versions: DELETED
```

```

bar-waybar/ → REMOVED completely
bar-faelight/ → ONLY bar (Rust)

**Result:** 100% Rust tooling. No dual systems. Clean codebase.

## Alternatives Considered

### Option A: Keep Bash Forever

**Pros:** Works, familiar, fast to write
**Cons:** No memory safety, no type safety, hard to maintain as complexity grows
**Rejected:** System demands guarantees Bash can't provide

### Option B: Mixed Bash/Rust with Fallbacks

**Pros:** Safer transition, users can choose
**Cons:** Two versions of same tool = confusion and mistakes
**Rejected:** Violates philosophy (one way, not two)

### Option C: Complete Cut-Over (CHOSEN)

**Pros:** Clean, intentional, aligns with philosophy, forces quality
**Cons:** Higher risk, must get it right, no safety net
**Chosen:** All-in or not at all. One system. One language. One way.

## Risks & Mitigation

### Risks

1. **Learning curve** - Rust is complex, could delay v5.0
2. **Breaking changes** - v5.1.0 is a major version bump
3. **Quality issues** - Rewrite bugs could break tools
4. **User adoption** - Some may not want the change

### Mitigation

1. **6-month learning period** (Jan-Jun 2026)
   - Rustlings exercises
   - Rewrite bump-system-version (first real project)
   - Rewrite core-status, dotctl, intent
   - Build faelight-bar as advanced project
   - Wayland protocol study (for faelight-bar)

2. **Extensive testing before v5.1.0**
   - Beta period
   - Test suite for all tools
   - Migration guide
   - Clear changelog

3. **Clear communication**
   - v4.0.0 announcement (README + blog)
   - v5.0.0 proves Rust works (faelight-bar)
   - v5.1.0 migration guide
   - Support for issues

4. **Version semantics**
   - v5.1.0 = MAJOR VERSION
   - Breaking changes expected
   - Users warned in advance

## Impact

### Positive

- **Security** - Memory-safe tools (critical for core-protect, safe-update)
- **Reliability** - Type safety catches bugs at compile-time
- **Performance** - Faster execution, lower resource usage
- **Maintainability** - Better error handling, clearer code structure
- **Philosophy** - Code formalizes principles

### Negative

- **Transition effort** - 3-6 months of work
- **Learning curve** - Rust is harder than Bash
- **Breaking changes** - v5.1.0 requires user updates
- **Dependency on Rust ecosystem** - Compiler, cargo, crates

## Success Criteria

**v5.0.0 Success:**

- faelight-bar runs stable
- No crashes, memory leaks
- Feature parity with Waybar (for our needs)
- Users can daily-drive it

**v5.1.0 Success:**

- All tools rewritten and tested
- No functionality regressions
- Performance equal or better
- Clean codebase (no bash remnants)
- Users successfully migrate

## The Philosophy

This isn't about Rust being "better than Bash."

This is about **0-Core evolving beyond what Bash can promise.**

We're not switching languages.
**We're formalizing a philosophy into code.**

---

_The forest demands guarantees. Rust provides them._ 🌲
```
