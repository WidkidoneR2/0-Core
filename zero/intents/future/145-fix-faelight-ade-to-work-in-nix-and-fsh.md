---
id: 145
date: 2026-07-12
type: future
title: "faelight-ade: the paired fsh + Friday pane, unverified since the migration"
status: planned
tags: [faelight-ade, ade, Nix]
---

## The Problem

faelight-ade is a working tool that nobody has run since the migration. It
builds, it is deployed to ~/.local/bin, it is in registry/tools.toml, and fsh
launches it (ade, commands/mod.rs:17009). What it does under Omarchy is unknown.

The old title said fix it to work in Nix and fsh. The Nix half was never true:
it is crossterm + ratatui + portable-pty + rusqlite, with no Nix dependency
anywhere. The fsh half asserts a defect with no reproduction attached -- a claim
that cannot be checked, which is the class this ledger exists to remove.

## Why it is worth verifying rather than assuming

It embeds fsh in a PTY. That makes it a live test of the same two-doors question
INT-173 answered for fsh-test: a shell driven through a pty is not the same
shell as -c, and ade is the only other consumer that drives the real REPL.

## Known, without running it

WARNING: let _ = std::process::Command::new("faelight-ade").spawn(); -- if the
binary is absent, ade does nothing and reports nothing. Same discarded-error
shape as the startup cd. Small, and it belongs here.

NOTE: the palette is hardcoded. GREEN = Rgb(42,255,213) is a cyan-teal and
ACCENT is deep sky blue. Neither is Hakker Green 00ff99. Another file carrying
its own colours -- evidence for the single token source, not work for this
intent.

NOTE: the header cites INT-346, an arch-era number with no ledger file. Belongs
to INT-231.

## Success Criteria
- [ ] Run it. Record what actually happens -- both panes, on Omarchy
- [ ] Every defect found gets a reproduction, or it is not written down
- [ ] The spawn failure is reported instead of swallowed
- [ ] Decide whether ade survives Quickshell and the fsh-side Friday work, or
      whether it was a prototype that has been overtaken
