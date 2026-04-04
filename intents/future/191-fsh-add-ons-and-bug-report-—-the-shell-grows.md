---
id: 191
date: 2026-04-03
type: planned
title: "fsh Add-Ons and Bug Report — The Shell Grows"
status: in-progress
tags: [fsh, shell, builtins, glob, grep, find, ux, daily-driver]
---
fsh is the daily driver. The 30-day clock is running.
Every friction point discovered in real use gets fixed here.
Every capability gap gets filled here.
This is not a version bump — it is the shell maturing.
The forest's shell should feel like an extension of thought.
Not a tool you fight. A tool that anticipates.
---
✅ Glob expansion — *.md, .rs, **/.toml do not expand
⬜ grep | head pipe — stderr bleeds through on pipe errors
⬜ Inline env var assignment — SHELL=/bin/zsh cmd not supported
⬜ fsh typed inside fsh — spawns new shell instead of showing status
⬜ zsh typed inside fsh — spawns zsh instead of showing fsh identity
✅ Tab completion for aliases — ci<TAB> should show cistart/cicomplete
---
⬜ grep — basic pattern matching (no need to fall through to system grep)
⬜ find — name + depth + type filters
⬜ tree — recursive directory view with depth control
⬜ stat — detailed file metadata
⬜ preview — quick file preview (bat for text, size for binary)
⬜ realpath — resolve absolute paths
✅ !! — repeat last command
⬜ !<pattern> — search and run from history
⬜ history search — fuzzy history search inline
⬜ alias — list all aliases (already partially done)
⬜ unalias — remove alias from DB
✅ export — set environment variable for session
✅ unset — remove variable
---
These are what make fsh different from every other shell.
Not just executing commands — understanding and transforming data.
⬜ filter — structured filtering like NuShell (where field > value)
⬜ where — broader than which: finds aliases, builtins, scripts, binaries
⬜ select — pick fields from structured output
⬜ group — group structured data by field
⬜ from json — parse JSON into forest table
⬜ to json — serialize forest data to JSON
⬜ open — read file into structured data (JSON, TOML, CSV auto-detected)
⬜ time <cmd> — measure execution time of any command
⬜ eval — run dynamically generated commands
⬜ source — load .fsh scripts into current session scope
---
These are the ones that make people stop and stare.
Build after Tier 1 and 2 are solid.
⬜ explain <command> — describe what a command does, its aliases, its source
⬜ run python — inline Python execution without heredoc
⬜ run js — inline JavaScript execution
⬜ preview <file> — smart preview based on file type
⬜ undo — revert last filesystem operation (mv, cp, rm tracking)
⬜ fsh typed in fsh → show faelight-fetch + forest identity screen
⬜ zsh typed in fsh → show fsh identity, offer to continue in zsh
---
When someone types `fsh` inside fsh — instead of spawning a subshell,
show who the forest is:
🌲 Faelight Shell v0.6.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Login shell   ✅ since 2026-04-03
Daily driver  ✅ day 1 of 30
Aliases       370 loaded
Builtins      55 native commands
Themes        forest · minimal · classic · jarvis
Health        100%
Jarvis        90/100 — Strategic Advisor
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
The forest thinks in Rust.
Every command understood. Nothing installed blindly.
---
⬜ All known bugs fixed
✅ Glob expansion working — *.rs expands correctly
⬜ Tier 1 builtins complete — grep, find, !!, alias/unalias, export/unset
⬜ Tier 2 structured data — filter, where, select, from/to json, open
⬜ fsh identity screen — typing fsh shows status not subshell
⬜ explain command working — forest knows its own tools
✅ Tab completion for aliases working
⬜ Zero friction daily driver — 30 days without reaching for zsh
---
**"A shell that grows with you
is not a tool — it is a collaborator.
Every bug fixed is a conversation improved.
Every builtin added is a thought made faster.
The forest's voice gets clearer every session."**
---
*"The best shell is the one you never have to think about.
You just think — and it executes."* 🌲
