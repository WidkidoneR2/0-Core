---
id: 155
date: 2026-03-26
type: future
title: "faelight-shell Prompt Themes — The Shell Has a Face"
status: planned
tags: [shell, prompt, themes, visual, identity, fsh, v12]
version: 12.0.0
priority: medium
---

## The Vision
faelight-shell has one prompt style — the forest prompt.
It is good. But the shell is mature enough to have multiple faces.
Different contexts call for different visual voices.

## The Four Themes

### Theme 1 — Forest (current, default)
```
🌲 ~/0-core (main*)
→ 100% · 5 today
fsh ❯
```
Full forest context. Health, git, commit count. The standard.

### Theme 2 — Minimal
```
~/0-core ❯
```
Just path and cursor. Zero noise. For when you need focus.
No health, no git, no counters. Pure signal.

### Theme 3 — Jarvis
```
🌲 ~/0-core (main*) INT-146 · 100% · predict: next session Wed
fsh ❯
```
Forest prompt PLUS inline prediction data.
Shows active intent, health, and a one-line prediction.
For when you want the forest's full situational awareness visible.

### Theme 4 — Classic
```
christian@fealight ~/0-core (main*) $
```
Traditional Unix prompt. For screenshots, demos, or nostalgia.
Still forest-aware under the hood.

## Commands
```bash
theme forest     — switch to forest theme (default)
theme minimal    — switch to minimal theme
theme jarvis     — switch to jarvis theme  
theme classic    — switch to classic theme
theme            — show current theme
```

Theme persists in config.fsh:
```
set prompt_theme = jarvis
```

## Implementation
All themes live in `prompt.rs`.
Theme is read from config on startup and on `theme` command.
No restart required — prompt updates immediately.

## Gate Check
```
⬜ theme command in fsh dispatch
⬜ Forest theme (existing, default)
⬜ Minimal theme (path only)
⬜ Jarvis theme (prediction inline)
⬜ Classic theme (traditional Unix)
⬜ theme persists in config.fsh
⬜ theme switches immediately without restart
```

## The Phrase
**"The shell that knows itself
can also choose how it appears.
A forest can be dense and alive,
or quiet and waiting."**

---
*"Same forest. Different voice. You decide."* 🌲
