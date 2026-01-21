# faelight-git v2.1 - Git Governance Layer

**Philosophy:** Manual control over automation. Risk visible before action.

## What This Is

A policy layer and risk oracle for Git operations. Git remains Git - faelight-git decides whether and how safely.

## Architecture
```
faelight-git/
├── src/
│   ├── git/          # git2-rs operations (no shell commands)
│   │   └── repo.rs   # Repository state, status, upstream tracking
│   ├── risk/         # Git Risk Score engine
│   │   └── engine.rs # Risk calculation with explainable factors
│   ├── commands/     # CLI command implementations
│   │   ├── status.rs # Risk-aware status display
│   │   ├── risk.rs   # Detailed risk assessment
│   │   └── commit.rs # Intent-aware commit verification
│   ├── lib.rs        # Public API
│   └── main.rs       # CLI entry point + hook management
└── Cargo.toml
```

**Design Principles:**
- Pure Rust - no shell command dependencies
- Modular - each component testable in isolation
- Extensible - ready for intent/snapshot integration
- Type-safe - git2-rs provides compile-time guarantees

## Commands

### `faelight-git status` - Risk-Aware Status

Shows repository state with quantified risk score.
```bash
$ faelight-git status

🌲 Git Status — Faelight Forest
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Branch: main
Upstream: origin/main
Core Lock: 🔓 UNLOCKED

Working Tree:
  • Modified files: 0
  • Untracked files: 0

Commits:
  • Ahead: 0
  • Behind: 0

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Risk Score: 🟢 0 / 100
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### `faelight-git risk` - Detailed Risk Assessment

Breaks down risk factors with actionable recommendations.
```bash
$ faelight-git risk

⚠️  Git Risk Assessment
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Total Risk: 🟡 25/100
Band: Caution

Risk Factors:
  +10 Dirty tree: 3 modified files
   +5 Untracked files: 2 untracked files
  +10 Unpushed commits: 5 commits ahead

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💡 Moderate Risk - Suggestions:
  • Consider creating a snapshot
  • Verify changes with: git diff --cached
```

### `faelight-git commit` - Intent Verification

Pre-commit analysis with intent tracking.
```bash
$ faelight-git commit --intent INT-066

🔍 Pre-commit Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Modified files: 4
Untracked files: 0
Risk Score: 🟢 10/100
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Intent: INT-066

✅ Pre-commit checks passed

Next steps:
  1. Review changes: git diff --cached
  2. Commit with intent:
     git commit -m "<message>

Intent: INT-066"
```

## Git Risk Score (GRS)

**Range:** 0-100
- 🟢 0-20: Safe - proceed with confidence
- 🟡 21-50: Caution - review carefully
- 🔴 51-100: Dangerous - high risk of issues

**Current Risk Factors:**

| Factor | Score | Description |
|--------|-------|-------------|
| Dirty working tree | +10 | Modified files present |
| Untracked files | +5 | New files not in Git |
| Core locked | +10 | 0-core is immutable |
| No upstream | +5 | No remote tracking |
| Unpushed commits | +2/commit | Commits not pushed |
| Behind upstream | +5 | Upstream has new commits |

**Planned Factors (v2.2+):**
- No snapshot since last commit: +20
- No intent reference: +15
- core-diff risk multiplier: dynamic
- Uncommitted changes to system files: +10

## Git Hooks (v0.1 preserved)
```bash
faelight-git install-hooks  # Installs pre-commit, commit-msg, pre-push
faelight-git remove-hooks   # Removes hooks, restores backups
```

**Hook Enforcement:**

| Hook | Checks | Blocks? |
|------|--------|---------|
| pre-commit | Core lock status, gitleaks secret scan | Yes |
| commit-msg | Message length, intent suggestions | No |
| pre-push | Health check | No (warns only) |

## Why git2-rs?

**v0.1 (shell-based):**
```bash
git status --porcelain    # Parse text output
git log @{u}.. --oneline  # Count unpushed commits
```

**v2.x (git2-rs):**
```rust
let repo = Repository::open(".")?;
let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;
```

**Benefits:**
1. **Type safety** - Errors caught at compile time
2. **Performance** - Native operations vs process spawning
3. **Cross-platform** - No shell dependencies
4. **Testability** - Mock repositories, unit tests
5. **Reliability** - Deterministic behavior

## Integration Points

**Current:**
- ✅ `faelight-core` - Shared utilities
- ✅ `gitleaks` - Secret scanning
- ✅ `dot-doctor` - Health checks

**Planned:**
- [ ] `intent` - Intent reference validation
- [ ] `faelight-snapshot` - Snapshot awareness
- [ ] `core-diff` - Semantic risk multipliers

## Roadmap

**v2.1** (Current)
- ✅ git2-rs integration
- ✅ Risk scoring engine
- ✅ Intent verification
- ✅ Production-ready hooks

**v2.2** (Q1 2026)
- [ ] Snapshot integration
- [ ] Intent enforcement in hooks
- [ ] core-diff risk multipliers

**v3.0** (Q2 2026)
- [ ] Provenance graph (Intent → Commit → Snapshot)
- [ ] Push command with risk gates
- [ ] Automated rollback on high-risk pushes
- [ ] Graph query language

## Design Principles

1. **Governance, not convenience** - Make the right thing explicit
2. **Risk visible before action** - No surprises
3. **Git remains Git** - No abstraction leakage, works with raw Git
4. **Composable** - Integrates with intent/snapshot/diff systems
5. **Explainable** - Every score has traceable reasoning

## Development
```bash
# Build
cargo build --release -p faelight-git

# Test
cargo test -p faelight-git

# Install
cp target/release/faelight-git ~/0-core/scripts/
```

---

**Built for Faelight Forest 0-Core**  
Part of the 30-tool Rust ecosystem  
Presented to Linux kernel developers, January 2026
