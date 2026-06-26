---
id: 089
date: 2026-06-24
type: future
status: in-progress
title: "fsh: clearer errors when && chains hit a builtin"
tags: [fsh, shell, errors, ux, builtins, paper-cut]
priority: low
---
## Why
When a command chain like `git status && d` is handed to `sh -c` for execution, fsh
builtins and aliases (`d`, `inspect`, vocabulary words, etc.) are invisible to `sh` --
so the user gets a bare, misleading error:
    sh: line 1: d: command not found
That message suggests the command does not exist, when in fact `d` is a perfectly valid
fsh builtin -- it just cannot be reached through the `&&`-to-`sh` handoff. The user
(observed live 2026-06-24, running `git status --short && d`) is left thinking something is
broken when it is only the known execution-model boundary.
## What
Make the failure legible. When a "command not found" occurs inside an sh-routed chain,
detect whether the missing name is actually a known fsh builtin/alias, and if so emit a
clear message naming the cause and the workaround. For example:
    fsh: 'd' is an fsh builtin -- && chains run through sh, which cannot see builtins.
         Run it on its own line, or use the parallel/sequence forms (see INT-267).
## Scope boundary (important)
This intent is ERROR-MESSAGE CLARITY ONLY. It does NOT change execution semantics.
Making `&&` actually route builtins correctly is the execution-model work associated
with INT-267 (parallel execution / the `cannot run via sh -c` boundary at
rust-tools/faelight-shell/src/main.rs:1096-1112). This intent makes the CURRENT failure
self-explanatory; INT-267's model is where a real fix would eventually live.
## Where it lives
rust-tools/faelight-shell/src/main.rs around line 1096 ("fsh builtins that cannot run
via sh -c") and 1112 (the && NOTE). The builtin/alias list already exists in fsh -- the
detection can reuse it to recognise the name before sh ever sees it, OR post-process the
sh "command not found" against the known-builtin set.
## Gates
- [ ] fsh recognises when a failed sh-chain command name is a known fsh builtin/alias
- [ ] emits a clear message: names the builtin, explains the && -> sh boundary, gives the workaround
- [ ] does NOT false-positive on genuinely missing commands (real typos still say "not found")
- [ ] verified live: `git status && d` produces the clear message, not bare "sh: d: command not found"
## The Rule
"An error that blames the wrong thing is worse than no error.
 If the forest knows `d` is its own word, it should say so." 🌲
