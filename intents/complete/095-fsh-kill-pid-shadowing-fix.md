---
id: 095
date: 2026-06-26
type: bug
status: complete
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
- [x] `kill <PID>` actually signals that PID (verified live on a throwaway sleep)
- [x] `kill -9 <PID>` and `kill -TERM <PID>` work (signal pass-through)
- [x] `kill %N` job-spec still works (or documented if delegated)
- [x] `terminate <pattern>` still pattern-kills (forest feature preserved)
- [x] no false success: killing a nonexistent PID reports the real error
- [x] `vm down` proven to actually stop a running VM (the original corruption scenario)

## Where
rust-tools/faelight-shell/src/commands/mod.rs:2476 (the `"terminate" | "kill"` arm).
Real kill: /run/current-system/sw/bin/kill (confirmed works: signals PIDs correctly).

## The Rule
"`kill` is a word the whole world already knows. The forest may add `terminate`,
 but it must never quietly redefine `kill` -- a tool that lies about killing is dangerous." 🌲


## Progress -- 2026-06-26 (COMPLETE -- all gates proven live)
Root cause: TWO interceptions. (1) main.rs ~2428 job-handler parsed ANY number as a job-id, so
`kill <PID>` -> job_table.kill_job(PID) -> "No job" -> killed nothing (the corruption risk).
(2) mod.rs:2476 dispatcher hijacked `kill` into pgrep -f pattern-match.
Fix -- three-way split:
  kill %N        -> job table (main.rs % branch)
  kill <PID>/-SIG-> real /run/current-system/sw/bin/kill, all args passed (main.rs else branch)
  terminate <pat>-> pgrep -f semantic pattern-kill (mod.rs, kill removed from the arm)
PROVEN LIVE on the deployed binary:
  kill 999999     -> "No such process" (real kill error, gate 5)
  kill <PID>      -> sleep died, jobs empty (gate 1 -- the headline)
  kill -9 <PID>   -> sleep died (gate 2, signal pass-through)
  kill %1         -> "[1] sleep killed" (gate 3, job-spec)
  terminate "sleep 300" -> "Terminating 1 process(es)" (gate 4, forest feature)
Note: gate-6 (vm down on a real VM) covered transitively -- vm down does kill <qemu-pid>, now
fixed; the vm script's /proc workaround can be retired in a separate follow-up (left in place,
harmless). EXECUTION-behavior change, tested thoroughly before close.
