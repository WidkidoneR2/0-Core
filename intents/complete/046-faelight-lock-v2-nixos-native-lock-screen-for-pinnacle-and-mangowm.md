---
id: 046
date: 2026-06-08
type: feature
title: "faelight-lock v2: NixOS-native lock screen for Pinnacle and MangoWM"
status: complete
tags: [feature, rust, faelight, wayland, pam, mango, ext-session-lock, security]
version: TBD
---

## Vision
A native Rust Wayland screen locker for Faelight Forest on NixOS, built on the
ext-session-lock-v1 protocol so the compositor enforces the lock: input is
captured exclusively and a crash leaves the session locked, never exposed. The
locker (faelight-lock) renders the forest lock surface; a small isolated helper
(faelight-lock-auth) performs the PAM exchange. It is the Lock action behind
faelight-logout (INT-064).

## Why Now
INT-064 added the candy-neon power menu whose fourth tile is Lock -- but the
locker had been deleted during a brief hyprlock detour (commit b00a9975), so the
system had no screen lock at all on NixOS. faelight-logout's Lock tile is dead
without it. Recovering the proven v2 locker closes that gap and completes the
menu.

## Approach
- Recover the proven v2 source from b00a9975^ (rust-tools/faelight-lock): a
  smithay-client-toolkit ext-session-lock-v1 locker (main.rs) plus a separate,
  privilege-isolated PAM helper (auth.rs / faelight-lock-auth). No rewrite.
- Integrate into the NixOS workspace -- it builds via the rust-tools/* glob; one
  dead `let home` line removed for a zero-warning build on nixpkgs 26.05.
- Deploy as system binaries inside the faelight-forest package; the locker calls
  the helper at its absolute system path. The PAM service
  security.pam.services.faelight-lock was already declared and is reused --
  pam_unix/unix_chkpwd does the privileged check, so no manual setuid was needed.
- Compositor-agnostic by protocol, so it works under MangoWM today and Pinnacle
  inherits the same path.

## Success Criteria
- [x] faelight-lock + faelight-lock-auth build clean on NixOS (release, zero warnings)
- [x] Locker locks the session and unlocks with the correct password via PAM
- [x] faelight-logout's Lock tile spawns it and it works end to end

## Gate Check
✅ Recovered v2 from b00a9975^ and builds clean on NixOS -- release build, zero warnings, both binaries

✅ Locks and unlocks via PAM -- faelight-lock-auth returns OK for the correct password; live lock/unlock confirmed under MangoWM

✅ Wired into faelight-logout -- Lock tile spawns faelight-lock, verified standalone and through the menu

## Future
GTK4 reskin (candy-neon, matching faelight-logout) is a possible later project
via gtk-session-lock. The recovered Rust locker is the secure baseline; revisit
the reskin only if it earns the effort.

## Depends On
  INT-064 (faelight-logout -- the Lock tile this locker serves)

---

*"The forest grows with intention."* 🌲
