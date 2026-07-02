---
id: 096
date: 2026-06-26
type: bug
status: complete
title: "fsh reload: identify the new build (stop blind re-exec)"
tags: [fsh, shell, reload, deploy, ux, lane-0, stability]
priority: medium
---
## Why
`reload` re-execs into the deployed fsh (works), but its same-binary guard never fired -- it
compared canonicalize(current_exe()) vs the deploy path, and the deployed binary is
makeWrapper-wrapped, so current_exe() resolves to a wrapper/coreutils path that never equals
the real store path. Result: `reload` ALWAYS re-execs and can't tell you whether anything new
was actually deployed. Christian: "when it reloads it should identify the new binary or version."
Bonus correction: this proves `reload` DOES pick up deploys (the earlier close+reopen ritual was
unnecessary -- reload re-execs into the new binary correctly).

## What
Record the launched-from build at startup; compare on reload.
- Startup (main.rs after db open): write canonicalize(/run/current-system/sw/bin/faelight-shell)
  -> /tmp/fsh-running-build. The store hash changes every rebuild = build identity.
- reload_fsh() (mod.rs): compare current deploy-target store path vs the recorded marker:
    same hash -> "Already on the current fsh build ... nothing new to reload" (NO re-exec)
    diff hash -> "New fsh build detected -- reloading: was <hash> / new <hash>" (re-exec)
    no marker -> reload anyway, say so.
Never uses current_exe() (wrapper-unreliable). exec() keeps the PID and the new process rewrites
the marker on startup, so a second reload correctly reports nothing-new. Single-terminal marker
(matches Christian's one-fsh-at-a-time workflow; multi-terminal is last-writer, acceptable).

## Gates
- [x] startup writes /tmp/fsh-running-build with the resolved deploy store path
- [x] reload with NO new deploy -> "Already on the current fsh build ... nothing new" (no re-exec)
- [x] reload AFTER a new deploy -> "New fsh build detected" showing was/new store paths, then reloads
- [x] verified live across a real rebuild+deploy cycle

## Where
main.rs ~551 (startup marker write), commands/mod.rs ~9513 (reload_fsh rewrite),
resolve_fsh_binary() ~9491 (unchanged, still resolves deploy path).

## The Rule
"A reload that can't tell you what it loaded is just a restart wearing a disguise.
 Name the build, or admit there's nothing new." 🌲


## Progress -- 2026-06-26 (COMPLETE -- all gates proven live)
Root cause: reload compared canonicalize(current_exe()) vs the deploy path; the makeWrapper
wrapper made current_exe() resolve to a non-fsh path (coreutils), so the guard never matched
and reload always blind-re-exec'd. Fix: startup records canonicalize(deploy symlink) ->
/tmp/fsh-running-build; reload compares the current deploy target against that marker.
PROVEN LIVE:
  reload (no new deploy) -> "Already on the current fsh build: <store-path>  Nothing new" (no re-exec).
  reload (after deploy)  -> "New fsh build detected -- reloading: was rmvz8.../ new drrb9..." then re-exec.
  marker file written at startup with the resolved deploy store path.
Bonus finding (correction to working conventions): reload DOES pick up deploys via exec() --
the habitual close+reopen-terminal step after a bundled-crane-tool deploy is NOT required;
`reload` is sufficient and now reports exactly what it loaded. Test scaffold (build nonce) removed.
