---
id: 288
date: 2026-10-12
type: future
title: "SafeShell: a Rust login launcher that hands the session to nsh, or to bash when nsh cannot run"
status: planned
tags: [nsh, safeshell, login, chsh, recovery, launcher, resilience]
depends_on: [277]
---

## START HERE

Filed 2026-10-12 from Christian's SafeShell design notes. Waits on INT-277: its G8 answers whether
chsh happens at all, and its G2 builds the off switch SafeShell reads. Begin at G0 only after 277 is
complete. Bears on INT-286 (a silent death is what SafeShell exists to prevent at login).

## Vision

If nsh becomes the login shell, a broken nsh still never locks Christian out. The passwd shell is a
small Rust launcher with no dependency on bash or nsh. It decides which shell replaces it and execs
it; it never becomes the shell itself:
  explicit bash -> bash; nsh not runnable -> bash; current nsh fails its check -> known-good nsh;
  known-good fails -> bash; otherwise current nsh.
bash and sh stay untouched.

## The Problem

Today nsh reaches the user only through the exec at the end of ~/.bashrc (INT-277), and bash is the
passwd shell, so a broken nsh costs a terminal, never a login. chsh straight to nsh would make a bad
build a lockout at every tty, ssh and su - login. The design notes (2026-10-12) answer that with a
launcher. What they assume and the box does not have yet, to be measured or ruled before code:
- Root. The notes put safeshell and nsh in /usr/local/bin, config in /etc/safeshell, and run chsh.
  chsh refuses a shell /etc/shells does not list, and each of those paths needs root. No agent runs
  sudo: every root step is Christian's, written here as exact commands he runs by hand, and the rest
  is proved without root (a fake sudo on PATH, stand-in trees). Ruling: whether safeshell lives in
  bin_dir (~/.local/bin, where ship already installs), so the /etc/shells line and chsh are the only
  root steps.
- nsh's login contract. The notes need nsh -l, -i, -il, --version and --self-test; which exist is
  measured at G0. nsh -c exists but has diverged from bash before (the 2026-09-05 digit guard).
- Where SHELL goes. The systemd user manager supplies SHELL from passwd (277 recon), so after chsh
  every $SHELL -c caller in the session runs safeshell -c, and ssh host cmd does the same through
  sshd. The notes send -c to nsh -c; until nsh -c is proved against bash for those callers, where -c
  goes is a ruling, not a default.
- SDDM. wayland-session runs an unknown passwd shell through its *) branch: /bin/sh reads
  /etc/profile and ~/.profile, and .bash_profile and .bashrc stop running at graphical login (277 G8
  measures it). SafeShell changes what the whole graphical session inherits.
- Known-good. ship already keeps a versioned copy of each outgoing binary (0-core/bin, ship
  main.rs:417). Ruling: known-good is ship's last backup, not a second /usr/local/libexec tree.
- The escape. The notes add SAFESHELL_SHELL, but at login nothing has set it yet, so a variable only
  helps nested use. The login-time escape is a file: the same off switch 277 G2 builds, read by both.
- Cost. A self-test on every login is paid at every tty, ssh and session start, and a check that can
  hang must never block a login.

## Success Criteria

- [ ] G0 census, read-only, after INT-277 is complete. nsh's flags today (-c, -l, -i, --login,
      --version, --self-test); how ship names and keeps its backups; what 277 G2 built for the off
      switch; 277 G8's written chsh answer. If G8 answers no, this intent is cancelled with that
      answer quoted, not worked.
- [ ] G1 rulings, written here before any code: where safeshell is installed; how a login, an
      interactive and a -c invocation are told apart (argv0 with a leading dash, -c, -i) and which
      shell each goes to; the self-test's time budget and what a timeout means; where known-good
      comes from; the escape file's path, shared with 277's off switch.
- [ ] G2 nsh's login contract. Each flag G1 needs exists in nsh with an nsh-test case, red first
      for each one missing: -l and argv0 -nsh mean login, -i forces interactive, --self-test runs
      non-interactive checks and exits 0 or non-zero inside the budget.
- [ ] G3 the launcher, std only, no new crates. Tests with stand-in shells: --bash execs bash; a
      missing or non-executable nsh execs bash with one stderr line; an exec that fails falls back
      instead of exiting 127; a failing or hanging self-test (a stand-in that sleeps) falls to
      known-good, then bash; login, interactive and -c each reach the shell G1 ruled, with argv and
      argv0 preserved; --diagnose prints every decision it would make and execs nothing.
- [ ] G4 never a lockout, proved without chsh and without sudo: safeshell run as a login shell
      under env -i with argv0 -safeshell, against stand-ins for every row of G3's tree, twice; then
      SDDM's wayland-session run as 277 G8 runs it, with SHELL set to safeshell, diffed against bash.
- [ ] G5 ssh and $SHELL -c callers: ssh localhost and ssh localhost 'echo hello' behave as they do
      under bash, measured against a stand-in, or recorded as Christian's own manual check.
- [ ] G6 the switch-over, written as exact steps Christian runs by hand: the /etc/shells line, chsh,
      and how to undo both from a tty and from a live USB. No agent runs any of them. Proof is his
      pasted output, then a login through each door: tty, ssh, SDDM, a ghostty window.
- [ ] G7 doors: ship, nsh-test all passing, d 0 failed, core intent validate clean; a d check that
      the passwd shell is safeshell or bash and that its target runs (ruling, as in 277 G7).
- [ ] G8 each gate carries evidence per INT-158.

## Non-goals
- Any root step run by an agent. sudo, anywhere, is Christian's own hand.
- Changing bash, sh, /etc/profile or Omarchy's own files.
- SafeShell as a shell: it parses nothing and reads no rc.
- New crates.
- A /usr/local/libexec version tree: ship's stage-and-rename and its backups cover atomic installs
  unless G1 rules otherwise.

## Dependencies
depends_on: 277. Its G8 decides whether this intent runs at all; its G2 builds the off switch this
one reads.
