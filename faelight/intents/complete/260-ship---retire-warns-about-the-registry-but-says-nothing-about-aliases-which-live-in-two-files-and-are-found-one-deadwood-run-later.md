---
id: 260
date: 2026-09-23
type: future
title: "ship --retire warns about the registry but says nothing about aliases, which live in two files and are found one deadwood run later"
status: complete
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
- [x] WATCH IT FAIL FIRST: retire a tool that has an alias and record what ship prints today. The
      registry warning WITHOUT an alias warning is the finding this intent is filed on
      <!-- evidence: 2026-09-24, the deployed PRE-FIX ship (~/.local/bin/ship, built 2026-09-23 19:26) run as ship --retire faelight-gen --dry-run printed only the would-move and to lines. No alias warning, and no registry warning either: both checks sat after the removal, so the dry-run skipped them. faelight-gen had 4 lines naming it across both files and ship said nothing about any of them. Dry-run used rather than a real retire, since the tool is still in use. -->
- [x] `ship --retire` names every alias for that tool in BOTH files, each with file and line
      <!-- evidence: 2026-09-24, target/release/ship --retire <tool> --dry-run lists every aliases.toml line of the tool's block (command, primary, aliases, in any field order) and every config.nsh alias line, each as path:line. Block scan replaced the next-line check, which lost faelight-gen's primary and aliases lines (161, 162). -->
- [x] proven against a tool with aliases in both files AT ONCE -- faelight-clipboard on 2026-09-23
      was exactly that case, and the repo half was found while the config half was not
      <!-- evidence: 2026-09-24, faelight-gen (4 lines: aliases.toml 160/161/162 + config.nsh:292), faelight-git (7), faelight-update (7) -- each has aliases in BOTH files at once. faelight-gen stood in for faelight-clipboard, which was cleaned by hand on 2026-09-23 and no longer has aliases to find. -->
- [x] removal stays EXPLICIT: by hand, or behind a flag. This gate says which was built and why
      <!-- evidence: BY HAND. ship reports and never removes (alias_report doc comment in ship/src/main.rs): config.nsh is outside the repository so no commit would record a removal, and a silent removal would make deadwood's check go quiet along with the problem. Ruled 2026-09-23: report, don't remove. -->
- [x] ⚠️ config.nsh is edited ONLY when the flag is passed, and ship SAYS it touched a file outside
      the repository -- no commit will record that change
      <!-- evidence: 2026-09-24, no flag was built (gate 66: by hand), and ship has NO write path to config.nsh. Every write call in ship/src/main.rs: 138 prunes old binary backups beyond KEEP; 149/155 stage and rename the deployed binary; 244/249/505 copy and remove the retired binary into the backup dir. shell_config() appears once, at 312, where alias_report READS it. The only subprocesses are cargo (build) and core deploy record (the deploy log, 169). ship does SAY config.nsh is outside the repository: alias_report prints that no commit will record a change to it. -->
- [x] the alias question is answered by the code deadwood already uses, not a second
      implementation. If that turns out to be impractical, say so here with the reason
      <!-- evidence: IMPRACTICAL, measured 2026-09-24. check_dead_aliases (faelight-deadwood/src/main.rs:592) answers a different question: does each config.nsh alias target still RESOLVE (BUILTINS, alias names, on_path). ship asks BEFORE the removal, while the target is still on PATH, so that code returns clean by construction for exactly the tool being retired. It also reads config.nsh only, never aliases.toml, and deadwood is binary-only (no [lib] target), so ship cannot call it. The two are complementary: ship reports before, deadwood proves after. The one genuinely shared piece is the alias-line parser (parse_alias, deadwood main.rs:646); ship matches alias lines more loosely. Moving parse_alias into faelight-core is the candidate follow-up, not done here. -->
- [x] a retirement lands with deadwood clean on the FIRST run, proven end to end on a real
      retirement rather than a fixture
      <!-- evidence: 2026-09-24, faelight-fm. Its binary was already off PATH before today and its [[alias]] block (aliases.toml 33-36) was still dangling -- the exact case 260 was filed on. BEFORE removal, faelight-deadwood --only aliases reported clean WITH that block present: deadwood cannot see the toml half, and ship is the only reader of aliases.toml (faelight-core aliases_registry() has no callers). Block removed via fpatch, tomllib parse verified. Then, first run each: ship --retire faelight-fm --dry-run named zero lines (toml half); faelight-deadwood --only aliases --strict exited 0 (config half); nsh-test full suite green, deadwood_strict_gate_passes row 193468 passed=1 at 2026-09-24 00:20:34 CDT in ~/.local/state/zero/state.db. Its commit field 9509a2a0 is HEAD: the fix was still uncommitted working tree. -->
