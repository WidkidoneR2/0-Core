---
id: 137
date: 2026-07-10
type: future
title: "friday-dev devShell missing libudev -- smithay backend_udev cannot compile in-shell"
status: planned
tags: [nix, devshell, smithay, libudev, build]
---

## Vision
`nix develop` (friday-dev) can build the ENTIRE workspace clean -- including the smithay
compositor crate -- so `bacon`, `cargo build`, and `cargo nextest` all succeed in-shell
without a missing system library. A dev shell that cannot compile the project is not a
complete dev shell.

## The Problem
smithay (Cargo.toml:44) is pulled in with `backend_udev` + `backend_session_libseat`,
which transitively require the `libudev-sys` crate. `libudev-sys`'s build script calls
`pkg-config --libs --cflags libudev` and PANICS when `libudev.pc` is not found.

The friday-dev devShell (flake.nix:258-284) provides pkg-config, clang, cmake, openssl,
sqlite, python3 -- but NOT udev. So any build that touches smithay fails inside the shell:

    error: failed to run custom build command for `libudev-sys v0.1.4`
    The system library `libudev` required by crate `libudev-sys` was not found.
    HINT: install a package such as libudev / libudev-dev / libudev-devel.

SECOND GAP (found 2026-07-10 via `cargo nextest list`): smithay-client-toolkit ALSO
panics on a missing `xkbcommon` (xkbcommon.pc not found) -- same class of bug, same
devShell. So the gap is TWO system libs, not one: udev AND xkbcommon.

Discovered 2026-07-10 during INT-130 / INT-028 reconciliation: `bacon` ran correctly
(watched + triggered a build -- bacon itself is NOT at fault) and surfaced this compile
failure. Scope was deliberately kept out of 028; filed here instead.

## The Solution
Add the udev library to the devShell's buildInputs so pkg-config can resolve libudev.pc.

- In flake.nix devShells.${system}.default buildInputs, add `udev` (pkgs.udev provides
  libudev.pc) AND `libxkbcommon` (provides xkbcommon.pc). Place near pkg-config/openssl.
- Rebuild the devShell (exit + re-enter `nix develop`, or `rebuild` if wired system-side).
- Confirm pkg-config sees it: `pkg-config --exists libudev && echo OK`.
- Confirm the real fix: smithay compiles in-shell (bacon / cargo build reaches completion,
  no libudev-sys panic).

## Success Criteria
- [ ] `udev` AND `libxkbcommon` added to friday-dev devShell buildInputs in flake.nix
- [ ] Inside `nix develop`: `pkg-config --exists libudev` AND `pkg-config --exists xkbcommon` both succeed
- [ ] `libudev-sys` AND `smithay-client-toolkit` build scripts no longer panic (both compile past)
- [ ] `bacon` completes a build cycle in-shell with no missing-system-library error
- [ ] DEMONSTRATED live on the running shell, not assumed -- output pasted into this intent

## Relationship
- Surfaced by INT-130 (028 reconciliation, bacon gate). 028's bacon gate closes on bacon's
  own function; THIS intent owns making the workspace actually compile in-shell.
- Related to any future compositor build work (smithay backends).

## Notes
- Root confirmed, not theorized: smithay at Cargo.toml:44 (backend_udev, backend_session_libseat).
  devShell at flake.nix:258. Missing inputs = udev + libxkbcommon. Fix is two lines in buildInputs.
- `bacon` is NOT broken -- it correctly detected and reported the failure. Do not "fix" bacon.
