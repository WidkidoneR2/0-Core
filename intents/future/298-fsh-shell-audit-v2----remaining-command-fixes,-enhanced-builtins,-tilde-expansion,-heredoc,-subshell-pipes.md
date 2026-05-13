---
id: 298
title: "fsh shell audit v2 -- remaining command fixes, enhanced builtins, tilde expansion, heredoc, subshell pipes"
status: in-progress
date: 2026-05-12
tags: [fsh, shell, audit, stability, builtins, tilde, heredoc, subshell]
priority: critical
depends_on: [291, 261]
---

## The Problem

fsh keeps breaking in ways that make daily use painful.
INT-291 fixed the most visible bugs, but the shell is still not ready
to replace zsh/nu as a daily driver. Every session reveals new regressions.

The root cause: fsh's main.rs is 3600+ lines of nested control flow with
systemic `continue 'repl` vs `continue 'segments` confusion, no test suite,
and fragile string parsing that breaks under real-world usage.

This intent takes a more systematic approach: audit, test, fix, verify.
No zsh/nu removal until fsh passes every item in this intent.

---

## KNOWN BROKEN (from audit 2026-05-12)

### Critical -- daily driver blockers

BUG 1: Tilde expansion in commands
  ls ~/0-core/intents/future/  -- returns "No such file or directory"
  Tilde not expanded for arguments to external commands in some contexts.
  Works standalone (cd ~/) but breaks for external command args.

BUG 2: Heredoc not working
  cat <<EOF -- hangs or does nothing in fsh
  cat > /tmp/file << 'EOF' -- Ctrl+C required to escape
  Fix: detect heredoc syntax and pass to sh -c

BUG 3: $(cmd | pipe) in assignments
  RESULT=$(ls /tmp | wc -l) -- | inside $() treated as pipe separator
  The expand_subshells function doesn't handle pipes inside $()
  Fix: make subshell expander pipe-aware

BUG 4: cat >> file intercepted by bat
  cat file >> other_file -- bat intercepts, shows "IO circle" error
  When cat is used with redirects, should use real /usr/bin/cat not bat
  Fix: detect cat + redirect and pass to real cat

BUG 5: ls ~/path inconsistent
  ls ~/0-core works but ls ~/0-core/subdir sometimes fails
  Tilde expansion not uniform across all command argument positions

---

## ENHANCED BUILTINS NEEDED

### grep
  Current: forest-enhanced (line numbers, match count) -- good
  Missing: grep -r (recursive) should work without falling to sh
  Missing: grep with multiple patterns
  Note: grep in && chains runs via sh-c, loses forest enhancement

### ls
  Current: uses eza/exa but some output formatting issues
  Missing: ls ~/path tilde expansion
  Missing: ls with sort flags (-t, -S) passing through correctly

### find (forest)
  Current: @shortcuts work, Unix passthrough works
  Enhancement: @term, @config, @engine shortcuts needed
  Enhancement: find output should optionally show file sizes

### awk / sed
  Current: pass-through to system awk/sed
  Enhancement: awk '{print $N}' patterns work, keep as-is
  No changes needed -- just verify they work

### cat
  Current: bat-enhanced for reading
  Bug: intercepted for redirects (should NOT intercept cat with > or >>)
  Fix: if cat has a redirect target or >> operator, bypass bat

### fsearch
  Current: content search via ripgrep
  Enhancement: fsearch should show context lines (-C 2)
  Enhancement: fsearch --rust should search only .rs files

---

## STRUCTURAL FIXES NEEDED

### Audit all continue 'repl inside 'segments loop
  Every continue 'repl inside the segments for loop breaks remaining segments.
  We fixed the known ones but there may be more.
  Fix: write a script that finds all continue 'repl in the file, identifies
  which ones are inside the 'segments loop, and verifies they use 'segments.

### Shell test suite (INT-298 gate requirement)
  Create a test file: 0-core/tests/fsh_audit.sh
  Each line tests one behavior, exits non-zero on failure
  Run as part of deploy check for faelight-shell
  Gate: 50 tests passing before any zsh/nu removal

### continue 'segments audit
  Run: grep -n "continue 'repl" main.rs | awk 'NR > SEGMENTS_LINE'
  Verify: all continue 'repl calls inside 'segments loop use 'segments

---

## REMAINING FROM INT-291 (not fixed)

BUG 3 from INT-291: Background Wayland processes from inside faelight-term
  Running GUI apps (&) from faelight-term doesn't work.
  Status: documented as PTY limitation. Workaround: launch from Niri.
  No fix planned -- this is expected PTY behavior.

---

## SUCCESS CRITERIA (GATES)

- [ ] ls ~/path works consistently for all paths
- [ ] cat <<EOF heredoc passes to sh -c correctly
- [ ] RESULT=$(cmd | pipe) works (pipe-aware subshell expansion)
- [ ] cat >> file uses real cat, not bat
- [ ] Shell test suite: 50 passing tests
- [ ] All continue 'repl inside 'segments loop audited and fixed
- [ ] grep, ls, find, cat, fsearch enhanced as described
- [ ] fsh used as daily driver for 1 full week without dropping to zsh
- [ ] No regressions from INT-291 fixes

## The Standard

"The shell is the forest mouth.
If it cannot speak clearly during construction,
it cannot speak clearly ever.
Fix the friction before it becomes habit.
The forest does not ship broken tools." 🌲
