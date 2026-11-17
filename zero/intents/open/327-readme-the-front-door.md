---
id: 327
date: 2026-11-17
type: future
title: "README: the front door shows what Project 0 is"
status: planned
tags: [docs, readme]
format: 2
---

## Vision
Someone landing on github.com/WidkidoneR2/0-Core sees in one screen what
Project 0 is: Rust on Arch Linux and Omarchy, one engine, a shell, and a
ledger that remembers why. Every number on the page is counted from the repo
or from a run on Christian's laptop, and says where it came from.

## The Problem
The root README predates INT-313 to INT-319. It has no badges, no picture of
the shell running, and no account of how an intent moves from plan to
release. Some lines no longer match how work is done: AI assistants now
commit on branches, and nothing says that only Christian's approval reaches
main.

## The Solution
Replace README.md and add two images under assets/readme/:
- banner.svg, drawn from assets/branding/zero-core-mark.jpg, system fonts,
  motion stops under prefers-reduced-motion.
- terminal-session.svg, a CSS-only replay of the 2026-11-15 session at
  d2858903 (nsh banner, doors 4 passed, d 28/28), labelled as a replay, with
  the same text in a readable block below it.
The README adds badges (MIT, top language from GitHub, Arch Linux, Omarchy
and Hyprland, no AI in core), "See it run", "At a glance" counted at
d2858903, a mermaid diagram of the life of an intent, and folds for the long
reference sections. "A human decides" reads: Christian reads every plan,
checks its Seal and approves it; AI assistants can draft plans, tests and
patches; nothing reaches main without his explicit approval.

## Success Criteria
- [ ] G1 README.md, assets/readme/banner.svg and
      assets/readme/terminal-session.svg are on main, and no draft notes or
      FILL markers remain in README.md.
      ask: the new README and both images are on main with no draft notes
      ruling:
      evidence:
- [ ] G2 the page renders on GitHub: badges, banner, terminal replay and
      both mermaid diagrams show.
      ask: the README renders on GitHub with badges, images and diagrams
      ruling:
      evidence:
- [ ] G3 doors and d green on the laptop.
      ask: doors and d are green on the laptop
      ruling:
      evidence:

## Out of scope
- No code changes and no version bumps.
- The generated tool READMEs and tools.toml are not touched.
