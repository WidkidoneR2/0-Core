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

## Progress (2026-07-10) -- SOLVED, pending commit
The gap was NOT two libs but SEVEN. Found by letting the compiler name each one in turn
(never guessed; every Nix attribute confirmed against nixpkgs before adding). All seven
added to the friday-dev devShell buildInputs in flake.nix:

  1. udev            -- libudev.pc      (libudev-sys / smithay backend_udev)
  2. libxkbcommon    -- xkbcommon.pc    (smithay-client-toolkit)
  3. seatd           -- libseat.pc      (libseat-sys / backend_session_libseat)
  4. libdisplay-info -- libdisplay-info.pc (libdisplay-info-sys)
  5. pam             -- security/pam_appl.h header (pam-sys / faelight-login)
  6. libinput        -- -linput link    (faelight-compositor / backend_libinput)
  7. libgbm          -- -lgbm link      (faelight-compositor / backend_gbm)

All seven already appear in the PACKAGE build inputs (flake.nix:39,46) -- the devShell was
simply missing what the package build already had.

PROVEN LIVE (fresh `nix develop`, 2026-07-10):
- pkg-config --exists libudev AND xkbcommon -> both OK.
- `cargo nextest list` -> entire workspace COMPILES AND LINKS, incl. faelight-compositor
  (the last + hardest crate). nextest listed real tests (faelight-shell pipeline tests,
  faelight-update tests, etc). "Finished test profile" -- no panics, no link errors.

Not yet committed at time of this note. Next: commit flake + charter, then tick gates.

## Success Criteria
- [x] `udev` AND `libxkbcommon` added to friday-dev devShell buildInputs in flake.nix <!-- 2026-07-10: all SEVEN added (udev, libxkbcommon, seatd, libdisplay-info, pam, libinput, libgbm). Commit 3b295a44 -->
- [x] Inside `nix develop`: `pkg-config --exists libudev` AND `pkg-config --exists xkbcommon` both succeed <!-- 2026-07-10: both returned OK in fresh nix develop -->
- [x] `libudev-sys` AND `smithay-client-toolkit` build scripts no longer panic (both compile past) <!-- 2026-07-10: both compiled past; ENTIRE workspace compiles+links incl. faelight-compositor -->
- [x] `bacon` completes a build cycle in-shell with no missing-system-library error <!-- 2026-07-10: ran bacon in fixed devShell, all green, 0 errors, no missing-lib panic -->
- [x] DEMONSTRATED live on the running shell, not assumed -- output pasted into this intent <!-- 2026-07-10: cargo nextest list + bacon output pasted live this session -->

## Relationship
- Surfaced by INT-130 (028 reconciliation, bacon gate). 028's bacon gate closes on bacon's
  own function; THIS intent owns making the workspace actually compile in-shell.
- Related to any future compositor build work (smithay backends).

## Notes
- Root confirmed, not theorized: smithay at Cargo.toml:44 (backend_udev, backend_session_libseat).
  devShell at flake.nix:258. Missing inputs = udev + libxkbcommon. Fix is two lines in buildInputs.
- `bacon` is NOT broken -- it correctly detected and reported the failure. Do not "fix" bacon.
