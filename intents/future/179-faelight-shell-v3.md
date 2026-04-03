---
id: 179
date: 2026-03-30
type: future
title: "faelight-shell v3 — The Daily Driver: The Shell Becomes Self-Aware"
status: in-progress
tags: [shell, fsh, daily-driver, self-aware, intelligence, zsh-retirement, v13]
version: 13.0.0
priority: high
depends_on: [146, 162, 171, 173, 174, 175, 176, 177]
spawned_by: 146
---

## The Vision
faelight-shell v2 proved the foundation:
structured pipelines, themes, persistence, completion, scripting.
Migration confidence: 70%.

v3 is the convergence — where every piece comes together
into a shell that is genuinely self-aware and ready to be
your only shell.

The four questions of a self-aware shell:
```
Why did this fail?      → INT-174 Structured Errors
What changed?           → INT-177 Shell Observability
What should I do next?  → Phase 28 Predictive Suggestions
What usually happens?   → INT-173 Command Registry + history
```

v3 answers all four.

## Prerequisites (must complete before v3 starts)
```
INT-162  Shell Architecture Hardening  — ExecContext, layer separation
INT-171  Pre-Command Decision Layer    — before_run hooks
```
Without INT-162, the intelligence features have no context to attach to.
Without INT-171, the safety guarantees are missing.

## Phase 19 — fsh as Login Shell
Register faelight-shell as a valid login shell:
```bash
echo "/home/christian/0-core/scripts/faelight-shell" | sudo tee -a /etc/shells
chsh -s /home/christian/0-core/scripts/faelight-shell
```
Read /etc/profile and ~/.profile on startup.
Compatible with greetd/faelight-login.

Gate: faelight-shell is the login shell. System boots into fsh.

## Phase 20b — Preexec Hook & Missing zsh Functions
Complete the zsh replacement:
```
preexec() intent-guard    → before_run in triggers.rs (INT-171)
git guardrail             → verify working, expand coverage
ya() cd-on-quit           → verify yazi integration correct
POSIX escape hatch        → sh { ... } for external scripts
```
Gate: Every zsh function replaced or intentionally omitted.

## Phase 28 — Predictive Suggestions (HIGH VALUE)
The single biggest daily driver win.
After every command, fsh suggests what usually comes next:
```
fsh ❯ fg commit "feat: ..."
💡 Usually followed by: d  (based on 47 sessions)
[enter] to accept  [esc] to skip
```

Data sources:
- shell_history command sequences
- session_patterns from state.db
- core predict next intent suggestion

Gate: After `fg commit`, fsh suggests `d` 80%+ of the time.

## Phase 29-32 — zsh Retirement
```
Phase 29: zsh fully optional — fsh handles 90%+ of daily commands
Phase 30: faelight-shell replaces zsh in all autostart configs
Phase 31: 95%+ Rust — only kernel interfaces remain in shell scripts
Phase 32: The forest is its own operating environment
```

Retirement criteria:
```
⬜ All Tier 1 + Tier 2 aliases in config.fsh
⬜ 90%+ of daily commands handled by fsh
⬜ No zsh dependency in any autostart script
⬜ faelight-login boots directly into fsh
⬜ 30 days of fsh as primary shell without issues
```

## Shell Intelligence Integration (INT-173-177)
These intents were created for v3. They build on ExecContext (INT-162):

### Command Registry (INT-173)
Unified registry of all commands — builtins, aliases, PATH binaries.
Enables: completion v3, safety rules, predict, self-documentation.

### Structured Errors (INT-174)
Every failure becomes a structured value with code, message, suggestion.
```
❌ E_NOT_GIT_REPO: Not a git repository
   💡 Run from ~/0-core or another git repo
```

### Script Debug Mode (INT-175)
```
run deploy.fsh --trace    # show each step with timing
run deploy.fsh --dry-run  # show without executing
```

### Failure Recovery (INT-176)
```
last_command retry        # re-run last failed command
last_command explain      # why did this fail?
history failures          # all failures this session
```

### Shell Observability (INT-177)
```
observe session           # what happened this session?
observe diff              # what changed vs last session?
observe anomalies         # what looks different from normal?
```

## The Self-Awareness Gate
v3 is complete when the shell can answer:
```
fsh ❯ why did last_command fail?
→ E_NOT_GIT_REPO: you were in ~/Downloads, not a git repo
→ Suggestion: cd ~/0-core first

fsh ❯ what changed this session?
→ 3 intents completed, 47 commits, health stable at 100%

fsh ❯ what should I do next?
→ Based on your patterns: run d after large commits

fsh ❯ what usually happens after fg commit?
→ 89% of the time: d (health check)
→ 67% of the time: gp (push)
```

## Migration Confidence Target
```
v2 complete:  70% daily driver
v3 target:   100% daily driver — zsh fully retired
```

## Gate Check
```
✅ INT-162 complete — ExecContext, layer separation (2026-03-30)
✅ INT-171 complete — before_run hooks (2026-03-30)
✅ Phase 19 — fsh registered as login shell — chsh complete (2026-04-03)
⬜ Phase 20b — all zsh functions replaced or deferred
⬜ Phase 28 — predictive suggestions working (80%+ accuracy)
⬜ INT-173 — Command Registry integrated
⬜ INT-174 — Structured Errors — every failure named
⬜ INT-175 — Script Debug Mode — --trace working
⬜ INT-176 — Failure Recovery — last_command retry/explain
⬜ INT-177 — Shell Observability — observe session/diff/anomalies
⬜ Phase 29 — zsh fully optional (90%+ daily commands in fsh)
⬜ Phase 30 — fsh replaces zsh in all autostart configs
⬜ Phase 31 — 95%+ Rust
⬜ Phase 32 — forest is its own operating environment
⬜ 30 days as primary shell without issues
⬜ core strategy jarvis score reflects shell intelligence gains
```

## The Phrase
**"v2 built the shell.
v3 makes it think.
The shell that knows why it failed,
what changed, and what comes next —
that is not a tool.
That is a partner."**

---
*"structured ✅  observable ✅  self-aware ✅
The three gates of a living shell.
v3 passes all three."* 🌲
