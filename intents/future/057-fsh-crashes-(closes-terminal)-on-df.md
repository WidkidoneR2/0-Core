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
