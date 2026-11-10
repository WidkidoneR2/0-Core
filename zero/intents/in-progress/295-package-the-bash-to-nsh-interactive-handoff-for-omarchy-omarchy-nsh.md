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

## Paths (G3, 2026-11-09)
The package directory is zero/shell/novashell/handoff/novashell-omarchy/. The setup
command in the tree is novashell-omarchy/novashell-omarchy inside that directory.
G2's test looks there. It first looked for a file named novashell-omarchy directly
in handoff/, and the lookup constant moved one level down with the directory.

The PKGBUILD installs two files:
- /usr/share/novashell-omarchy/bashrc-handoff.sh, the 277 block, byte for byte
- /usr/bin/novashell-omarchy

The marked line is one line. A removed package leaves it in ~/.bashrc and bash
continues, with nothing from that line on stderr:

`[ -r /usr/share/novashell-omarchy/bashrc-handoff.sh ] && source /usr/share/novashell-omarchy/bashrc-handoff.sh # novashell-omarchy`

makepkg looks up a local source by its basename in the package directory.
`../bashrc-handoff.sh` in source=() is sought as `./bashrc-handoff.sh` and missed.
bashrc-handoff.sh in the package directory is a symlink to the canonical block
one directory up. source= lists that name, and sha256sums is the checksum of
the file the link names.

install saves the sha256 of ~/.bashrc, taken before the write, as one hex line at
~/.local/state/novashell-omarchy/bashrc.sha256. A second install leaves both files
untouched. revert removes the marked line and compares that sha256. When the sha256
does not match, it says so and leaves every other byte of ~/.bashrc in place.

install refuses, and writes nothing, when ~/.bashrc already contains the inline 277
block (a contiguous copy of bashrc-handoff.sh). The message names the opening line
and two manual steps: delete that block, then run novashell-omarchy install.
Moving an existing inline block over is Christian's call at G6.

Handoff version 277.1 is the first line of VERSION beside the PKGBUILD. pkgver copies
it. 277 is that block (sha256 178960349fa03a05). .1 is the first package of it.

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
- [x] G2 red, one PR, on a stand-in HOME made from Omarchy's default/bashrc, never the live files.
      Two failures, both shown failing: there is no novashell-omarchy setup command, and a simulated
      reinstall-configs (`cp -af` of the skeleton onto the stand-in) deletes a planted marked source
      line and leaves no trace. Red first. The fix is not in this gate.
<!-- evidence: 2026-11-09. Merged as 4ac3356c. Christian ran bash zero/shell/novashell/handoff/test-g2-red.sh: exit 101, both failures (g2_there_is_no_novashell_omarchy_setup_command and g2_reinstall_deletes_the_marked_line_and_nothing_notices). Doors green. The run itself: cargo test -p zero-doctor --lib g2_ -- --ignored --nocapture --test-threads=1. Exit 101. 0 passed; 2 failed. g2_there_is_no_novashell_omarchy_setup_command: no novashell-omarchy setup command; looked on PATH and at zero/shell/novashell/handoff/novashell-omarchy; novashell-omarchy setup: No such file or directory. g2_reinstall_deletes_the_marked_line_and_nothing_notices: cp -af deleted the planted marked source line and nothing noticed; marked line absent: true; cp stderr: ""; trace files: []; shell_handoff: Warn the handoff is not in .bashrc; terminals open bash; expected a warning that names the marked source line (novashell-omarchy). Stand-in HOME under /tmp only. Fixture zero/shell/novashell/handoff/omarchy-v4.5.0-default.bashrc, payload sha256 d78bd5405919c9c0d9aa5a3db47696286aa9c9d0a1c759785719054605da9a4b, Omarchy v4.5.0 default/bashrc at https://github.com/omacom/omarchy/blob/ac2690cd/default/bashrc. -->
- [x] G3 build, one PR. G2's missing-command case goes green. The PKGBUILD is novashell-omarchy,
      built and installed by Christian. The setup command is not a pacman post_install. On a
      stand-in HOME it is idempotent (twice gives the same bytes), saves the sha256 of ~/.bashrc
      before it writes, writes only the package file under a stand-in /usr plus one marked source
      line, and does not chsh or edit a terminal config. Revert deletes that line and the sha256
      of ~/.bashrc matches the saved value. No root step is run by an agent. Any pacman command
      is written out for Christian. The path under /usr is recorded in this file in that PR.
<!-- evidence: 2026-11-09, merged as 6b8722bc. On a /tmp stand-in HOME, status reported line/handoff/off/nsh absent. install saved sha256 9e07c633… and added one marked line. A second install wrote nothing. revert removed the line and the sha256 matched (9e07c633…). install on a copy of the real ~/.bashrc refused (inline block), wrote nothing, exit 1. makepkg --noconfirm --nodeps with temp PKGDEST and BUILDDIR built novashell-omarchy-277.1-1-any.pkg.tar.zst; both sha256sums Passed; not installed. Doors: cargo test -p novashell 258 passed; nsh-test 302/302; core intent validate 342 valid; d 96% 0 failed (1 known rustdoc warning). cargo test -p zero-doctor --lib was 79 passed, 1 ignored (g2_reinstall, red until G5), only when detached from the terminal (setsid -w … < /dev/null). From an interactive terminal the job was stopped by SIGTTIN/SIGTTOU (Stopped in bash, suspended in nsh). Christian has not installed the package; that is G6. The stop is fixed here. Red: bash_rc_cannot_take_the_controlling_terminal, exit 101, the child session equalled the test session. Green after the spawn calls setsid with stdin /dev/null. process_group(0) was measured on a pty to stop the child (state T), so it is not the fix. Verified under script -qec on a real pty (/dev/pts/0): cargo test -p zero-doctor --lib finished and was not stopped. On this VM, 78 passed, 2 failed (the_registry_parses_and_declares_tools and the_quick_scan_is_the_critical_tier_and_nothing_else, both looking for ~/0-core, which is absent), 1 ignored. -->
- [x] G4 parity, one PR, red first, stand-in HOME. What G1 carries does carry: the environment and
      the PATH entries 277 G1 names (OMARCHY_PATH, mise shims, ~/.local/bin, ~/.cargo/bin), shown
      in a real nsh through an nsh-test Repl case or a recorded run. No alias is translated. The
      package README lists what does not carry, including c, d, h and ls, and the bash functions,
      starship, mise, and the fzf key bindings. A missing entry is a failure, not a silence.
<!-- evidence: 2026-11-09. Merged as 7ed2fc90. Run straight from an nsh terminal, with no setsid, and nothing was suspended (the terminal fix works). cargo test -p zero-doctor --lib: 81 passed, 0 failed, 1 ignored (g2_reinstall, red until G5). cargo test -p novashell: 258 passed. nsh-test 303/303, including repl_295_packaged_line_carries_env_and_path. core intent validate: 342 valid. d 96%, 0 failed (1 known rustdoc warning). Red: g4_readme_names_every_census_entry_that_does_not_carry exited 101 before the README existed and named all 13 census entries (c, d, h, ls, hide nsh builtins, d is the doctor, bash functions, try, mise functions, starship, mise activation, fzf key bindings, bash completions). Green: the same test passed once zero/shell/novashell/handoff/novashell-omarchy/README.md listed them. -->
- [x] G5 non-locking, one PR, red first, stand-in HOME, on the packaged form. Red: shell_handoff
      still compares ~/.bashrc with the whole 27-line block, so a file that has only the marked
      line is reported as no handoff. Green: the check compares the marked line, passes when the
      line is present and the file it sources is the packaged handoff, and warns when the line is
      gone. A stand-in nsh that exits 101, 132, 134, 135 or 139 lands in bash with one stderr
      line. Any other status exits bash with that status. The off file lands in bash. NSH_ACTIVE=1
      lands in bash. A simulated reinstall-configs removes the marked line and leaves a working
      bash. Running the setup command puts the line back. Run twice.
<!-- evidence: 2026-11-10. Merged as 9531c1ee via PR #11. On Christian's laptop: zero-doctor 95 passed, 0 ignored; novashell 258; nsh-test 303/303; 342 intents valid. Shipped with ship core (core 4.4.7, zero-doctor is a library inside core). Live d after the G6 install shows Shell Handoff: "marked line present; packaged handoff matches the canonical block; nsh is executable; nothing below it". Red: cargo test -p zero-doctor --lib g5_marked_line_alone_is_reported_as_no_handoff exited 101. 0 passed, 1 failed. The rc was the Omarchy v4.5.0 fixture plus only the marked line. shell_handoff returned Warn, "the handoff is not in .bashrc; terminals open bash". It still compared the whole 27-line block. Green, twice, both exit 0: cargo test -p zero-doctor --lib g5_ -- --test-threads=1. Run 1: 13 passed, 0 failed, 0.12s. Run 2: 13 passed, 0 failed, 0.12s. g2_reinstall_deletes_the_marked_line_and_nothing_notices is no longer ignored; both runs passed. The marked line passes when the sourced file is byte-identical to bashrc-handoff.sh (sha256 178960349fa03a05). ZERO_DOCTOR_HANDOFF and the path argument of shell_handoff_at name a stand-in, so /usr/share is not the only path. A missing or different packaged file warns and does not fail. The line gone warns, names novashell-omarchy, and says to run novashell-omarchy install. An intact inline block with no package still passes. Both together warn of a double handoff. Stand-in nsh on 101, 132, 134, 135 and 139 stays in bash with one stderr line. Statuses 0, 1, 2, 100, 130, 131, 133, 136, 137, 138, 140, 200 and 255 exit bash with that status. The off file and NSH_ACTIVE=1 land in bash. cp -af of the v4.5.0 fixture removes the marked line and bash still runs; novashell-omarchy install puts the line back and shell_handoff passes. Stand-in HOMEs under /tmp only. -->
- [x] G6 live, one sitting, by Christian by hand on his laptop. Not an agent. Before the install,
      277 G2's off switch and always-bash door are shown working, and a tty or a second terminal
      stays open. Then he runs the setup command. A new foot terminal lands in nsh. A new ghostty
      terminal lands in nsh. No terminal config is edited. The passwd shell is still bash. Revert
      deletes the marked line and the sha256 of ~/.bashrc matches the pre-install value. d 0 failed.
      Alacritty and kitty are not installed, so they stay out.
<!-- evidence: 2026-11-09, Christian, by hand on his laptop, one sitting, about 9:23-9:35 PM CST. Safety net first: ~/.bashrc backed up to ~/.bashrc.pre-g6 (sha256 b67db5a6edc47bff…, both identical); a spare terminal stayed open; the 277 G2 off switch was proven (touch ~/.config/nsh/off, and a new foot printed "nsh handoff is off (rm ~/.config/nsh/off to turn it on); staying in bash."), then the switch was removed; the always-bash door was proven (NSH_ACTIVE=1 bash -i stayed in bash). makepkg -f built novashell-omarchy-277.1-1-any.pkg.tar.zst; both sources passed sha256 validation. Christian ran sudo pacman -U himself. pacman -Ql lists exactly /usr/bin/novashell-omarchy and /usr/share/novashell-omarchy/bashrc-handoff.sh. novashell-omarchy install refused while the inline 27-line block was present ("nothing was written", sha256 unchanged b67db5a6…) and printed the manual steps. Christian deleted the inline block by hand (lines 35-61; diff vs backup = 35,61d34, 28 lines, nothing else changed); new sha256 03ab33060f93910a…. novashell-omarchy install saved 03ab33060f93910a4a6f7d1e62524486fcf3a681ed153407b05203410fcce4b5 to ~/.local/state/novashell-omarchy/bashrc.sha256 and appended the one marked line as the last line; status: line present, handoff present, off absent, nsh executable. A new foot terminal and a new ghostty terminal each landed in nsh (full Project 0 banner). The passwd shell is still /usr/bin/bash. find ~/.config/foot ~/.config/ghostty -newer ~/.bashrc.pre-g6 = 0 (no terminal config edited). d: 0 failed. Shell Handoff passes on the marked line (warnings only: uncommitted build output, since cleaned, and the 1 known rustdoc warning). novashell-omarchy revert: "removed the marked line", "sha256 matches the value saved before install" (03ab3306…, both); a new foot terminal then opened in plain bash. Christian then reinstalled the line; nsh is his daily shell through the package. Alacritty and kitty stay out of scope as written. -->
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
