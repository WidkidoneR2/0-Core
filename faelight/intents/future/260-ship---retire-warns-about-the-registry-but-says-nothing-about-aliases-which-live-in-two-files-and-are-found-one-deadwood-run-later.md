---
id: 260
date: 2026-09-23
type: future
title: "ship --retire warns about the registry but says nothing about aliases, which live in two files and are found one deadwood run later"
status: planned
tags: [ship, retirement, aliases, deadwood]
---

## Vision
Retiring a tool tells you EVERYTHING that still points at it, at the moment you can act on it.

## The Problem
`ship --retire` takes the binary off PATH and warns about one thing: "the registry still lists X
as deployable; deadwood flags it as an orphan until that entry says retired = true." It says
nothing about aliases. Measured 2026-09-23, across two retirement rounds in ONE session:

```text
    wallpaper, vault, zone     6 dead aliases, all in config.nsh
    intent-guard, clipboard    4 dead aliases: 3 in registry/aliases.toml AND config.nsh,
                               1 (guard -> intent-guard) in config.nsh alone
```

Both times `nsh-test` went red on deadwood_strict_gate_passes ONE RUN LATER, and both times the
fix was a second pass. ⚠️ AND THE SECOND ROUND MISSED ONE BECAUSE IT SEARCHED FOR A GUESSED ALIAS
NAME RATHER THAN THE TOOL NAME: the clipboard aliases were found and removed, `guard` was not,
because nobody thought to ask what intent-guard was called.

⚠️ THEY LIVE IN TWO PLACES WITH DIFFERENT RULES, AND THAT IS THE DESIGN CONSTRAINT:

```text
    faelight/registry/aliases.toml    IN the repo; documents ~50 of them
    ~/.config/faelight-shell/config.nsh   OUTSIDE the repo; defines 301 -- the live shell
```

A retirement commit can carry the first and CANNOT carry the second. The file itself says so:
"Total: 50+ primary aliases documented. Full 301 aliases are active in shell but documented
incrementally."

## The Solution
`ship --retire` REPORTS. It does not remove.

It already knows the tool name and already prints the registry warning, so it prints every alias
naming that tool -- file and line, both files -- at the moment the tool is retired. Removal stays
explicit: by hand, or behind a `--with-aliases` flag that says what it did.

⭐ THE TWO-STEP IS THE POINT, NOT A COMPROMISE. ship reports at the moment you can act; deadwood
proves afterwards that you did. A ship that removed silently would make deadwood's check go quiet,
and the proof would disappear along with the problem. Two questions, two moments, two tools.

⚠️ AND A TOOL THAT EDITS YOUR LIVE SHELL CONFIG IS A DIFFERENT TOOL. config.nsh is the environment
you are typing in, and no commit records a change to it. Reporting costs nothing and is always
right; removing needs a flag, and even then ship must say it touched a file outside the repository.

★ ONE SOURCE OF TRUTH, ASKED TWICE. deadwood already answers "what points at this tool, and does
the target resolve" -- check_dead_aliases does exactly that. ship should ask THAT code rather than
growing a second answer that can drift from it. Two implementations of one question is the shape
this project keeps finding: two tokenizers, two alias owners, two usage strings.

## Success Criteria
- [ ] WATCH IT FAIL FIRST: retire a tool that has an alias and record what ship prints today. The
      registry warning WITHOUT an alias warning is the finding this intent is filed on
- [ ] `ship --retire` names every alias for that tool in BOTH files, each with file and line
- [ ] proven against a tool with aliases in both files AT ONCE -- faelight-clipboard on 2026-09-23
      was exactly that case, and the repo half was found while the config half was not
- [ ] removal stays EXPLICIT: by hand, or behind a flag. This gate says which was built and why
- [ ] ⚠️ config.nsh is edited ONLY when the flag is passed, and ship SAYS it touched a file outside
      the repository -- no commit will record that change
- [ ] the alias question is answered by the code deadwood already uses, not a second
      implementation. If that turns out to be impractical, say so here with the reason
- [ ] a retirement lands with deadwood clean on the FIRST run, proven end to end on a real
      retirement rather than a fixture
