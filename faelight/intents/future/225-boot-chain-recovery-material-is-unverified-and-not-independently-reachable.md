---
id: 225
date: 2026-08-17
type: fix
title: "boot chain recovery material is unverified and not independently reachable"
status: planned
priority: high
depends_on: []
tags: [fix, bugfix]
---

## Vision

The boot chain's recovery path is reachable without the boot chain, and someone has proven it by
using it.

## The Problem

Measured 2026-08-17. **The boot chain is healthy. Its recovery material is not independently
recoverable.**

- Secure Boot enabled (user), TPM2 present, Measured UKI yes, `PK`/`KEK`/`db`/`custom` all present
  in `/var/lib/sbctl/keys`, firmware menu reachable via `systemctl reboot --firmware-setup`.
- ⚠️ **The signing keys exist in exactly one place: `/var/lib/sbctl`, on the LUKS-encrypted root.**
  They sign every UKI. If the disk or the LUKS header is lost, no bootable signed image can be
  produced.
- ⚠️ **The rescue USB is REJECTED while Secure Boot is enforcing** -- measured twice, and the second
  time it silently fell through and booted the signed disk, so the media appears to do nothing at
  all. A user at 3am would conclude the stick was blank.
- ⚠️ The documented path is therefore firmware menu -> supervisor credential -> factory reset ->
  re-enrolment. **That depends on firmware access and a password, neither of which the system can
  guarantee.**
- ⚠️ **FORESTBACKUP was not attached, so the sbctl and EFI-variable backups could not be verified.
  Status is UNKNOWN, which is the highest-risk state there is** -- "we have a backup" and "we have a
  backup that works" are currently indistinguishable.

Decision 146 states the invariant this violates: **a lockout-class component must have a recovery
path that does not depend on that component being functional.**

⚠️ INT-161 is COMPLETE and is not being reopened. INT-158 is forward-only. The justification for
this intent is not ledger hygiene -- it is that the invariant which exposes this did not exist when
161 closed, and recovery-independence of the boot chain is a live safety question.

## The Solution

Verify, test, record. Nothing about Lanzaboote changes.

★ Two items collapse into one sequence: **the `efibootmgr -v` dump IS the EFI-variable backup, and
it is also the record that makes removing the dangling entry safe.** Backup first, delete second,
never the reverse.

## Success Criteria

### Establish what exists

- [ ] FORESTBACKUP is attached and its contents inventoried. Whatever is or is not on it is
      recorded here as a fact rather than an assumption.
- [ ] ★ **Boot counting / automatic boot assessment: determined enabled or not.** systemd-boot
      reports the feature as supported. ⚠️ Do this FIRST -- if it is available it is the only
      genuinely independent recovery mechanism in the chain, needing no human, no firmware menu and
      no keys, and it changes how much weight everything below carries.
- [ ] A full `efibootmgr -v` dump is taken and stored outside the encrypted root. **This is the
      EFI-variable backup.**

### Prove the recovery material actually works

- [ ] ⚠️ **`/var/lib/sbctl` is backed up AND RESTORE-TESTED** -- restored to a scratch location and
      the key material confirmed to match. **A backup that has never been restored is unverified,
      not present.**
- [ ] ★★ **THE GATE THAT MATTERS MOST: the supervisor password is retrievable WITHOUT this machine.**
      ⚠️ If it lives only in a KeePass database on the encrypted root, the recovery path depends on
      the failed system and the whole chain is circular. Where it lives is recorded; the secret
      itself is not.
- [ ] The `recovery_verified` date for the boot chain is recorded in `RISK.toml`, per decision 146.
      Until this gate closes, the field is absent and that absence is correct.

### Write down what cannot be automated

- [ ] The firmware recovery dependency is documented in `docs/recovery-runbook.md`: firmware menu,
      supervisor credential, factory reset, re-enrolment.
- [ ] ⚠️ **The rescue-USB limitation is documented explicitly** -- under Secure Boot enforcement the
      stick is rejected and the machine silently boots normally, so *the media appears to do
      nothing*. Anyone reaching for it in an emergency must know to disable Secure Boot first.
- [ ] The runbook change is verified against the ISO, not just the working tree. ⚠️ The runbook
      shipped two dead paths inside the rescue image once already, and nobody walks a runbook until
      they need it.

### Cleanup, last and deliberately

- [ ] The dangling `Boot0004 "Windows Boot Manager"` entry is removed. It points at a partuuid that
      does not appear in `lsblk` at all -- a leftover from before the NixOS install.
      ⚠️ **Lockout-class by ADJACENCY: `efibootmgr -B` operates on the same NVRAM store as
      `Boot0003 "Linux Boot Manager"`, so a one-digit slip deletes the live boot entry.** Only after
      the dump above exists. Reboot and confirm the system still boots before closing this gate.

## Prior art -- do not duplicate

- **decisions/146** -- the invariant, the activation contract, `recovery_verified`
- **decisions/145** -- the ownership model this sits under
- **INT-161** -- Lanzaboote on metal, COMPLETE. Not reopened.
- **INT-160** -- the rescue USB, COMPLETE. Its gate 7 proved LUKS unlock, mounts, chroot and
  rollback from the media -- with Secure Boot NOT enforcing at the time.
- **INT-059** -- the VM rehearsal, where the Access Denied behaviour was first measured
- **INT-192** -- tools that cannot express UNDETERMINED, which is why "unknown" needs a field

## Non-goals

- Changing anything about Lanzaboote. The chain is healthy.
- Disabling or weakening Secure Boot.
- Regenerating, rotating or moving the signing keys.
- Rebuilding the rescue ISO. Its content is fine; what is missing is a documented understanding of
  when it can and cannot be used.

## Risk

`system` for everything except the final cleanup gate, which is **`critical`** -- it writes to EFI
NVRAM and sits one digit away from the live boot entry.

⚠️ Per `nix/profiles/RISK.toml`'s own promotion rule -- *promote to critical if it ever carries
boot, login, or disk settings* -- that gate is done last, alone, after the dump exists, and with a
reboot to confirm before it is ticked.
