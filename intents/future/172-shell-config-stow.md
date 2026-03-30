---
id: 172
date: 2026-03-30
type: future
title: "Shell Config Stow — config.fsh Under Version Control"
status: in-progress
tags: [stow, config, faelight-shell, version-control, symlink, structure]
version: 11.5.0
priority: medium
---
## The Problem
~/.config/faelight-shell/config.fsh is a loose file.
It is NOT tracked in git. It is NOT stowed. It is NOT part of 0-core.

Every change made to it is invisible to the forest.
If the system is rebuilt, config.fsh is lost.
This violates the core principle: everything is understood and controlled.

All other configs follow the stow pattern:
  shell-zsh       → ~/.config/zsh/
  config-faelight → ~/.config/faelight/
  editor-nvim     → ~/.config/nvim/

config.fsh has no stow home. It is an orphan.

## The Solution
Create stow package shell-faelight mirroring shell-zsh exactly:

~/0-core/03-interfaces/stow/shell-faelight/
└── .config/
    └── faelight-shell/
        └── config.fsh     ← source of truth

~/.config/faelight-shell/config.fsh
    → symlink to ~/0-core/03-interfaces/stow/shell-faelight/.config/faelight-shell/config.fsh

One source of truth. Version controlled. Stowed. Editable from 0-core.

## What Does NOT Change
- plugins/ directory stays at ~/.config/faelight-shell/plugins/ — not stowed
- faelight-shell behavior is identical — it reads the same path
- No Rust code changes required

## Implementation Order (careful, one step at a time)
Step 1: Read current config.fsh — confirm exact content
Step 2: Create stow package directory structure
Step 3: Copy config.fsh into stow package
Step 4: Verify content matches exactly
Step 5: Remove original ~/.config/faelight-shell/config.fsh
Step 6: Run stow to create symlink
Step 7: Verify symlink points correctly
Step 8: Launch faelight-shell — confirm aliases load
Step 9: Run d — verify 100% health
Step 10: fg commit

## Gate Check
```
⬜ shell-faelight stow package created
⬜ config.fsh content verified identical before/after
⬜ ~/.config/faelight-shell/config.fsh is now a symlink
⬜ Symlink resolves to ~/0-core/03-interfaces/stow/shell-faelight/...
⬜ faelight-shell loads aliases correctly after stow
⬜ d shows 100% health, stow symlinks all valid
⬜ fg commit — config.fsh tracked in git
```

## The Phrase
**"A config file outside version control
is a config file you will lose.
The forest owns its own configuration."**
