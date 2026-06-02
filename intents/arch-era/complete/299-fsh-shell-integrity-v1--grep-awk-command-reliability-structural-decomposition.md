---
id: 299
title: "fsh Shell Integrity v1 -- grep, awk, command reliability, structural decomposition"
status: complete
date: 2026-05-12
tags: [fsh, shell, integrity, grep, awk, command, structure]
---
## The Problem

fsh is supposed to be the forest mouth.
But right now, every command is a question.
grep shows nothing. awk behaves differently than zsh.
copy and vocabulary words are unreliable.
The shell intercepts commands it shouldn't, silently drops output,
and the 3600-line monolith makes every bug invisible.

A shell you have to question is not a daily driver.
A shell you cannot trust is not a foundation for a summer presentation.
This intent fixes the integrity of fsh -- systematically, completely, permanently.

---

## ROOT CAUSES

### 1. Command interception gone wrong
fsh "enhances" commands (grep, cat, ls) via builtins and aliases.
When those enhancements have bugs, output disappears silently.
grep in fsh: the forest-enhanced version may swallow output in
certain execution contexts (&&, pipelines, sh-c delegation).

### 2. The 3600-line monolith
main.rs is one file. execute() is 1100+ lines.
When a bug exists in command dispatch, finding it requires
reading hundreds of lines of nested control flow.
Every fix risks breaking something else because nothing is isolated.

### 3. continue 'repl vs 'segments confusion
Known issue from INT-291: some continue 'repl calls inside the
'segments loop break remaining segments in a command.
Not all instances were found and fixed. Commands silently stop.

### 4. No -c flag
fsh cannot be invoked as: fsh -c "command"
This means scripts, tests, and non-interactive use are broken.
The test suite in INT-298 had to work around this with piped stdin.

### 5. SIGPIPE panic
ls ~/path | head -5 panics fsh with "broken pipe".
A shell must never panic on standard Unix pipe patterns.

### 6. Vocabulary words not audited
Human-first vocabulary (delete, find, write, read, copy, move)
may conflict with external commands or have broken dispatch paths.
Needs a systematic audit.

---

## THE WORK

### Phase 1 -- Command Reliability (daily driver blockers)

GREP:
  - Audit fsh grep builtin: where does output go?
  - grep in && chains must work (currently routes to sh-c, loses enhancement)
  - grep -r must work natively
  - grep with multiple patterns must work
  - Fix: if grep enhancement is broken in context, fall back cleanly to /usr/bin/grep

AWK:
  - Verify awk passthrough works in all contexts
  - awk '{print $N}' patterns -- test and confirm
  - awk in pipelines -- test and confirm

COPY / VOCABULARY WORDS:
  - Audit all vocabulary words: delete, find, write, read, copy, move, list
  - Each word: does it dispatch correctly? Does output appear?
  - Does it conflict with external commands of the same name?
  - Fix any broken dispatch paths

### Phase 2 -- Structural Fixes

SIGPIPE:
  - Add signal::set(Signal::SIGPIPE, SigHandler::SigDfl) at startup
  - ls ~/path | head -5 must never panic
  - Any pipe truncation must exit cleanly, not crash

-c FLAG:
  - Add fsh -c "command" support to main()
  - Parse argv[1] == "-c", take argv[2] as command string
  - Execute it non-interactively, exit with command exit code
  - This enables: fsh -c "echo hello" == "hello"
  - This fixes the test suite piped stdin workaround

CONTINUE LABEL AUDIT:
  - Find every continue 'repl inside the 'segments for loop
  - Verify: should it be continue 'repl or continue 'segments?
  - continue 'repl breaks all remaining segments (usually wrong)
  - Fix every incorrect label
  - Document: which continue 'repl calls are intentional?

### Phase 3 -- Structural Decomposition (make bugs visible)

Extract from main.rs into focused modules:
  - src/expand.rs    -- tilde, glob, variable, subshell expansion
  - src/dispatch.rs  -- command routing logic
  - src/pipe.rs      -- pipe chain execution
  - src/heredoc.rs   -- heredoc collection and dispatch
  - src/builtin/     -- one file per builtin command

Target: main.rs under 800 lines (orchestration only)
Target: no single function over 200 lines

---

## SUCCESS CRITERIA (GATES)

Phase 1:
- [x] grep pattern file shows output in fsh (matching zsh behavior) — verified + test added 2026-05-13
- [x] grep -r works in fsh without falling to sh — verified + test added 2026-05-13
- [x] grep in && chains shows output — verified + test added 2026-05-13
- [x] awk '{print $1}' works in pipelines — verified + test added 2026-05-13
- [x] copy (vocabulary) dispatches correctly — verified 2026-05-12
- [x] All vocabulary words audited -- each has a test — 10/10 words: delete, find, write, read, copy, move, list, gt, db, it — 2026-05-12

Phase 2:
- [x] fsh -c "echo hello" outputs "hello" and exits 0 — implemented in main() + builtin, works inside and outside fsh 2026-05-13
- [x] ls ~/path | head -5 does not panic — SIGPIPE fixed 2026-05-12
- [x] All continue 'repl inside 'segments loop audited and labeled — 1 bug found+fixed (INT-265 forest pipeline, line 2307) 2026-05-12
- [x] SIGPIPE handled silently -- no panic on truncated pipes — 2026-05-12

Phase 3:
- [ ] main.rs under 800 lines
- [x] expand.rs extracted — 13 functions, 641 lines (expand_globs, split_logical, subshells, heredoc, is_complete_command + more) 2026-05-13
- [ ] No single function over 200 lines
- [x] All 75 fsh_audit.sh tests pass after decomposition — warmup fixed, deterministic 2026-05-13
- [x] fsh_audit.sh expanded to 75 tests — grep, awk, fsh -c, vocabulary, structural 2026-05-13

---

## THE TEST STANDARD

Before any gate is marked complete:
"Has this been demonstrated in fsh, not just implemented?"

Every fix gets a test in fsh_audit.sh.
No gate closes without a passing test.
The shell is the daily driver. Test it like one.

---

## THE STANDARD

"The forest mouth must speak clearly.
Every command must do exactly what it says.
No silent failures. No swallowed output.
No shell that makes you question your own typing.

The shell is the foundation.
If the foundation is uncertain,
everything built on it is uncertain.

Fix the foundation first.
The forest does not ship uncertain tools." 🌲