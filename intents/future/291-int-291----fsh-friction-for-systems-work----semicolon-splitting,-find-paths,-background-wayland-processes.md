---
id: 291
title: "fsh friction for systems work -- semicolon splitting, find paths, background Wayland processes"
status: planned
date: 2026-05-12
tags: [fsh, shell, bug, friction, wayland, find, semicolon, background]
---

Discovered during INT-286 (faelight-term v3) development.
These friction points slow down systems-level work.
Each is small alone. Together they add up to real friction.

---

## BUG 1 -- Semicolon command splitting

Symptom:
  command1; command2
  fsh splits this into two separate commands incorrectly in some contexts.
  The second command runs but loses context of the first.

Workaround:
  Write to /tmp/script.sh and run with sh /tmp/script.sh

Example that failed:
  /home/christian/0-core/target/debug/faelight-term-v3 2>/tmp/v3.log; tail -20 /tmp/v3.log
  fsh ran these as two disconnected commands, 
  the binary ran but the tail showed nothing.

Fix needed:
  Semicolon chains should work the same as && chains.
  If && works, ; should work.
  Review split_semicolons() -- it was fixed for for/while but may still
  have edge cases with process substitution and redirects.

---

## BUG 2 -- find command with paths outside forest root

Symptom:
  find /tmp/some-directory -name "*.rs" | sort
  Returns no results even when files exist.

Verified:
  ls /tmp/cosmic-term-study/src/ -- works, shows files
  find /tmp/cosmic-term-study/src -name "*.rs" -- returns nothing

Workaround:
  Use ls instead of find for directory exploration.
  Or drop to zsh for find operations.

Fix needed:
  find is likely being intercepted or the path is being rewritten.
  Check if fsh has any path normalization that strips /tmp paths.
  Or check if find is aliased to something that filters output.

---

## BUG 3 -- Background Wayland processes from inside faelight-term

Symptom:
  Running a Wayland GUI app (&) from inside faelight-term causes
  blocking_dispatch to return immediately instead of blocking.
  The window never appears in Niri.
  Running the same binary from foot works correctly.

Root cause (suspected):
  faelight-term uses a PTY. The PTY interacts with the Wayland event loop
  in the child process. blocking_dispatch may be detecting a non-socket
  fd and returning early.

Workaround:
  Launch Wayland GUI apps from foot or zsh directly.
  Or launch with niri msg action spawn -- which is compositor-level.

Fix needed (long term):
  Investigate how faelight-term's PTY setup affects child process
  Wayland connections. May need to unset/reset certain file descriptors
  before execing child processes.
  Alternatively: document this as expected behavior (nested Wayland
  clients have limitations) and note it in faelight-term docs.

---

## COSMIC NOTE -- Shell features needed for terminal development

From INT-286 development experience, fsh needs these for smooth
systems/compositor work:

1. Process substitution: $(command) in more contexts
2. Background job output isolation: & jobs shouldn't pollute terminal 
   unless explicitly redirected
3. niri msg integration: maybe a built-in or alias for common niri commands
4. /tmp path handling: treat /tmp as a fully accessible path, not special

---

## GATES

[ ] BUG 1: semicolon chains work same as && chains
[ ] BUG 2: find /tmp/... returns correct results
[ ] BUG 3: documented in faelight-term -- known limitation with note
[ ] fsh can be used for full INT-286 Phase 3+ development without /tmp workarounds
[ ] No need to drop to zsh for systems work

---

"The shell is the forest mouth.
If it cannot speak clearly during construction,
it cannot speak clearly ever.
Fix the friction before it becomes habit." 🌲
