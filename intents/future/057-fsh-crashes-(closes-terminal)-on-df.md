---
id: 057
date: 2026-06-12
type: fix
title: "fsh crashes (closes terminal) on df"
status: planned
tags: [fix, bugfix, fsh, stability, crash, int-056]
version: TBD
---
## Vision
fsh runs stock external commands without ever crashing. Concretely, `df`
(and anything in the same class) executes cleanly instead of taking the
terminal down with it.

## Why Now
`df -h /nix` closes the terminal every single time it is run from fsh --
reproducible, and it already cost a live working session. A stock coreutils
command killing the shell is a daily-driver stability defect and a direct
blocker on the 1.0.0 "all Arch-era commands work" parity bar. It also sits
beside INT-056 (Forest Recovery Protocol) -- shell stability from a
different angle.

Observed:
- `df -h /nix` -> terminal closes immediately, every run.
- `lsblk` and other commands are unaffected.
- The window dying == the fsh process exiting, so fsh is almost certainly
  panicking rather than df failing (df is a plain coreutils binary).

## Approach
Hypothesis: fsh panics while handling `df` -- on command dispatch, argument
parsing, or while processing the output of df -- and the panic exits the
process, taking the terminal with it.

Isolation (non-destructive -- do NOT re-run df in the live terminal; each
run kills it):
- Run fsh as a child of bash so the parent survives the crash:
  `RUST_BACKTRACE=1 faelight-shell` from inside bash, type `df`, then read
  the backtrace in the bash scrollback after fsh dies.
- Narrow the trigger: `df` vs `df -h` vs `df /nix` vs `df -h /nix` --
  isolates command-dispatch vs arg-parsing vs path-handling.
- Check whether fsh writes any panic or crash log.

Then fix the offending path and audit dispatch for sibling
panic-on-external-command cases.

## Success Criteria
- [ ] Root cause identified (backtrace captured)
- [ ] Trigger narrowed (which df form crashes, and why)
- [ ] Fix: fsh runs df without panicking
- [ ] Regression: df, df -h, df /nix, df -h /nix all run cleanly in fsh
- [ ] Dispatch audited for similar panic-on-external-command cases

## Gate Check
⬜ Not started
---
*"The forest grows with intention."* 🌲

## Diagnosis - 2026-06-13 (execution cleared, crash localized to interactive editor)

Status: root cause LOCALIZED, not yet fixed. This is not a one-line fix.

### Proven innocent (df executes correctly)
- `env RUST_BACKTRACE=1 /run/current-system/sw/bin/faelight-shell -c 'df'` runs CLEAN: full df output, exit 0, empty panic log.
- Therefore `execute()`, the whole `match cmd.as_str()` dispatch, and `run_external()` are all cleared.
- `run_external` runs `sh -c <line>` with inherited stdio via `.status()` (spawn + wait). df survives it fine.

### Crash is INTERACTIVE-ONLY
- The panic lives in the REPL / line-editor layer that `-c` mode bypasses -- `main.rs` or the editor module, NOT `commands/mod.rs`.
- A panic there kills fsh; since fsh is the login shell, the terminal closes. Likely the same root as the nested-fsh `exit`-closes-terminal symptom.

### Cleared this session (commands/mod.rs)
- `highlight_rust_line` (line 76) and `colorize_line` (line 111): both safe for "df" (slices guarded by short-circuits; "df" falls through to plain output). These color OUTPUT/Rust lines, not interactive input.
- `reload` builtin (~line 468) and `exec_cmd` (line 9216): both use `.exec()` INTENTIONALLY to restart the shell. Not the bug -- do not touch.
- disk-usage `df` spawn at line 11121 (inside a dashboard fn): guarded by `parts.len() >= 5`, only runs on the dashboard command. Not the bug.

### Remaining suspect
- The input line-editor wiring in `main.rs` / editor module: the `Highlighter` / `Hinter` / `Completer` impl, or the per-keystroke render path.
- Friday interactive hinter is LOWER priority (observed firing on input without crashing).

### Next-session opener (the clean kill)
1. Launch `faelight-shell` FROM a bash login (not as the shell) so a panic drops back to bash instead of closing the terminal: `RUST_BACKTRACE=1 faelight-shell`, then type `df`.
2. The backtrace names the exact `main.rs` line. Fix is almost certainly a bounds / None guard there.
3. Regression baseline after the fix: `faelight-shell -c 'df'` must still run clean.

### Repro facts
- Daily-driver binary: `/run/current-system/sw/bin/faelight-shell` (NixOS system path).
- `-c '<cmd>'` non-interactive mode works and is the safe way to test execution paths without risking the terminal.
