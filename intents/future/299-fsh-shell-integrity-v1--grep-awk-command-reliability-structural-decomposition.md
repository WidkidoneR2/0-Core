📋  299 — fsh Shell Integrity v1 — grep, awk, command reliability, structural decomposition
  Status: [planned]  Date: 2026-05-12
  Tags: fsh, shell, integrity, grep, awk, command, structure
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
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
- [ ] grep pattern file shows output in fsh (matching zsh behavior)
- [ ] grep -r works in fsh without falling to sh
- [ ] grep in && chains shows output
- [ ] awk '{print $1}' works in pipelines
- [ ] copy (vocabulary) dispatches correctly
- [ ] All vocabulary words audited -- each has a test

Phase 2:
- [ ] fsh -c "echo hello" outputs "hello" and exits 0
- [ ] ls ~/path | head -5 does not panic
- [ ] All continue 'repl inside 'segments loop audited and labeled
- [ ] SIGPIPE handled silently -- no panic on truncated pipes

Phase 3:
- [ ] main.rs under 800 lines
- [ ] expand.rs extracted with tilde/glob/var/subshell
- [ ] No single function over 200 lines
- [ ] All 50 fsh_audit.sh tests still pass after decomposition
- [ ] fsh_audit.sh expanded to 75 tests covering new behaviors

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
