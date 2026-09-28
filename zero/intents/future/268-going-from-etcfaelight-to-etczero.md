---
id: 268
date: 2026-09-28
type: future
title: "going from /etc/faelight to /etc/zero"
status: planned
tags: [faelight, zero, nsh, novashell]
---

## Vision

/etc/faelight is gone from every live file, and Project 0's system-level presence is /etc/zero --
ruled by Christian 2026-09-28. It is REAL, not a name: every file there has one named writer and at
least one reader, the writer is run by the system rather than by a shell or a script holding sudo,
and every reader says UNREADABLE when it cannot read, never "" or 0.

## The Problem

What is known, and where each fact comes from:

```text
    on this machine   /etc/faelight does not exist (INT-247 census, 2026-09-24: absent)
    its writer        the NixOS system build, as root, at rebuild. nsh still spawned
                      faelight-export at every shell start for it until c3ad4a8d removed the
                      dead call (2026-09-28)
    its readers       five, which answered "" and 0 for three weeks after the migration; INT-250
                      moved them to git and state.db. The D-Bus service reported NO ACTIVE
                      INTENT for those three weeks
    what it held      as INT-250's notes name it: INTENT, HEALTH, and a commit count. The full
                      list is recon, not memory
    still naming it   nine live comment lines, INT-250's history notes (fsearch 2026-09-28):
                      engine autobiography, novashell commands/mod.rs twice, zero-docs,
                      zero-daemon dbus.rs twice, zero-release main.rs twice, zero-core paths.rs
```

WARNING -- THE HARD PART IS THE WRITER, NOT THE NAME. /etc is root-owned. NixOS wrote /etc/faelight
declaratively during the system build. Omarchy has no such step, and automation holding sudo is the
2025-12-14 lockout class. Whatever writes /etc/zero has to be a mechanism the SYSTEM runs.

AND NO ZOMBIE. A directory nothing writes, or a file nothing reads, is what INT-247's NO ZOMBIE CODE
gate forbids. Every file earns its place in /etc/zero by needing to be system-wide: readable by a
system service, another user, or before login. Anything only this user reads stays per-user in
~/.local/state/zero, where Project 0's state already lives.

## The Solution

```text
    1  RECON     read-only. Every file /etc/faelight held, from history (git log -S, INT-250,
                 the NixOS module as it was): its writer, its readers then, and what reads that
                 information now
    2  PER FILE  system-wide -> /etc/zero, or per-user -> stays in ~/.local/state/zero. Ruled
                 by Christian, with the reason
    3  WRITER    for what goes to /etc/zero: a declarative mechanism the system runs --
                 candidates to measure, not assume: a pacman hook, systemd-tmpfiles, a system
                 service. Chosen by Christian. Rehearsed in a throwaway root before the machine
    4  MOVE      one file per commit: writer, readers and a doctor check together. Readers
                 report UNREADABLE as unreadable
    5  OR CLOSE  if step 2 finds nothing that needs to be system-wide, /etc/zero is not created
                 and the reason is recorded here. That is a complete outcome, not a failure
```

### Order

Steps 1 and 2 read and rule, touch nothing, and can run any time. Steps 3 and 4 wait for INT-247 to
close: the faelight and forest removal comes before every other intent (Christian, 2026-09-28).
INT-247's comment pass rewords the nine /etc/faelight comments; this intent does not need them.

### Not in scope

The per-user state directories (INT-247 Layer 3, done). Any change that makes nsh or a script
write /etc.

## Success Criteria

- [ ] RECON recorded here: every file /etc/faelight held, its writer, its readers then, and what
      reads that information now -- measured from history, not recalled
- [ ] PER FILE, the ruling: system-wide (to /etc/zero) or per-user (stays in ~/.local/state/zero),
      with the reason
- [ ] THE WRITER, for anything going to /etc/zero: declarative, run by the system -- not nsh, not a
      script holding sudo -- chosen by Christian, and rehearsed in a throwaway root first
- [ ] NO ZOMBIE: every file under /etc/zero has one named writer and at least one reader, and a
      doctor check reports a file with no writer, or a reader of a missing file, as a failure
- [ ] UNREADABLE IS UNREADABLE: every reader of /etc/zero reports a file it cannot read as
      unreadable -- no "" and no 0 (INT-247's criterion, INT-250's lesson)
- [ ] /etc/faelight appears in no live file (shared with INT-247's gate)
- [ ] OR: if nothing needs to be system-wide, the intent closes with /etc/zero not created and the
      reason recorded here

## Relationship

- INT-247 -- its /ETC/FAELIGHT gate; its comment pass rewords the nine comments
- INT-250 -- moved the readers off /etc/faelight; its notes are the recon's first source
- The 2025-12-14 lockout -- why nothing automated holds sudo
