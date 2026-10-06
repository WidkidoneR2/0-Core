---
id: 277
date: 2026-10-05
type: future
title: "Prove the bash-to-nsh session path on Omarchy and answer chsh"
status: planned
tags: [nsh, omarchy, login, session, resilience, bashrc, handoff, chsh, sddm, uwsm, ghostty, mise, recovery]
depends_on: []
---

## Vision
nsh is the shell in every terminal on Omarchy, and a broken, missing or
unwanted nsh never costs a prompt: a working bash is always one step away,
by design and by demonstration. Whether nsh should become the login shell
(chsh) is answered in writing from measurements on this machine, not from
outside reviews.

## The Problem
nsh reaches the user through one path, measured 2026-10-05 on Omarchy
4.0.4-1: SDDM autologin -> omarchy.desktop -> uwsm -> systemd --user unit
wayland-wm@hyprland.desktop -> start-hyprland -> Hyprland -> ghostty ->
bash (passwd shell) -> ~/.bashrc -> exec ~/.local/bin/nsh.

The session layer is safe: SDDM, uwsm and Hyprland never run nsh or the
interactive rc. The shell layer is not proven:

- A build that panics at startup closes every ghostty window AND every tty
  login (a tty login is interactive bash -> .bash_profile -> .bashrc ->
  exec nsh). The -x guard only catches a missing binary, so the handoff
  comment "a broken build costs a prompt here" holds for a missing build
  only. Known by reading; G2 demonstrates it.
- There is no deliberate off switch and no always-bash door.
- Omarchy's rc sets up bash hooks before the exec that never reach nsh:
  mise activate (its install dirs sit in nsh's PATH ahead of the shims, so
  per-project versions go stale after cd), zoxide, try, fzf bindings, bash
  completions. MISE_SHELL=bash and STARSHIP_SHELL=bash leak into nsh's env.
- ghostty's bash integration is discarded at the exec: the ssh() wrapper
  (TERM fallback, SetEnv/SendEnv), OSC 133 marks, OSC 7 cwd, OSC 2 title.
  ssh from nsh sends TERM=xterm-ghostty to hosts that may lack terminfo.
- Update surface: omarchy-install-dev-env and omarchy-install-editor-helix
  append lines to ~/.bashrc below the exec, where they never run;
  omarchy-reinstall-configs overwrites ~/.bashrc and ~/.bash_profile from
  /etc/skel with no backup, removing the handoff. Nothing reports either.
- Outside reviews asked whether nsh is ready for chsh. nsh has never been
  the passwd shell on any machine.

## The Solution
Keep bash in /etc/passwd. Make the handoff in ~/.bashrc (Christian's file)
the product: a written contract, a crash fallback, an off switch and an
always-bash door, each demonstrated. Close the integration gaps the exec
creates, or record each as a finding. Add a d check that the handoff is
wired. Measure what chsh would change, without chsh and without sudo, and
answer it in writing.

## Recon (G0 evidence, collected 2026-10-05 before cistart)
- passwd /usr/bin/bash; /bin/sh -> /usr/bin/bash; nsh not in /etc/shells;
  $SHELL inside nsh is /usr/bin/bash, supplied by the systemd user manager
  from passwd (`systemctl --user show-environment`).
- SDDM: autologin.conf User=christian, Session=omarchy.desktop in
  /usr/local/share/wayland-sessions, Exec
  `uwsm start -g -1 -e -D Hyprland hyprland.desktop`. wayland-session runs
  a bash passwd shell as `bash --login` (non-interactive); an unknown shell
  takes its *) branch (/bin/sh reads /etc/profile and ~/.profile, the
  passwd shell never runs).
- uwsm: wayland-wm@hyprland.desktop.service runs start-hyprland;
  wayland-wm-env@ prepares env via `uwsm aux prepare-env`; env.d/10-omarchy
  sources env-bootstrap, sets TERMINAL and EDITOR, runs
  `mise activate bash --shims`. main.py never_export includes SHELL, TERM,
  COLORTERM, SHLVL; filter_varnames drops SHELL; no shell launch sites.
- Profile chain: .bash_profile only sources .bashrc; .profile and
  .bash_login absent; /etc/profile.d/omarchy.sh sources
  /usr/share/omarchy/default/bash/env-bootstrap (POSIX sh: OMARCHY_PATH,
  mise shims, ~/.local/bin).
- ~/.bashrc: line 5 returns for non-interactive shells; line 9 sources
  Omarchy's rc (envs, shell, aliases, functions, init); line 22 cargo PATH;
  lines 43-46 the handoff (interactive, NSH_ACTIVE unset,
  -x ~/.local/bin/nsh -> export NSH_ACTIVE=1; exec). Lines 29-31 cite
  INT-190 and niri-session (INT-190 is cancelled; it was about
  shell_history writers).
- ghostty: no command= (runs the passwd shell);
  shell-integration-features = no-cursor,ssh-env; GHOSTTY_SHELL_FEATURES
  inside nsh = path,ssh-env,title.
- nsh PATH: ~/.cargo/bin, five mise install dirs (claude, codex, gh,
  node 26.7.0, opencode), mise shims, /usr/share/omarchy/bin,
  /usr/local/sbin, /usr/local/bin, /usr/bin, ~/.local/bin, perl dirs.
- Updates: 0 migrations mention bashrc (4.0.4); omarchy-upgrade-to-quattro
  backs up .bashrc and only rewrites old rc-source lines in place (this
  .bashrc would be left unchanged); omarchy-reinstall-configs is
  `cp -af /etc/skel/. ~/`.
- Outside-review ledger. Confirmed: profile.d/omarchy.sh -> env-bootstrap
  sets OMARCHY_PATH and PATH; session start does not run the interactive
  rc; env-bootstrap is plain POSIX sh; the session goes through uwsm.
  Absent: "maintainers advise against chsh" (nothing on the box says so).
  False: INT-190 is the gate.

## Success Criteria
- [ ] G0 Recon the login and terminal chain, read-only. Evidence is the
      ## Recon section above (five rounds, 2026-10-05). Tick after cistart.
- [ ] G1 Handoff contract written here before any edit to ~/.bashrc, each
      row demonstrated:
      login bash (bash --login, non-interactive) never reaches the handoff;
      bash -c never reaches it; a terminal opened from Hyprland lands in nsh;
      `bash` typed inside nsh stays bash (NSH_ACTIVE);
      a terminal launched from inside nsh inherits NSH_ACTIVE and lands in
      bash -- record whether that is wanted;
      nsh's PATH holds the OMARCHY_PATH entries, mise shims, ~/.local/bin
      and ~/.cargo/bin;
      each export and bash hook Omarchy's rc sets before the exec (mise,
      starship, zoxide, try, fzf, completions) carries into nsh, has an nsh
      equivalent, or is recorded as a gap -- the stale mise install dirs
      ahead of the shims first.
      The handoff comment's INT-190/niri story is proposed for correction;
      Christian edits his own file.
      Proof: env -i HOME="$HOME" TERM="$TERM" bash --login -c 'echo reached'
      prints reached (an exec into nsh would never run the -c string); one
      line of evidence per row.
- [ ] G2 A broken or unwanted nsh never costs a prompt. Rulings before
      work: the crash mechanism -- (a) nsh as a child of the interactive
      bash, staying in bash on panic (101) or signal (>128), else exiting
      with its status, or (b) exec plus a crash stamp in ~/.local/state;
      the off-switch file location; whether the always-bash door is a
      Hyprland bind or a launcher entry.
      Rows: crash fallback; off switch (a file the handoff checks, one
      command off, one command on); always-bash door (a launch that sets
      NSH_ACTIVE=1).
      Proof, red first, never against the live ~/.bashrc or the shipped
      binary: a copy of the handoff run as `bash --rcfile <copy> -i` from
      inside a running nsh, pointed at a stand-in that dies before the
      prompt -- today the stand-in takes the shell down; after, it lands in
      bash with one stderr line naming why; the real nsh lands in nsh; the
      off file present lands in bash; the door lands in bash with the
      stand-in still broken. Run twice.
- [ ] G3 Startup resilience, isolated through the state-dir and state-db
      overrides (names confirmed at G3 recon; never the live state.db).
      nsh reaches a prompt, or exits non-zero fast enough for G2 to catch
      it, under: corrupt state.db; read-only state dir; config.nsh syntax
      error; empty HOME; launch cwd deleted; PATH=/usr/bin only; TERM=dumb;
      core absent from PATH. Proof: one nsh-test Repl case per row, green
      on the shipped binary; a failing row is recorded as a finding before
      it is fixed.
- [ ] G4 exec resolves PATH. Red 2026-09-05: `exec nsh` -> No such file.
      Also: exec of a missing command in interactive nsh reports and keeps
      the shell. Proof: red capture, fix, nsh-test case, green.
- [ ] G5 Terminal and job-control matrix in the pty harness: Ctrl-C ends
      the foreground child, not the shell; Ctrl-Z stops it, jobs shows it
      stopped, fg resumes it with the terminal, bg continues it;
      `yes | head -3` returns and the shell survives; closing the terminal
      with a background job does what NOVASHELL.md says. Proof: one Repl
      case per row.
- [ ] G6 Ghostty integration parity. ghostty runs bash and injects its
      integration; the exec to nsh discards it. Per item, nsh provides it
      or the gap is recorded as a finding: ssh (TERM fallback and
      SetEnv/SendEnv) first, then OSC 7 cwd for new windows and splits,
      OSC 2 title, OSC 133 prompt marks. Proof: each item observed in a
      real ghostty window.
- [ ] G7 The handoff survives Omarchy updates and installers. Measured on
      4.0.4: no migration touches ~/.bashrc; the quattro upgrade leaves this
      .bashrc unchanged; omarchy-reinstall-configs overwrites it with no
      backup; two installers append below the exec. Rows: a d check reports
      the handoff present and its target executable (built on Christian's
      ruling; watched failing against a copy with the block removed, then
      passing); appended lines below the exec are either reported by that
      check or the handoff is moved so it runs after the whole rc (ruling).
- [ ] G8 The chsh impact map, measured without chsh and without sudo. Run
      SDDM's own script once with each shell and diff what it hands the
      session:
        env -i HOME="$HOME" USER="$USER" LOGNAME="$USER"
          PATH=/usr/local/sbin:/usr/local/bin:/usr/bin SHELL=<shell>
          /bin/sh /usr/share/sddm/scripts/wayland-session env
      with /usr/bin/bash and with ~/.local/bin/nsh. Already recorded: uwsm
      never launches a shell and drops SHELL; the systemd user manager
      supplies SHELL from passwd, so chsh changes SHELL for every session
      app and every $SHELL -c caller would go through nsh -c (re-measure the
      2026-09-05 -c digit-guard divergence first); tty, ssh, su - and
      sudo -i would run nsh itself as a login shell (argv0 -nsh) with no
      login read; .bash_profile/.bashrc stop running at SDDM login.
      Default answer stays bash in /etc/passwd. Proof: both env outputs and
      the diff pasted here, and the written answer.
- [ ] G9 Doors: nsh on PATH, nsh-test all passing, d 0 failed,
      core intent validate clean; START HERE record; NOVASHELL.md and
      AGENTS.md changes proposed, not made.

## Non-goals
- chsh or any /etc/passwd change
- SafeShell or any wrapper binary in /etc/passwd (the login chain never
  needs one; the off switch and the always-bash door cover the wish
  behind it)
- nsh reading /etc/profile or Omarchy's bash scripts
- modifying Omarchy's own files (/usr/share/omarchy, /etc,
  /usr/local/share/wayland-sessions)
- test user accounts (they need sudo)
- the per-segment safety-guard defect, the -c digit-guard fix and the
  main.rs refactor (separate findings)
- sudo, anywhere

## Dependencies
depends_on is empty: nothing must finish first. INT-262 closed on
2026-10-06, so nothing holds cistart.

## START HERE
Written 2026-10-06; supersedes the 2026-10-05 entry. INT-262 is complete:
nsh 5.1.0 and nsh-test 2.0.1 shipped, everything pushed through be041ac1,
nsh-test 226/226, d 28/28. Nothing has been done on 277 yet.

Next, in order:
1. cistart 277. The file moves out of future/, so take its new path from
   cistart's output before any payload names it.
2. Tick G0 with an evidence comment pointing at ## Recon (five rounds,
   2026-10-05). If Omarchy has updated past 4.0.4-1 since, re-run the
   recon rows that read Omarchy's files before ticking.
3. Take the three G2 rulings before any work:
   - the crash mechanism: (a) nsh as a child of the interactive bash,
     staying in bash on panic (101) or signal (>128), else exiting with
     its status -- recommended, because the fallback shows in the same
     window; or (b) exec plus a crash stamp in ~/.local/state, where the
     failure only shows on the next terminal;
   - where the off-switch file lives;
   - whether the always-bash door is a Hyprland bind or a launcher entry.
4. Write G1's contract here before anything touches ~/.bashrc, nsh or
   Omarchy. Christian edits ~/.bashrc himself.

A version bump touches four places: Cargo.toml, Cargo.lock, any copy
hardcoded in source, and zero/registry/tools.toml. nsh-test 2.0.1 missed
the registry until core integrity proposal #22 caught it (2026-10-06).

Carried method: a payload that spans two steps finds its first step by a
marker that survives rustfmt, never by its bytes; one ship at the end of
a fix series; after ship, reload with a second window open.
