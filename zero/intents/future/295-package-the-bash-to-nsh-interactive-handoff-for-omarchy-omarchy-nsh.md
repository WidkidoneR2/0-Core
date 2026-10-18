---
id: 295
date: 2026-10-18
type: future
title: "Package the bash-to-nsh interactive handoff for Omarchy (omarchy-nsh)"
status: planned
tags: [nsh, omarchy, handoff, package, terminal, resilience]
depends_on: [277]
---

## Vision
Turning nsh on as the interactive shell on Omarchy is one command, turning it off is one command,
and the files that do it live outside Omarchy's own, so an Omarchy update or a config reset does
not silently undo them. bash stays the passwd shell. The handoff is the one INT-277 proves and
hardens, not a second design. A broken nsh costs a prompt, never a terminal, a tty or a session.
Whether the form is an install script, a package built from this repo, or a contribution to
Omarchy is decided at G1 by measurement and by Christian, not assumed.

## The Problem
Measured 2026-10-18 on Omarchy 4.0.4.

- Today the handoff is four lines Christian wrote by hand at the end of ~/.bashrc (lines 43-46):
  interactive, NSH_ACTIVE unset, ~/.local/bin/nsh executable, then export NSH_ACTIVE=1 and exec.
  It cannot be installed, reverted or shared. Nothing catches a crash after the exec, and
  omarchy-reinstall-configs overwrites the file with no backup (INT-277 G7 measured both).
- INT-277 (depends_on 286, 287, 289, 290) already owns proving and hardening that path: G1 the
  handoff contract, G2 crash fallback, off switch and always-bash door, G6 ghostty parity, G7
  survival of Omarchy updates. It edits Christian's own ~/.bashrc by hand and ships nothing. This
  intent packages what 277 proves and adds no second mechanism.
- Omarchy's shared shell config is bash code: /usr/share/omarchy/default/bash holds aliases,
  completions, env-bootstrap, envs, fns, functions, init, inputrc, rc and shell. ~/.bashrc line 9
  sources rc before the exec. The exec keeps the environment (envs, PATH) and discards the aliases,
  functions and hooks, and nsh cannot source bash functions. So "source the same shared files"
  cannot be the design; what reaches nsh, by translation or not at all, is a G1 ruling. 277 G1
  records the per-hook gaps for Christian's machine.
- The premise that Omarchy ships omarchy-zsh and omarchy-fish packages is unconfirmed. Installed
  omarchy packages are omarchy, omarchy-settings, omarchy-keyring, omarchy-emacs, omarchy-nvim,
  omarchy-themes-intros and linux-omarchy with its headers. No zsh or fish package is installed and
  no script in /usr/share/omarchy/bin or /usr/share/omarchy/install mentions zsh or fish. Whether the
  repository offers one is not measured; G0 checks the sync database.
- omarchy-shell, omarchy-launch-shell, omarchy-restart-shell and omarchy-refresh-shell exist. They
  appear to concern Omarchy's desktop shell, not a command shell; G0 confirms, so the names are not
  confused.
- The Omarchy manual lists Alacritty as the default terminal, with Ghostty, Foot and Kitty
  supported. Christian's machine runs ghostty with no command line, so it starts the passwd shell.
- Audience is undecided: Christian's machine only, or other Omarchy users. His standing rule is that
  changes stay inside his own tools and never modify Omarchy's own code; a package in Omarchy's
  repository would be exactly that, so it is a ruling and not a default.

## The Solution
Wait for 277's G1, G2 and G7 rulings and evidence, then census, then rule, then build the smallest
installable form of exactly that contract.

- G0 measures what Omarchy ships, where a user-owned handoff can live so that reinstall-configs does
  not wipe it, and how many of Omarchy's aliases and functions are portable to nsh.
- G1 rules the form (install script with a drop-in, a PKGBUILD in this repo that Christian builds and
  installs himself, or a contribution), the home of the handoff, what carries to nsh, and the name:
  the omarchy- prefix is Omarchy's namespace.
- G2 to G5 build and prove it on a stand-in HOME first, never against the live ~/.bashrc, red
  before green.
- G6 is Christian running it by hand on his machine.

## Success Criteria
- [ ] G0 census, read-only; start only once 277 G2 has its rulings. Evidence is the output, kept in
      ~/.cache/zero:
      a. 277's G1, G2 and G7 rulings and evidence as written in that intent: crash mechanism (child
         or exec plus stamp), off-switch file, always-bash door, the d check.
      b. What Omarchy ships: pacman -Ss omarchy for any zsh, fish or shell package; pacman -Qo on
         /etc/skel/.bashrc; the files default/bash/rc sources and whether it reads any user-owned
         drop-in path; how omarchy-reinstall-configs and the installers treat ~/.bashrc and
         ~/.bash_profile; what the four omarchy-*-shell commands do.
      c. Portability: every entry in default/bash aliases, functions, fns, init and shell, classed as
         portable to nsh as written, needs translation, or bash-only, with nsh's alias and function
         syntax read from NOVASHELL.md and config.nsh.
      d. nsh's interactive entry points: nsh, -i, -c, --version, NSH_ACTIVE, the exit code and time of
         a start that fails, and what a preflight such as nsh -c : tells a handoff.
      e. The terminals: how Omarchy starts a shell (xdg-terminal-exec, TERMINAL), which terminal
         configs it writes, and what each of Alacritty, Ghostty, Foot and Kitty needs, or why it is
         deferred.
      f. Every premise in The Problem that a result above contradicts (the unconfirmed omarchy-zsh
         and omarchy-fish packages, the desktop-shell commands, line numbers) is corrected in this
         file before G1 starts.
- [ ] G1 rulings before any code, each approved by Christian: the form and the audience (install
      script with a drop-in, PKGBUILD built and installed by Christian, or an Omarchy contribution);
      the handoff's home and what survives reinstall-configs; the mechanism, which is 277 G2's
      ruling and not a new one; what carries to nsh and how (translation or recorded gap); the name;
      the one-command revert. Gates G2 to G6 are then re-read against these rulings and amended in this
      file before any code, if a ruling changes their shape (a contribution, for instance, is not
      tested on a stand-in HOME).
- [ ] G2 red, on a stand-in HOME made from Omarchy's default skeleton, never the live files: there is
      no one-command install today, and a simulated reinstall-configs (cp -af of the skeleton) removes
      the handoff with no trace. Both cases fail now.
- [ ] G3 build. G2 green. The smallest installable form per G1. Install is idempotent (twice gives the
      same bytes), backs up before touching anything, writes only its own files plus one marked
      block, and the revert restores the original byte for byte (sha256 compared). No root step is
      run by an agent; any such step is written out for Christian.
- [ ] G4 parity. What G1 says carries to nsh does carry: Omarchy's environment and PATH entries, and
      each translated alias or function, shown in a real nsh through an nsh-test Repl case or a
      recorded run. Every item that cannot carry is listed in the package README, not dropped
      silently.
- [ ] G5 non-locking, red first on the packaged form: a stand-in nsh that dies before its prompt lands
      in bash with one stderr line; the off file lands in bash; the always-bash door lands in bash;
      a simulated reinstall-configs leaves a working bash and a re-install restores the handoff.
      Run twice.
- [ ] G6 live, by Christian by hand. Before the install, 277 G2's off switch and always-bash door are
      shown working on the live machine, and a tty or second terminal stays open as the way out.
      Then: install; a new ghostty terminal lands in nsh; the passwd shell is still bash; revert
      returns a plain bash terminal; d 0 failed. Terminals other than ghostty are checked or
      deferred with a reason.
- [ ] G7 doors: nsh on PATH, nsh-test all passing, d 0 failed, core intent validate clean; the
      package README states install and revert and who owns it per INT-291's rule; commits carry
      Intent INT-295 and the plan Seal with its note under refs/notes/seals; main and
      refs/notes/seals pushed; core intent trace reads every seal intact; each gate carries
      evidence per INT-158.

## Non-goals
- No chsh and no login shell. That door is INT-277 G8 and, only if it opens, INT-288.
- No second handoff mechanism: crash fallback, off switch and the always-bash door are INT-277 G2's.
- No sudo or root step run by an agent; install steps that need root are written for Christian.
- No change to bash, sh, /etc/profile or any Omarchy file; the package owns only its own files.
- No contribution to Omarchy's repository unless Christian rules it at G1.
- No NovaShell change here; anything nsh needs becomes its own intent.
- Ghostty integration parity (ssh, OSC 7, 2 and 133) is INT-277 G6's, not this intent's.
- No desktop-shell (Quickshell) work. No new crates. AGENTS.md is not edited here.

## Dependencies
depends_on: 277. The edge is whole-intent: this intent starts after 277 closes, so 277's G8 answer
exists by then. The reasons are 277's G1, G2 and G7: started earlier, this would package a handoff
that is not yet hardened (the exec has nothing that catches a post-exec crash) and G1 here would
repeat 277 G2's rulings. G8's answer changes nothing in this intent: if it says no, INT-288 is
cancelled with that answer quoted and this intent continues unchanged. Sibling of INT-288, not a
follower of it.
Informs INT-291 and INT-293: the package README is a hand-owned file and the package is one more
tool record.
