---
id: 097
date: 2026-02-27
type: decision
title: "New Tools Integration — atuin, tokei, difft, btm, onefetch"
status: complete
tags: [tools, aliases, workflow, daily-use]
---

## Decision

Install and integrate 8 new tools into the daily workflow via aliases.

## Tools and Rationale

| Tool       | Alias   | Replaces       | Why |
|------------|---------|----------------|-----|
| btm        | top     | htop           | Rust, better layout, process tree |
| onefetch   | repo    | —              | instant repo summary with git stats |
| hyperfine  | bench   | time (manual)  | statistical benchmarking |
| ouch       | extract | various        | one command for all archive formats |
| difft      | diff    | diff           | semantic diff — understands syntax |
| tokei      | loc     | wc -l          | language-aware LOC with breakdown |

## What Was Not Installed

- tealdeer: not available, skip
- cargo-flamegraph: installed but no alias — used directly when needed

## Philosophy Alignment

Each tool was evaluated against "Understanding over convenience":
- All are read-only or additive (no automation risk)
- All provide more signal than what they replace
- None introduce workflow lock-in
- All are Rust-based (consistent with ecosystem)

## Aliases Added

332 total aliases after this change. Audit clean, no conflicts.
