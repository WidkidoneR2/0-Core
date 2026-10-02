---
id: 268
date: 2026-09-28
type: future
title: "going from /etc/faelight to /etc/zero"
status: complete
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

## Recon 2026-10-01 (read-only, measured from history)

Every file /etc/faelight held on framework16, its writer, its readers then, and what reads that
information now. Omarchy is the system. Project 0 is the group of tools Christian built through
his shell, nsh; faelight was their old name, and zero replaces it.

```text
    VERSION    writer   NixOS build, framework16 configuration.nix:179, from meta/VERSION
               readers  faelight-login (the greeter; it no longer exists), novashell
                        commands/mod.rs:13350, health_tui.rs:351
               now      paths::version_file(), the one owner of zero/meta/VERSION (paths.rs:48)
    INTENT     writer   faelight-export, spawned by nsh at every start (main.rs:2328, INT-242,
                        121eb525); the dead call removed in c3ad4a8d
               readers  daemon dbus.rs:185 and :195; compositor chrome title (2502d39f; that
                        compositor removed in c832aa6d)
               now      the intent ledger (INT-250)
    HEALTH     writer   faelight-export
               readers  daemon dbus.rs:173
               now      the health-status file the doctor writes (INT-250)
    COMMITS    writer   faelight-export; the faelight-release writes removed in f92e596a
               readers  engine autobiography/mod.rs:89, docs main.rs:403, novashell
                        commands/mod.rs:11038 and :11125
               now      git rev-list (INT-250)
    recovery-runbook.md
               writer   the rescue ISO build, rescue configuration.nix:47 (7d13eb41)
               readers  a human booted from the rescue USB; never framework16
               now      not this host
```

Sources: git grep -F environment.etc at deb94bfe~1 (the commit before nix/ was removed) finds one
/etc/faelight file for framework16, VERSION, and one for the rescue host. git grep -F /etc/faelight
at c92d8d58 (INT-250 filed) finds nine reads in .rs.

Correction, forward-only: INT-250 named five readers. Its filing commit shows nine; the four
novashell sites (commands/mod.rs:11038, :11125, :13350 and health_tui.rs:351) were not in its
table. All nine are gone: fsearch /etc/faelight 2026-10-01 finds no .rs file. INT-250 is not
retrofitted. The nine comment lines named under The Problem are gone too.

## Ruling 2026-10-01 (Christian): nothing goes to /etc/zero

Every file is per-user or retired. /etc/zero is not created. Ruled by Christian 2026-10-01.

Reasons, each measured 2026-10-01:

```text
    1  Pre-login is SDDM (systemctl status display-manager.service: sddm.service, /usr/bin/sddm).
       No Project 0 code runs before login. faelight-login does not exist; fsearch zero-login
       finds nothing.
    2  The daemon serves on the SESSION bus only: org.zero.Core, zero-daemon dbus.rs:240 and
       :260. The system bus is used only as a client, for logind PrepareForSleep (dbus.rs:287).
    3  No system service reads any of it (reason 2), and no zero unit exists for any reader
       (systemctl --user list-units zero*: 0 loaded units).
    4  Every piece of information already has a per-user owner: zero/meta/VERSION, the ledger,
       the doctor health file, git. A copy under /etc/zero would be a zombie.
    5  Nothing reads /etc/zero: fsearch /etc/zero finds it only in intents.
```

Order: INT-247 closed 2026-10-01. Steps 3 and 4 do not run on this outcome.

## Success Criteria

- [x] RECON recorded here: every file /etc/faelight held, its writer, its readers then, and what
      reads that information now -- measured from history, not recalled
<!-- evidence: demonstrated 2026-10-01, read-only. git grep -F environment.etc at deb94bfe~1 over nix/; git grep -F /etc/faelight at c92d8d58; git log -S/etc/faelight and -Sfaelight-export. Table in Recon 2026-10-01 above. -->
- [x] PER FILE, the ruling: system-wide (to /etc/zero) or per-user (stays in ~/.local/state/zero),
      with the reason
<!-- evidence: ruled by Christian 2026-10-01: every file per-user or retired, reasons 1 to 5 in Ruling 2026-10-01 above. -->
- [x] THE WRITER, for anything going to /etc/zero: declarative, run by the system -- not nsh, not a
      script holding sudo -- chosen by Christian, and rehearsed in a throwaway root first
<!-- evidence: closed by declining. Nothing goes to /etc/zero (Ruling, reasons 1 to 5), so there is no writer to choose or rehearse. No sudo, no system unit, no pacman hook was added. -->
- [x] NO ZOMBIE: every file under /etc/zero has one named writer and at least one reader, and a
      doctor check reports a file with no writer, or a reader of a missing file, as a failure
<!-- evidence: closed by declining. /etc/zero is not created (ls 2026-10-01: No such file or directory), so no file exists to lack a writer or a reader, and no doctor check is built for a directory that does not exist. fsearch /etc/zero 2026-10-01: intents only, so nothing reads a missing file. -->
- [x] UNREADABLE IS UNREADABLE: every reader of /etc/zero reports a file it cannot read as
      unreadable -- no "" and no 0 (INT-247's criterion, INT-250's lesson)
<!-- evidence: closed by declining. No reader of /etc/zero exists (fsearch /etc/zero 2026-10-01: intents only). The nine old /etc/faelight reads are gone (fsearch /etc/faelight 2026-10-01: no .rs file). -->
- [x] /etc/faelight appears in no live file (shared with INT-247's gate)
<!-- evidence: fsearch /etc/faelight 2026-10-01: hits only in zero/intents and zero/meta/CHANGELOG.md, history per AGENTS.md section 1. The same gate is ticked in INT-247 line 3559, evidence 2026-09-29. -->
- [x] OR: if nothing needs to be system-wide, the intent closes with /etc/zero not created and the
      reason recorded here
<!-- evidence: ls -la /etc/zero 2026-10-01: No such file or directory. Reasons 1 to 5 in Ruling 2026-10-01 above. -->

## Relationship

- INT-247 -- its /ETC/FAELIGHT gate; its comment pass rewords the nine comments
- INT-250 -- moved the readers off /etc/faelight; its notes are the recon's first source
- The 2025-12-14 lockout -- why nothing automated holds sudo
