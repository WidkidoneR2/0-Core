---
id: 190
date: 2026-04-03
type: incident
title: "fsh Login Shell Broke Niri Session"
status: resolved
tags: [fsh, login-shell, niri, incident, greetd]
---

## What Happened
Setting fsh as login shell via chsh caused Niri to not start after login.
User was dropped into bare fsh terminal instead of graphical desktop.

## Root Cause
niri-session is a POSIX shell script that checks $SHELL and re-execs
itself through it. With SHELL=faelight-shell, it tried to use fsh with
POSIX -c flag syntax. fsh does not implement this, so Niri never started.

## Timeline
- chsh set fsh as login shell
- Reboot: faelight-login worked, password accepted
- Expected: Niri desktop. Actual: bare fsh terminal
- niri --session showed PermissionDenied (red herring)
- Workaround found: export SHELL=/bin/zsh then /usr/bin/niri-session
- Permanent fix: SHELL=/bin/zsh added to faelight-login session env

## The Fix
Added SHELL=/bin/zsh to greetd session env in faelight-login/src/main.rs
so niri-session uses zsh for its internal POSIX logic while fsh remains
the interactive shell inside faelight-term.

## Architecture
greetd -> faelight-login -> niri-session (needs SHELL=zsh)
  -> niri -> faelight-term -> faelight-shell (fsh lives here)

## Lesson
System launcher scripts that check $SHELL need POSIX shell.
Interactive shell and session launcher shell are different roles.

## Resolution
- faelight-login sets SHELL=/bin/zsh in session env
- Niri starts correctly
- fsh remains interactive shell inside faelight-term
- Long-term: implement POSIX -c flag in fsh (INT-179 scope)
