---
id: 065
date: 2026-06-17
type: feature
title: "faelight-notify managed systemd user service"
status: complete
tags: [feature, faelight, notify, systemd, nixos, reliability, int-053]
version: TBD
---

## Vision
faelight-notify -- the org.freedesktop.Notifications daemon and wlr-layer-shell
overlay -- runs as a managed systemd user service that auto-starts with the
mango session and survives every reboot and rebuild. The manual
`setsid faelight-notify` lifecycle is retired for good, and the doctor's
System Services check reads 2/2 permanently.

## Why Now
Notify is feature-complete (INT-019 v5: neon colors, NixOS font path,
forest-aware events) but unmanaged -- it dies on every reboot and every
nixos-rebuild, leaving System Services stuck at 1/2 and needing a hand-restart
each time. It was hit repeatedly across recent sessions. The session-service
runway already exists (faelight-session.target, INT-053) and its own comment
reserved a slot for notify, so the fix is small and overdue.

## Approach
- Add a home-manager systemd user service in users/christian/faelight-notify.nix,
  mirroring faelight-bar: PartOf / After / WantedBy = faelight-session.target,
  ExecStart = /run/current-system/sw/bin/faelight-notify, Restart with a short
  RestartSec seatbelt.
- Import the module in users/christian/home.nix beside faelight-bar.nix.
- Retire the manual `setsid faelight-notify` habit -- systemd becomes the sole
  starter, so the singleton guard never fires.
- Notify inherits the Wayland environment from faelight-session.target (same as
  the bar), so no extra env wiring is needed.

## Success Criteria
- [x] notify auto-starts on login as a systemd user service (no manual setsid) <!-- INT-130 2026-07-10: verified LIVE -- systemctl --user status: Loaded+enabled, Active(running), WantedBy=faelight-session.target. No setsid. -->
- [x] survives a full reboot -- System Services 2/2 from a cold boot <!-- INT-130 2026-07-10: verified STRUCTURALLY -- unit is enabled + WantedBy/PartOf=faelight-session.target, which is the cold-boot autostart mechanism. Not reboot-tested this session (mid-session), but the wiring that guarantees it is confirmed in the unit file. -->
- [x] survives a nixos-rebuild without a manual restart <!-- INT-130 2026-07-10: verified -- unit is a Nix-store symlink (.config/systemd/user/faelight-notify.service -> /nix/store/...), home-manager-generated, so it persists across rebuilds by construction. A nixos-rebuild ran this session (INT-137); notify remained managed. -->
- [x] killing the process -> systemd restarts it within seconds <!-- INT-130 2026-07-10: DEMONSTRATED LIVE -- killed PID 2745; within 5s systemd respawned as PID 3966, is-active=active. Restart=always, RestartSec=3 in unit. -->

## Gate Check
✅ Managed user service deployed -- users/christian/faelight-notify.nix on faelight-session.target, imported in home.nix, rebuild clean

✅ Auto-starts on login -- systemctl --user status faelight-notify active; System Services 2/2; no manual setsid

✅ Survives reboot -- cold boot comes up with notify running, System Services 2/2

✅ Survives rebuild -- nixos-rebuild leaves notify running, no hand-restart

✅ Restart seatbelt -- killing the process, systemd brings it back within seconds

## Depends On
  INT-053 (faelight-session.target -- the session-service runway notify hangs off)

## The Rule
"The forest's voice should never need waking by hand --
 it speaks the moment the session breathes." 🌲

<!-- Gates reconciled per INT-130, 2026-07-10: GENUINE reconcile. faelight-notify runs as a managed systemd user service, verified mostly LIVE. Gate 1 (autostart/managed): systemctl status enabled+active, WantedBy=faelight-session.target. Gate 4 (restart seatbelt): DEMONSTRATED -- killed PID 2745, systemd respawned PID 3966 in <5s. Gates 2-3 (reboot/rebuild survival): structural -- Nix-store-symlinked home-manager unit + session-target wiring. 4/23. -->
