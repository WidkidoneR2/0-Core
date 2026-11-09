---
id: 295
date: 2026-10-18
type: future
title: "Package the bash-to-nsh handoff as novashell-omarchy"
status: in-progress
tags: [nsh, omarchy, handoff, package, terminal, resilience]
depends_on: [277]
---

## Vision
Turning nsh on as the interactive shell on Omarchy is one command, turning it off is one command,
and bash stays the passwd shell. The handoff logic lives in a file the package installs under /usr.
~/.bashrc gets one marked line that sources it. An Omarchy package update does not remove that
line. omarchy-reinstall-configs does, and the doctor is what says so. The handoff is the one
INT-277 proves and hardens, not a second design. A broken nsh costs a prompt, never a terminal,
a tty or a session.

## The Problem
Laptop, 2026-11-09: Omarchy 4.5.0-3 and omarchy-settings 4.5.0-3. Source of that tree:
github.com/omacom/omarchy tag v4.5.0, commit ac2690cd (tag object 97e664bd). PKGBUILDs:
github.com/omacom/omarchy-pkgs, master 9ede717, which pins a948d8a at pkgrel 2. An earlier
release commit, 1187e06, pinned 76e11b4 at pkgrel 1. The installed package is pkgrel 3.
default/bash/init and omarchy-reinstall-configs are the same bytes at 76e11b4, a948d8a and
ac2690cd, and those bytes are what the laptop hashed (init e6a253e4, reinstall-configs
d9abe210).

- The thing to package is 277's child block, not the old four-line exec. ~/.bashrc is 61 lines.
  Line 9 sources Omarchy's rc. Christian reports the block at lines 31-59. The canonical file
  is zero/shell/novashell/handoff/bashrc-handoff.sh, 27 lines, so the doctor's byte compare is
  still the check that "intact" means. The block runs nsh as a child of interactive bash and
  stays in bash on status 101, 132, 134, 135 or 139 (277, lines 224-231). Any other status
  exits bash with nsh's status. ~/.config/nsh/off is absent, so the handoff is on. It cannot
  be installed or reverted as a package yet.
- omarchy-reinstall-configs still does `cp -af /etc/skel/. ~/` with no backup
  (bin/omarchy-reinstall-configs line 19 at ac2690cd). That wipes ~/.bashrc. The line added
  in 4.5.0, omarchy-install-video-decode, writes mpv and Chromium settings and does not
  restore bashrc.
- Omarchy's rc sources only files under $OMARCHY_PATH (default/bash/rc). It reads nothing
  under $HOME. nsh has alias and set, and no function syntax (zero/shell/novashell/src/config.rs).
  277 G1 carries environment and PATH. z and zi are native. Starship, the try and mise
  functions, fzf key bindings and bash completions do not carry. v4.5.0's init drops
  fzf's completion.bash and keeps the key bindings.
- omarchy-zsh 1.5.0-2 and omarchy-fish 1.5.0-1 are in the sync database and are not installed.
  Sources, tag v1.5.0: github.com/omacom-io/omarchy-zsh 4f79a296 and
  github.com/omacom-io/omarchy-fish 3658ad06. Recipes in omarchy-pkgs. pacman -S does not edit
  home and does not chsh. omarchy-setup-zsh and omarchy-setup-fish replace the whole ~/.bashrc,
  after a timestamped backup, with a template that execs zsh or fish. There is no crash
  fallback. Uninstall does not restore the backup.
- omarchy-shell, omarchy-launch-shell, omarchy-restart-shell and omarchy-refresh-shell are
  Quickshell, the desktop shell. The v4.0.4 to v4.5.0 diff of the first three does not mention
  bashrc or nsh.
- Foot is the default terminal (manual/15-terminal.md at v4.5.0; the system and user xdg lists
  are foot.desktop). Ghostty is installed. Alacritty and kitty are not. No command or shell
  key in the installed configs. TERMINAL=xdg-terminal-exec. SHELL=/usr/bin/bash.
- /etc/skel/.bashrc is owned by bash 5.3.20-1 and its bytes match Omarchy's default/bashrc.
  omarchy-settings post_upgrade copies /usr/share/omarchy/etc-overrides/dot.bashrc onto that
  path (omarchy-settings.install). Arch bash lists etc/skel/.bashrc in backup=, so a bash
  upgrade keeps the modified skel file. It does not touch ~/.bashrc.

## The Solution
Package 277's block as novashell-omarchy. Christian builds the PKGBUILD himself. It is not a
contribution to omarchy-pkgs for now.

- The handoff logic is a file the package installs under /usr. ~/.bashrc gets one marked line
  that sources it. The line is the only edit to his file.
- The mechanism stays 277 G2: nsh is a child, the off file is ~/.config/nsh/off, and the
  always-bash door is NSH_ACTIVE=1. Not an exec.
- What carries is the environment and PATH. No alias is translated.
- Install and revert are a command he runs. pacman post_install does not edit ~/.bashrc.
  Revert deletes the marked line. The sha256 of ~/.bashrc is compared to the value taken
  before install.
- Each of G2 to G5 and G7 is one small PR, red first, on a stand-in HOME. G6 is Christian
  on his laptop. No gate edits the live ~/.bashrc.

## Rulings (G1, 2026-11-09)
Christian ruled each of these.

1. Form: a PKGBUILD he builds himself. Not a contribution to omarchy-pkgs for now.
2. Name: novashell-omarchy. The omarchy-* prefix is left to Omarchy. The filename of this
   intent stays; core intent validate checks the id prefix, not the title.
3. Home: the handoff logic lives in a file outside /etc/skel, installed by the package under
   /usr. ~/.bashrc gets one marked line that sources it. Omarchy's rc sources nothing under
   $HOME, so the line is what makes the file run.
4. Mechanism: 277's child, with the crash fallback. Status 101, 132, 134, 135 or 139 stays
   in bash. Not exec.
5. Carry: environment and PATH only. No alias translation. c, d, h and ls would hide nsh
   builtins, and d is the doctor.
6. Revert: delete the marked line, and compare the sha256 of ~/.bashrc with the value saved
   before install.
7. Install: a command Christian runs. Never a pacman post_install. No chsh. No terminal
   config edit.
8. G6 terminals: foot and ghostty. No terminal config change.
9. Known risk: omarchy-reinstall-configs wipes the marked line. zero-doctor's shell_handoff
   must notice, and the setup command puts the line back. The doctor's check today compares
   ~/.bashrc with the whole 27-line block. It has to compare the marked line instead.

## Success Criteria
- [x] G0 census, read-only; start only once 277 G2 has its rulings. Evidence is the output, kept in
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
<!-- evidence: 2026-11-09. Census of v4.0.4 (c668141e) plus the addendum against the laptop script ~/.cache/zero/int-295-g0-laptop.txt. a. 277 G2 is the child, off file ~/.config/nsh/off, always-bash door NSH_ACTIVE=1; shell_handoff warns and never fails. b. v4.5.0 ac2690cd; rc sources only $OMARCHY_PATH; no user drop-in; reinstall-configs is cp -af with no backup (d9abe210); the four omarchy-*-shell commands are Quickshell. Laptop: pacman -Qo says bash 5.3.20-1 owns /etc/skel/.bashrc and the bytes match Omarchy default/bashrc; omarchy-settings owns etc-overrides/dot.bashrc; sync db has omarchy-zsh 1.5.0-2 and omarchy-fish 1.5.0-1, not installed. c. Nothing in default/bash is portable to nsh as written. 24 aliases can be rewritten. Functions, init and shell are bash-only. c, d, h and ls would hide builtins. d. VM debug nsh 5.3.11, not Omarchy: --version exits 0 before the runtime; there is no -i; nsh -c : exits 1, command not found, so it is a false failure; nsh does not read NSH_ACTIVE. e. TERMINAL=xdg-terminal-exec; Foot is the default; ghostty installed; alacritty and kitty absent; no command or shell key. f. The Problem section above is that correction. omarchy-pkgs master 9ede717. -->
- [x] G1 rulings before any code, each approved by Christian: the form and the audience (install
      script with a drop-in, PKGBUILD built and installed by Christian, or an Omarchy contribution);
      the handoff's home and what survives reinstall-configs; the mechanism, which is 277 G2's
      ruling and not a new one; what carries to nsh and how (translation or recorded gap); the name;
      the one-command revert. Gates G2 to G6 are then re-read against these rulings and amended in this
      file before any code, if a ruling changes their shape (a contribution, for instance, is not
      tested on a stand-in HOME).
<!-- evidence: 2026-11-09, Christian. Recorded in ## Rulings. Form: PKGBUILD he builds, not an omarchy-pkgs contribution for now. Name: novashell-omarchy. Home: one marked source line in ~/.bashrc, logic in a package file under /usr. Mechanism: 277's child. Carry: environment and PATH only. Revert: delete the line and compare sha256 to the pre-install value. Install: a command he runs, never post_install; no chsh; no terminal config. G6: foot and ghostty. G2 to G7 below are amended to that shape. No code in this commit. -->
- [ ] G2 red, one PR, on a stand-in HOME made from Omarchy's default/bashrc, never the live files.
      Two failures, both shown failing: there is no novashell-omarchy setup command, and a simulated
      reinstall-configs (`cp -af` of the skeleton onto the stand-in) deletes a planted marked source
      line and leaves no trace. Red first. The fix is not in this gate.
- [ ] G3 build, one PR. G2's missing-command case goes green. The PKGBUILD is novashell-omarchy,
      built and installed by Christian. The setup command is not a pacman post_install. On a
      stand-in HOME it is idempotent (twice gives the same bytes), saves the sha256 of ~/.bashrc
      before it writes, writes only the package file under a stand-in /usr plus one marked source
      line, and does not chsh or edit a terminal config. Revert deletes that line and the sha256
      of ~/.bashrc matches the saved value. No root step is run by an agent. Any pacman command
      is written out for Christian. The path under /usr is recorded in this file in that PR.
- [ ] G4 parity, one PR, red first, stand-in HOME. What G1 carries does carry: the environment and
      the PATH entries 277 G1 names (OMARCHY_PATH, mise shims, ~/.local/bin, ~/.cargo/bin), shown
      in a real nsh through an nsh-test Repl case or a recorded run. No alias is translated. The
      package README lists what does not carry, including c, d, h and ls, and the bash functions,
      starship, mise, and the fzf key bindings. A missing entry is a failure, not a silence.
- [ ] G5 non-locking, one PR, red first, stand-in HOME, on the packaged form. Red: shell_handoff
      still compares ~/.bashrc with the whole 27-line block, so a file that has only the marked
      line is reported as no handoff. Green: the check compares the marked line, passes when the
      line is present and the file it sources is the packaged handoff, and warns when the line is
      gone. A stand-in nsh that exits 101, 132, 134, 135 or 139 lands in bash with one stderr
      line. Any other status exits bash with that status. The off file lands in bash. NSH_ACTIVE=1
      lands in bash. A simulated reinstall-configs removes the marked line and leaves a working
      bash. Running the setup command puts the line back. Run twice.
- [ ] G6 live, one sitting, by Christian by hand on his laptop. Not an agent. Before the install,
      277 G2's off switch and always-bash door are shown working, and a tty or a second terminal
      stays open. Then he runs the setup command. A new foot terminal lands in nsh. A new ghostty
      terminal lands in nsh. No terminal config is edited. The passwd shell is still bash. Revert
      deletes the marked line and the sha256 of ~/.bashrc matches the pre-install value. d 0 failed.
      Alacritty and kitty are not installed, so they stay out.
- [ ] G7 doors, one PR: cargo test -p novashell, nsh-test all passing, d 0 failed, core intent
      validate clean. The package README states install, revert, and who owns the file, per
      INT-291. Commits carry Intent: INT-295 and the plan Seal, with the note under
      refs/notes/seals. main and refs/notes/seals are pushed. core intent trace reads every seal
      intact. Each gate carries evidence per INT-158.

## Non-goals
- No chsh and no login shell. That door is INT-277 G8 and, only if it opens, INT-288.
- No second handoff mechanism: crash fallback, off switch and the always-bash door are INT-277 G2's.
  The zsh and fish exec is not adopted.
- No sudo or root step run by an agent; install steps that need root are written for Christian.
- No change to bash, sh, /etc/profile or any Omarchy file. The package owns its file under /usr.
  The one marked line is the only write in ~/.bashrc, and only when Christian runs the command.
- No contribution to omarchy-pkgs for now. Ruled at G1, 2026-11-09.
- No NovaShell change here. The shell_handoff compare in G5 is zero-doctor. Anything nsh
  itself needs becomes its own intent.
- Ghostty integration parity (ssh, OSC 7, 2 and 133) is INT-277 G6's, not this intent's.
- No desktop-shell (Quickshell) work. No new crates. AGENTS.md is not edited here.
- No alias translation.

## Dependencies
depends_on: 277. The edge is whole-intent: this intent starts after 277 closes, so 277's G8 answer
exists by then. The reasons are 277's G1, G2 and G7: started earlier, this would package a handoff
that is not yet hardened (the exec has nothing that catches a post-exec crash) and G1 here would
repeat 277 G2's rulings. G8's answer changes nothing in this intent: if it says no, INT-288 is
cancelled with that answer quoted and this intent continues unchanged. Sibling of INT-288, not a
follower of it.
Informs INT-291 and INT-293: the package README is a hand-owned file and the package is one more
tool record.
