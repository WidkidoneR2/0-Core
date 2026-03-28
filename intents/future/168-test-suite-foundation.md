---
id: 168
date: 2026-03-28
type: future
title: "Test Suite Foundation — 73K Lines of Rust Deserves Tests"
status: planned
tags: [testing, reliability, rust, cargo-test, integrity, v12]
version: 12.0.0
priority: high
---

## The Problem
73,398 lines of Rust. Zero automated tests.

Every change is verified by:
1. Cargo builds without error
2. Running `d` manually
3. Checking the output looks right

This works now because:
- One developer who knows the system deeply
- Changes are incremental and focused
- Doctor catches regressions visually

This will break when:
- v12 Strategy adds complex reasoning logic
- Multiple domains interact in unexpected ways
- A refactor changes behavior silently
- fsh pipeline semantics change subtly

## What Needs Testing

### Priority 1 — Core Domain Unit Tests
Each domain should have tests for its primary functions:
```rust
// engine/src/domains/predict/tests.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_sessions_returns_data() { }

    #[test]
    fn test_health_trajectory_empty_db() { }

    #[test]
    fn test_intent_velocity_counts_correctly() { }

    #[test]
    fn test_coupling_filters_headers() { }
}
```

### Priority 2 — fsh Pipeline Tests
The pipeline operators must behave consistently:
```rust
// faelight-shell/src/tests/pipeline.rs
#[test]
fn test_first_operator() {
    // gc | first 5 → returns exactly 5 rows
}

#[test]
fn test_where_operator() {
    // ps | where cpu > 50 → only rows where cpu > 50
}

#[test]
fn test_sort_operator() {
    // data | sort name asc → sorted correctly
}
```

### Priority 3 — Stress Test Integration
Integrate INT-152 stress tests into cargo test:
```rust
#[test]
fn test_event_storm_no_corruption() {
    // 500 events, verify count, cleanup
}

#[test]
fn test_prediction_under_load() {
    // all 9 predict commands run without panic
}
```

### Priority 4 — Doctor Health Checks
Each doctor check should be independently testable:
```rust
#[test]
fn test_stow_symlinks_check() { }

#[test]
fn test_intent_ledger_check() { }

#[test]
fn test_schema_validation_check() { }
```

## The Testing Philosophy
Tests should be:
- **Fast** — under 5 seconds total
- **Isolated** — use temp db, not production state.db
- **Honest** — test real behavior, not implementation details
- **Minimal** — one test per behavior, not exhaustive coverage

We are NOT aiming for 100% coverage.
We are aiming for: "no silent regressions."

## Phase 1 — Test Infrastructure (1 session)
Set up the test harness:
```rust
// engine/src/test_utils.rs
pub fn test_context() -> AppContext {
    // Creates isolated in-memory SQLite db
    // Temp directory for file operations
    // No production state reads
}
```

Add to Cargo.toml:
```toml
[dev-dependencies]
tempdir = "0.3"
```

## Phase 2 — Core Domain Tests (2-3 sessions)
Write tests for highest-risk domains:
- predict (9 commands, complex logic)
- doctor (24 checks, health calculation)
- reaction (cooldown logic, rule evaluation)
- stress (verify stress tests pass in CI)

## Phase 3 — fsh Pipeline Tests (1-2 sessions)
Test all pipeline operators with known inputs/outputs.
Schema validation tests once schemas are defined (INT-162).

## Phase 4 — CI Integration
```bash
# Run on every commit via git hook:
cargo test --workspace 2>&1 | grep -E "FAILED|passed|test result"
```

Add to faelight-git pre-commit hook:
Tests must pass before commit is allowed.

## Gate Check
```
⬜ test_utils.rs — isolated test context
⬜ predict domain — 9 tests (one per command)
⬜ doctor domain — health % calculation tested
⬜ reaction domain — cooldown logic tested
⬜ stress tests — integrated into cargo test
⬜ fsh pipeline — first/where/sort/select tested
⬜ cargo test passes with 0 failures
⬜ Pre-commit hook runs tests
⬜ Test run < 30 seconds
```

## The Phrase
**"The forest that tests itself
discovers failures in the forest,
not in the field.
Tests are not overhead.
They are the forest checking its own roots."**

---
*"73,398 lines of Rust.
Every line has been tested manually.
None have been tested automatically.
That changes now."* 🌲
