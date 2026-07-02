---
id: 081
date: 2026-06-23
type: feature
title: "fsh reload thinks Nix: hot-swap the rebuilt binary"
status: complete
tags: [fsh, faelight-shell, nixos, reload, exec, lane0, hot-swap]
priority: high
---
## Why
Roadmap Lane 0 papercut: `exec fsh` does not hot-swap the rebuilt binary -- must
close+reopen the terminal to pick up a new fsh after a rebuild. This slows ALL fsh work
(every shell change forces a terminal restart), so it is the meta-bug to fix before the
Rust-native feature lanes.

ROOT CAUSE (read-confirmed 2026-06-23): both the `exec` builtin (exec_cmd, mod.rs:9301-9304)
and the `reload` arm (mod.rs:486) resolve the target via std::env::current_exe(). On Arch
that worked by accident -- current_exe() pointed at /usr/bin/fsh and a rebuild overwrote
that file in place, so "re-exec myself" picked up the new bytes. On NixOS the running
binary lives at an IMMUTABLE, content-addressed store path
(/nix/store/<hash>-faelight-forest/bin/.faelight-shell-wrapped). A rebuild produces a NEW
store path with a NEW hash; the old one is never overwritten. So current_exe() re-execs the
SAME OLD binary, by definition -- it can never pick up a new one. This is the Arch mental
model baked into the code.

THE FIX ALREADY EXISTS in this codebase: the /tmp/fsh-reload-signal handler (main.rs:828-841)
does it correctly -- it tries /run/current-system/sw/bin/faelight-shell FIRST and only falls
back to current_exe(). That Nix-native pattern just was never wired into the `exec`/`reload`
builtins. This intent applies the proven pattern to the two commands the user actually types.

## Evidence baseline (2026-06-23, read-only)
- exec_cmd (mod.rs:9293) + reload arm (mod.rs:485) both use std::env::current_exe().
- main.rs:828 reload-signal handler already resolves /run/current-system/sw/bin first.
- Reboot to gen 213 cleared generation drift: booted-system == current-system now, so
  a real hot-swap test is meaningful (no stale-generation false negative).

## What
Make `reload` (and `exec fsh`) resolve the fsh binary the NixOS way: try
/run/current-system/sw/bin/faelight-shell FIRST, current_exe() only as fallback. Add honest
Nix-awareness: if the resolved current-system binary is the SAME store path already running
(nothing new deployed), say so instead of silently re-execing the same bytes. Rust change to
faelight-shell -> rebuild -> deploy. Daily-driver shell core: back up every file, cargo check
before build, nixos-rebuild build (not switch) to verify clean first, keep a fresh terminal
as the instant known-good escape throughout.

## Gates
- [x] G1: `reload` and `exec fsh` resolve via /run/current-system/sw/bin/faelight-shell
      FIRST, current_exe() as fallback (the main.rs:828 pattern, factored into a shared
      helper). cargo check + release build clean. Every edited file backed up.
- [x] G2: Nix-honest behavior -- if the current-system fsh is the SAME store path already
      running (nothing new to swap to), report it clearly ("already running the current
      fsh") rather than a silent same-binary re-exec.
- [x] G3: verified live -- make a visible change to fsh, rebuild + deploy, type `reload`
      in an EXISTING terminal, and confirm it comes up as the NEW binary (distinct store
      path / build marker) WITHOUT closing and reopening the terminal.

## Notes
- Scope: faelight-shell only (exec_cmd + reload arm + a shared resolver helper). No change
  to the /tmp/fsh-reload-signal handler beyond possibly sharing the helper.
- Safety: shell core. A fresh terminal always yields a known-good fsh, so we can never get
  stranded in a broken shell. Revert = cp the .bak + rebuild.
- Bootstrap nicety: G3 proves the fix by using a rebuild to deploy a new fsh and watching
  the NEW reload mechanism pick itself up -- the fix tests itself.
- Relationship to roadmap: unblocks the faster fsh edit loop that Lane 3 (Rust-native) work
  will lean on.


## Evidence log
### 2026-06-23 -- G1+G2+G3 DEMONSTRATED
Added resolve_fsh_binary() (current-system-first candidate list, current_exe() fallback)
and reload_fsh() (canonicalize same-path check -> honest 'already current' message).
Rewired reload arm + exec_cmd self-case. Backups mod.rs.bak-20260623T174237 / -174328.
cargo check clean (after 1-char |_| closure fix), release build clean, nixos-rebuild build
clean, deploy 5/6 ADVISORY. PROVEN live via all-process /proc hash probe:
  BEFORE reload: running f930dsz (old), current-system -> q2cnckm (new) [different].
  AFTER reload : single fsh PID running q2cnckm (new); old f930 process gone; terminal
  never closed. Hot-swap confirmed in-place. (Splash banner alone was ambiguous; the
  hash probe was the decisive evidence -- earlier heredoc probes were fooled by bash
  subshell nesting, not a swap failure.)
All 3 gates met.

## Outcome
`reload` (and `exec fsh`) now follow /run/current-system to the freshly-deployed binary
instead of re-execing their own immutable store path. fsh rebuilds hot-swap in place --
no more close+reopen. Unblocks the faster fsh edit loop for Lane 3 (Rust-native) work.

## The Rule
"On Nix, the binary moves. Reload must follow it, not re-run itself." 🌲
