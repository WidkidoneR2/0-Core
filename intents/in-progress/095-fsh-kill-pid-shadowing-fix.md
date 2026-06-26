---
id: 095
date: 2026-06-26
type: bug
status: in-progress
title: "fsh: kill hijacked to pattern-match -- kill <PID> does not signal that PID"
tags: [fsh, shell, kill, safety, corruption-risk, lane-0, stability]
priority: high
---
## Why (corruption risk -- roadmap's top Lane 0 bug)
fsh's `kill` builtin (commands/mod.rs:2476, `"terminate" | "kill" if !args.is_empty()`) does
NOT signal a PID. It runs `pgrep -f '<args>'`, treating the argument as a COMMAND-LINE PATTERN,
then kills matches. So `kill 30360` runs `pgrep -f '30360'` -- which matches processes whose
command line CONTAINS "30360" (almost never the PID's own process) -- and kills nothing, or the
WRONG process. Proven live 2026-06-26: `kill 30360` on a real `sleep 300` (PID 30360) -> "No job",
process still running 47s later. This silently broke `vm down`'s `kill <qemu-pid>` (2026-06-23)
-> no-op -> two VMs on one qcow2 (DISK CORRUPTION RISK). The vm script carries a Python /proc
walker workaround (pkgs/faelight/scripts/vm:30-47) precisely because of this.

## What (split kill from terminate)
`kill` must do what every Unix user expects: signal a PID/job. `terminate <pattern>` keeps the
forest's intentional semantic pattern-kill. Split the one hijacked handler into two:
- `kill <PID>` / `kill -SIG <PID>` / `kill -9 <PID>` / `kill %job` -> pass through to the real
  /run/current-system/sw/bin/kill with all args. Standard, safe, predictable.
- `terminate <pattern>` -> the CURRENT pgrep -f pattern-match behavior (kept, intentional).

## Scope (EXECUTION-behavior change -- higher stakes than INT-089)
This changes what `kill` DOES (not just its error text). kill is the most safety-critical
builtin. Test all paths before+after deploy. Do NOT remove the vm /proc workaround in this
intent -- retire it separately once kill is proven (its own small follow-up).

## Gates
- [ ] `kill <PID>` actually signals that PID (verified live on a throwaway sleep)
- [ ] `kill -9 <PID>` and `kill -TERM <PID>` work (signal pass-through)
- [ ] `kill %N` job-spec still works (or documented if delegated)
- [ ] `terminate <pattern>` still pattern-kills (forest feature preserved)
- [ ] no false success: killing a nonexistent PID reports the real error
- [ ] `vm down` proven to actually stop a running VM (the original corruption scenario)

## Where
rust-tools/faelight-shell/src/commands/mod.rs:2476 (the `"terminate" | "kill"` arm).
Real kill: /run/current-system/sw/bin/kill (confirmed works: signals PIDs correctly).

## The Rule
"`kill` is a word the whole world already knows. The forest may add `terminate`,
 but it must never quietly redefine `kill` -- a tool that lies about killing is dangerous." 🌲
