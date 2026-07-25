---
id: 197
date: 2026-07-25
type: arch
title: "the safety gate evaluates the typed command, not the command that will run"
status: planned
tags: [architecture, rust, design]
---

## Vision
The safety gate evaluates the representation that will actually execute.

## The Problem
safety_guard::check has two call sites, main.rs:1175 and main.rs:1217. Alias
expansion is at main.rs:2311. Both calls are far above it, so the guard inspects
the line as TYPED -- after comment-stripping, history expansion, normalize and
brace expansion, but before variables, subshells, globs and aliases.

So an alias whose expansion is a gated command is not gated. Given
`alias zap='rm -rf /'`, the guard's first word is `zap`, which matches no deny
entry, no allow entry, no safe entry, and fails its own `first_word == "rm"`
test. It returns None. The executor then expands and runs it.

PROPORTION, stated so future readers do not over-scope this: fsh is a
single-user personal shell and this gate is an "are you sure" confirmation, not a
boundary defending against an attacker. No alias in the current 285 expands to
`rm -rf`. This is a real gap worth closing, not an emergency.

## Why this is NOT INT-195
INT-195 owns HOW the command word is derived -- an implementation inconsistency,
fixed by routing through commands::command_word(). This intent owns WHICH
REPRESENTATION the gate evaluates, and WHEN it sees it. That is a policy
decision, not a parsing one, and folding them together would let the policy
question be quietly answered by whoever happened to be fixing the derivation.

## ⚠️ Moving the call is not the obvious fix
The allow, deny and safe lists all match BARE NAMES. `safe` contains entries like
core, git, cargo, cat, ls, cd. If the gate ran after expansion it would see
expanded text, and `d` expands to `/run/current-system/sw/bin/core doctor run`,
whose first word is an absolute PATH -- so it would fall out of the safe list and
start gating a harmless doctor run. Any placement change has to reconcile the
lists with whatever representation is chosen. That reconciliation IS most of the
work; the call move is the small part.

## The actual question
Which representation should the gate evaluate?
  - AS TYPED: what it does today. Honest about intent, blind to aliases.
  - CANONICAL COMMAND WORD: quote-aware, still blind to aliases.
  - POST-EXPANSION: sees what will run, but breaks bare-name list matching and
    raises the same question again for plugins.
  - THE EXECUTION PLAN: the eventual answer once INT-169's spine owns execution,
    since the plan is precisely "what will run" in structured form.
Choosing among these is the intent. Do not open by moving the call.

## Success Criteria
- [ ] The representation is CHOSEN and RECORDED with reasoning, including what is
      given up. A decision, not a default
- [ ] The allow, deny and safe lists are reconciled with that representation, or
      the mismatch is recorded as a stated limitation
- [ ] PROVEN: an alias whose expansion is a gated command is gated. Watched
      failing first, on a throwaway alias, never on a real destructive one
- [ ] REGRESSION PROVEN: commands that are safe today are still not gated --
      specifically the bare-name entries reached through aliases, `d` being the
      known hard case
- [ ] The interaction with INT-195 is stated: derivation and placement touch the
      same function, and whichever lands second must not silently undo the first
- [ ] The gate's behaviour is covered by a REPL test, since this is interactive
      behaviour and the -c door does not reach fsh's own dispatch at all
