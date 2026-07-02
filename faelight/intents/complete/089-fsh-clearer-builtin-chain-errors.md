---
id: 089
date: 2026-06-24
type: future
status: complete
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
- [x] fsh recognises when a failed sh-chain command name is a known fsh builtin/alias
- [x] emits a clear message: names the builtin, explains the && -> sh boundary, gives the workaround
- [x] does NOT false-positive on genuinely missing commands (real typos still say "not found")
- [x] verified live: `git status && d` produces the clear message, not bare "sh: d: command not found"
## The Rule
"An error that blames the wrong thing is worse than no error.
 If the forest knows `d` is its own word, it should say so." 🌲


## Progress -- 2026-06-26 (COMPLETE -- proven live)
Root cause located precisely: INT-322 routes &&-chain segments WITH a redirect (2>, >, >>) to
sh (line ~1190 main.rs); sh can't see fsh builtins/aliases -> misleading "command not found".
This is the papercut that bit ~12x this session (cistart/deploy/cicomplete + 2>&1).
Fix (clarity only, execution unchanged): added completion::is_fsh_only_word() -- true for
aliases + forest builtins NOT on PATH (deploy, cistart, d, fg...), false for PATH tools
(git, cargo). At the redirect->sh branch, if the segment's command word is a forest word AND
sh failed, emit a clear message naming the word + the redirect->sh boundary + the workaround.
PROVEN LIVE:
  `true && cistart 099 2>&1` -> "sh: cistart: command not found" THEN the clear forest message.
  `cistart 099 2>&1 | tail` -> ran via dispatcher, NO spurious warning (gate 3).
  `git status 2>&1` -> normal, no warning (PATH tool, gate 3).
Scope honored: did NOT change execution semantics (real routing = INT-267/322). The charter's
"&&" framing refined: the trigger is redirect-on-a-chain-segment, not && alone.
