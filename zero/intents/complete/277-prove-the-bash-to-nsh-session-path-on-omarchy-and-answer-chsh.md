---
id: 277
date: 2026-10-05
type: future
title: "Prove the bash-to-nsh session path on Omarchy and answer chsh"
status: complete
tags: [nsh, omarchy, login, session, resilience, bashrc, handoff, chsh, sddm, uwsm, ghostty, mise, recovery]
depends_on: [286, 287, 289, 290]
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
- [x] G0 Recon the login and terminal chain, read-only. Evidence is the
      ## Recon section above (five rounds, 2026-10-05). Tick after cistart.
<!-- evidence: demonstrated: five recon rounds on omarchy 4.0.4-1 (2026-10-05), see ## Recon. Re-read 2026-10-26 on omarchy 4.0.6-1, all matching: passwd shell /usr/bin/bash; /etc/shells has no nsh; .bash_profile byte-identical to /etc/skel (sha256 e149407c...); ~/.bashrc lines 1-12 and 25-50 unchanged, handoff at 43-46; omarchy-reinstall-configs line 19 is cp -af /etc/skel/. ~/ ; wayland-session lines 11-47 unchanged (SHELL saved at 11, exported at 47); uwsm env.d/10-omarchy sources env-bootstrap (TERMINAL and EDITOR now come from default/uwsm/default when present, same values); user manager SHELL=/usr/bin/bash; 0 migrations mention bashrc; ghostty config has no command line. Not re-read: env-bootstrap and rc init contents, installer append lines, the quattro upgrade; G1 and G7 measure those. -->
- [x] G1 Handoff contract written here before any edit to ~/.bashrc, each
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
<!-- evidence 2026-10-27 (G1): canonical block zero/shell/novashell/handoff/bashrc-handoff.sh, sha256 178960349fa03a05, commit c158c461. nsh-test 298/298.
PROOF: env -i HOME="$HOME" TERM="$TERM" bash -l -c 'echo reached' printed: reached (-l is the short form of the login flag the gate names; measured with the long form too).
R1 login bash: the proof line above; the block is gated on an interactive shell, so a non-interactive login bash never reaches it.
R2 bash -c: printed reached-c.
R3 a terminal lands in nsh: harness rows repl_277_handoff_typed_exit_is_the_session_status and repl_277_handoff_nsh_gets_the_mise_shims_and_not_the_install_dirs start interactive bash with the block as its rc file, and the session that answers is real nsh. The real-window check on the adopted ~/.bashrc is G6 (C3), because it cannot exist before the edit.
R4 bash typed inside nsh stays bash: repl_277_handoff_always_bash_door_lands_in_bash. NSH_ACTIVE=1 in the environment makes the block do nothing, and the block exports it before it starts nsh, so every bash that nsh starts inherits it.
R5 a terminal launched from inside nsh lands in bash: the same mechanism, inherited NSH_ACTIVE. Ruled by Christian: accept and record. It is wanted.
R6 PATH: from the systemd user PATH (10 entries), a login bash ends with 10, and nsh -c started with that PATH reports the same 10 in the same order, nothing added or lost. The OMARCHY_PATH entry /usr/share/omarchy/bin, the mise shims, ~/.local/bin and ~/.cargo/bin are all in the PATH nsh receives (measured in the harness PATH answer). Side note: bash puts the mise shims dir on PATH twice. Harmless to the block.
R7 exports and hooks: mise, the stale install dirs ahead of the shims are removed for nsh only when the shims dir is on PATH, bash keeps its own PATH after a crash, and without a shims dir the install dirs stay (rows repl_277_handoff_nsh_gets_the_mise_shims_and_not_the_install_dirs, repl_277_handoff_bash_keeps_its_own_path_after_a_crash, repl_277_handoff_install_dirs_stay_when_there_is_no_shims_dir). zoxide: z and zi are native in nsh. starship: replaced by nsh's own prompt, not carried. GAPS recorded, not verified inside nsh: the try and mise shell functions, fzf key bindings, bash completions.
COMMENT CORRECTION proposed: the INT-190/niri story in the old handoff comment is wrong for the current chain. It sits inside lines 24-46 of ~/.bashrc, the range adoption replaces, so Christian's adoption corrects it.
ADOPTION RULE: the live ~/.bashrc still has the old exec handoff at lines 24-46. Replace those lines with the canonical block. Appending would never reach the new block, because the old exec runs first. -->
- [x] G2 A broken or unwanted nsh never costs a prompt. Rulings before
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
<!-- evidence: commits d0e31e8a (handoff harness and cases), 7684d9d9, e62e9341, 11999e42, 2a0cdb72, 93c06c46, ad5e0690; demonstrated 2026-10-26 and 2026-10-27, never against the live ~/.bashrc or the shipped handoff. Red first: the exec handoff took the shell down on a stand-in that exits 101, twice. Green on the new block (nsh as a child of bash): statuses 101, 132, 134, 135, 139 stay in bash with one stderr line naming why, every other status including 0, 74, 129 and 141 exits bash with that status (repl_277_handoff_crash_codes_fall_back_and_other_codes_exit); the real nsh through the copy lands in nsh; the off file lands in bash with the notice (repl_277_handoff_off_switch_stays_in_bash, real nsh); the door lands in bash with the stand-in still broken and with the real nsh (repl_277_handoff_always_bash_door_lands_in_bash). Mutation checks: replacing exit "$nsh_status" in the block copy turned three handoff cases red; removing the NSH_ACTIVE guard clause turned the door case red, 274 of 275, restored 275 of 275, block 747 bytes sha256 ed135d0accfe. Run twice: the suite passed 275 of 275 on the debug build in two separate runs on 2026-10-27. Real nsh, scratch HOME, core limit 0, each signal twice on 2026-10-27: SIGABRT gives the fallback line (status 134), SIGILL gives it (status 132), bash answers, a typed exit 5 ends bash with 5, four of four. A real nsh panic exits 101 (F-0096, repl_f0096_a_repl_thread_panic_exits_101). SIGSEGV 139 and SIGBUS 135: the stand-in status table plus the real 134 and 132, ruled by Christian 2026-10-27. Always-bash door on the real desktop 2026-10-27: Hyprland bind SUPER + CTRL + SHIFT + B in ~/.config/hypr/bindings.lua (Christian own file, backup ~/.cache/zero/bindings.lua.pre-277) opens foot; inside it NSH_ACTIVE is 1 and the chain is python3, bash, foot, no nsh. Off switch in a real window: not shown, because the live ~/.bashrc still holds the old exec handoff that never reads ~/.config/nsh/off; it is the first check after Christian adopts the block (START HERE step 5). -->
- [x] G3 Startup resilience, isolated through the state-dir and state-db
      overrides (names confirmed at G3 recon; never the live state.db).
      nsh reaches a prompt, or exits non-zero fast enough for G2 to catch
      it, under: corrupt state.db; read-only state dir; config.nsh syntax
      error; empty HOME; launch cwd deleted; PATH=/usr/bin only; TERM=dumb;
      core absent from PATH. Proof: one nsh-test Repl case per row, green
      on the shipped binary; a failing row is recorded as a finding before
      it is fixed.
<!-- evidence 2026-10-27: G3 shipped binary (~/.local/bin/nsh, NSH_BIN set to it), nsh-test 298/298 after ship. One Repl row per condition, all green: repl_277_g3_corrupt_state_db_still_starts, repl_277_g3_read_only_state_dir_still_starts, repl_277_g3_config_syntax_error_still_starts, repl_277_g3_empty_home_still_starts, repl_277_g3_launch_directory_deleted_still_starts, repl_277_g3_path_usr_bin_only_still_starts, repl_277_g3_term_dumb_still_starts, repl_277_g3_core_absent_from_path_still_starts. -->
- [x] G4 exec resolves PATH. Red 2026-09-05: `exec nsh` -> No such file.
      Also: exec of a missing command in interactive nsh reports and keeps
      the shell. Proof: red capture, fix, nsh-test case, green.
<!-- evidence 2026-10-27: G4 shipped binary, nsh-test 298/298 after ship. Rows green: repl_277_g4_exec_nsh_by_bare_name_keeps_the_session, repl_277_g4_exec_finds_a_command_on_path, repl_277_g4_exec_skips_a_file_on_path_that_is_not_executable, repl_277_g4_exec_of_a_missing_command_keeps_the_shell. The exec reload through the handoff is covered too: repl_277_handoff_exec_reload_keeps_the_session. -->
- [x] G5 Terminal and job-control matrix in the pty harness: Ctrl-C ends
      the foreground child, not the shell; Ctrl-Z stops it, jobs shows it
      stopped, fg resumes it with the terminal, bg continues it;
      `yes | head -3` returns and the shell survives; closing the terminal
      with a background job does what NOVASHELL.md says. Proof: one Repl
      case per row.
<!-- evidence 2026-10-27: G5 shipped binary, nsh-test 298/298 after ship. Rows green: repl_277_g5_ctrl_c_ends_the_foreground_child_not_the_shell, repl_277_g5_ctrl_z_stops_jobs_shows_it_and_fg_resumes_it_with_the_terminal, repl_277_g5_bg_continues_a_stopped_job_and_the_shell_answers, repl_277_g5_yes_piped_to_head_returns_and_the_shell_survives. Closing the terminal with a background job: repl_277_handoff_window_close_hangs_up_bash_and_leaves_no_job (the hangup reaches bash, the job goes with it). The matching NOVASHELL.md line is a G9 proposal, not made. -->
- [x] G6 Terminal integration parity across the installed terminals. Rescoped
      2026-10-26 (Christian ruled): foot (in use), ghostty (Omarchy default, the
      2026-10-05 recon), kitty and Rio, all stated installed and recently updated.
      The handoff gives the prompt to nsh, so what a terminal integration does in
      bash does not reach it. Recon first: confirm each is installed and which
      version (which and the version flag), then measure what each expects from
      the shell; do not assume. Per terminal and item, nsh provides it or the gap
      is recorded as a finding: ssh (TERM fallback and SetEnv/SendEnv) first, then
      OSC 7 cwd for new windows and splits, OSC 2 title, OSC 133 prompt marks.
      Proof: each item observed in a real window of that terminal, foot first,
      then ghostty, kitty, Rio. Each terminal also passes C3 and C9 from its own
      window.
<!-- evidence 2026-10-27: real windows, each output labelled by its process chain. FOOT 1.28.0: TERM xterm-256color (safe for ssh); OSC 2 title christian@christian:/tmp; OSC 7 end to end, Ctrl+Shift+N from nsh in /tmp gave the new shell /tmp (a hand-made OSC 7 was honoured too); OSC 133 prompt jump Ctrl+Shift+Z worked; C3 SUPER+Return window chain nsh <- bash <- foot; C9 exec of the deployed nsh keeps nsh <- bash <- foot; typed exit closes the window. GHOSTTY 1.3.1: OSC 2 title christian@christian:/tmp; OSC 7 followed, the new window shell got /tmp; C3 chain nsh <- bash <- ghostty <- systemd; C9 after exec nsh <- bash <- ghostty; TERM xterm-ghostty is the ssh gap in F-0103. RIO 0.5.28: OSC 2 title christian@christian:/tmp; C3 chain nsh <- bash <- rio; C9 after exec nsh <- bash <- rio; TERM rio is the ssh gap in F-0103; OSC 7 ignored by Rio for new windows, F-0107. OSC 133 terminal side effect not observed in ghostty or Rio, F-0108 (confirm-close-surface=false in the ghostty config hides the only visible effect). kitty is not installed here (no binary, no terminfo), so it is outside the installed terminals. Every terminal: after the terminal hands the shell a directory, nsh moves to the core root, F-0106. ssh: no ~/.ssh/config, so no SetEnv or SendEnv to carry. -->
- [x] G7 The handoff survives Omarchy updates and installers. Measured on
      4.0.4: no migration touches ~/.bashrc; the quattro upgrade leaves this
      .bashrc unchanged; omarchy-reinstall-configs overwrites it with no
      backup; two installers append below the exec. Rows: a d check reports
      the handoff present and its target executable (built on Christian's
      ruling; watched failing against a copy with the block removed, then
      passing); appended lines below the exec are either reported by that
      check or the handoff is moved so it runs after the whole rc (ruling).
<!-- evidence 2026-10-27: Shell Handoff check (probe shell_handoff, commits 15355b06 and 282d7dd8) read against three copies of the adopted ~/.bashrc through ZERO_DOCTOR_BASHRC, real file untouched. Adopted copy: PASS, handoff block intact in .bashrc; nsh is executable; nothing below it. Block removed: WARN, the handoff is not in .bashrc; terminals open bash. A line appended below: WARN, 1 line(s) below the handoff run only when nsh is off or has crashed; nsh sessions never see them. The check compares the rc with the canonical block zero/shell/novashell/handoff/bashrc-handoff.sh (sha256 178960349fa03a05). First alternative of the gate taken: appended lines are reported, so moving the handoff is not needed. Live d on the adopted rc: 26 passed, 0 failed, health 89%. -->
- [x] G8 The chsh impact map, measured without chsh and without sudo. Run
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
<!-- evidence: demonstrated 2026-10-27 with no chsh, no sudo and no Omarchy file touched. SDDM's own script in the gate's literal form, once per shell: env -i HOME USER LOGNAME PATH=/usr/local/sbin:/usr/local/bin:/usr/bin SHELL=<shell> /bin/sh /usr/share/sddm/scripts/wayland-session env. Both runs hand the session the same nine variables once _, SHLVL, PWD and OLDPWD are set aside: DEBUGINFOD_URLS=https://debuginfod.archlinux.org, HOME=/home/christian, LANG=en_US.UTF-8, LOGNAME=christian, OMARCHY_PATH=/usr/share/omarchy, PATH=/usr/local/sbin:/usr/local/bin:/usr/bin:/home/christian/.local/share/mise/shims:/home/christian/.local/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl, USER=christian, VSSCRIPT_PATH=/usr/lib/python3.14/site-packages/vapoursynth/libvsscript.so, and SHELL. The diff is one line: SHELL=/usr/bin/bash against SHELL=/home/christian/.local/bin/nsh. Why: with a bash passwd shell the script re-runs itself as a bash login shell (lines 14 to 18); with nsh it takes the unknown-shell branch (lines 40 to 43), sources /etc/profile and ~/.profile, nsh never runs, and /etc/profile.d/omarchy.sh supplies the same Omarchy environment. THE LOGIN BLOCKER: /etc/shells lists /bin/sh, /bin/bash, /bin/rbash, /usr/bin/sh, /usr/bin/bash, /usr/bin/rbash, /usr/bin/systemd-home-fallback-shell and /usr/bin/git-shell. The libc call getusershell, which pam_shells uses, returns /usr/bin/bash and does not return /home/christian/.local/bin/nsh. /etc/pam.d/sddm-autologin line 3 and /etc/pam.d/system-login line 3 are both `auth required pam_shells.so`, so a passwd shell outside /etc/shells is refused at SDDM autologin and at every tty login. chsh itself needs a listed shell and /etc/shells is root-owned, so the route needs sudo, a non-goal. Disclosed limit: not shown by a real login, which would take chsh or a test account; read from the PAM files plus the libc call. INT-288 faces the same blocker, since whatever sits in the passwd field must be listed. -c CALLERS, re-measured 2026-10-27 (shipped nsh 5.3.7, the debug build, and bash as control, scratch directory): nsh -c "echo test > 0.5" creates the file, exactly as bash does, so the 2026-09-05 digit-guard divergence still stands; the working directory is kept, not moved to ~/0-core. nsh -c with 3 or more commands writes the [i/N] progress lines to stdout (2 commands are clean; argv0 -nsh is the same): F-0105 at novashell/src/main.rs:1434, so a $SHELL -c caller running a chain would get corrupted output. LOGIN START: argv0 -nsh with -c, a bare environment and a scratch HOME holding .profile, .bash_profile and .bashrc markers. -bash read .bash_profile only and got OMARCHY_PATH=/usr/share/omarchy, the mise shims and ~/.local/bin. -nsh read none of the three and got PATH=/usr/local/sbin:/usr/local/bin:/usr/bin with OMARCHY_PATH empty, so no ~/.local/bin, where nsh, ship and core live. Already recorded at G0: uwsm never launches a shell and drops SHELL; the systemd user manager supplies SHELL from passwd, so chsh would change SHELL for every session app. WRITTEN ANSWER: bash stays the passwd shell and chsh is not opened. Three independent reasons: (1) pam_shells refuses SDDM autologin and tty logins for an unlisted shell and the fix needs root; (2) $SHELL -c callers would meet the digit-guard divergence and the F-0105 stdout pollution; (3) a login nsh reads no profile and starts with no PATH to its own tools. The interactive path is the handoff (G2), which needs none of this. INT-288 (SafeShell) runs only if this answer opens chsh, so it stays unstarted; INT-295's edge is unaffected. -->
- [x] G9 Doors: nsh on PATH, nsh-test all passing, d 0 failed,
      core intent validate clean; START HERE record; NOVASHELL.md and
      AGENTS.md changes proposed, not made.
<!-- evidence 2026-10-27: G9 doors on the shipped binary: nsh on PATH, nsh-test 298 of 298, d 26 passed 3 warnings 0 failed, core intent validate all 337 valid. No START HERE: intent complete. NOVASHELL.md and AGENTS.md untouched. Proposed, none made: NOVASHELL.md hangup line (F-0091) and TTY2 correction; F-0101 and keybind idea in his bindings.lua; F-0103 ssh alias in config.nsh; F-0102 and F-0106 ruling; F-0105 fix. Close-the-window bugs first: F-0089, F-0091, F-0096, F-0097. -->

## Non-goals
- chsh or any /etc/passwd change
- SafeShell or any wrapper binary in /etc/passwd (the login chain never
  needs one; the off switch and the always-bash door cover the wish
  behind it)
- SafeShell itself: INT-288, which runs only if G8's written answer opens chsh,
  and reads G2's off switch
- nsh reading /etc/profile or Omarchy's bash scripts
- modifying Omarchy's own files (/usr/share/omarchy, /etc,
  /usr/local/share/wayland-sessions)
- test user accounts (they need sudo)
- the per-segment safety-guard defect, the -c digit-guard fix and the
  main.rs refactor (separate findings)
- sudo, anywhere

## Rulings (G2, 2026-10-26)
Christian ruled all three as recommended.

1. Crash mechanism: (a). nsh runs as a child of the interactive bash. Status 101,
   132, 134, 135 or 139 (panic, SIGILL, SIGABRT, SIGBUS, SIGSEGV) stays in bash
   with one stderr line naming the code. Any other status, including 0, 74, 129,
   141 and a typed exit N, exits bash with nsh's status. Residual: a typed
   exit 101 lands in bash.
   Measured basis (2026-10-26): no Cargo.toml sets panic, so the default unwind
   applies and a panic exits 101 (fsearch panic --file Cargo.toml: no results;
   control fsearch edition --file Cargo.toml: 21 crates); a bare exit returns 0
   (commands/mod.rs:1202-1207); 141 is the broken-pipe hook (main.rs:861-865);
   129 is SIGHUP re-raised (main.rs:1236-1240); 74 is INT-286 gave-up
   (main.rs:3443-3444).
   Rows G1 and G5 must demonstrate: Ctrl-Z at the nsh prompt does not suspend
   nsh; exec of the deployed nsh from inside nsh still works with bash as parent.
2. Off switch: the file ~/.config/nsh/off. Off is touch, on is rm. It is the
   user own directory and not an /etc/skel file, so omarchy-reinstall-configs
   does not overwrite it.
3. Always-bash door: a Hyprland bind in the user own bindings file that launches
   a terminal with NSH_ACTIVE=1. No Omarchy file is touched. A launcher entry is
   optional later.

Finding, by reading and demonstrated at G2: a tty login runs the same handoff
(bash, .bash_profile, .bashrc, exec nsh), so a startup crash ends a tty login
too. AGENTS.md Recovery says TTY2 depends on nothing this project ships. A
correction is proposed at G9, not made.

## G2 evidence (2026-10-26, partial; nothing ticked)
Red first, on a copy of the handoff and stand-ins, never the live ~/.bashrc or the
shipped binary: today's exec handoff took the shell down on a stand-in that exits
101 (landed-in-bash never printed), twice.
Green, new handoff (nsh as a child of bash; stay in bash on 101, 132, 134, 135,
139): 10 of 10 cases, run twice: crash 101, crash 139, clean 0, 74, 129, 141,
typed exit 7 (stand-in), off switch, always-bash door, nsh missing.
Real nsh through the copy (bash --rcfile <copy> -i, run from inside nsh):
- R1 lands in nsh: NSH_ACTIVE=1, process tree outer nsh > bash > nsh.
- R2 Ctrl-C at the prompt: nsh survives (echo still-here ran).
- R3 Ctrl-Z at the prompt: nothing happens, no stopped job, no fallback (the
  next echo $? printed 0).
- R4 typed exit 7: the process exits 0, not 7, with the real nsh alone and through
  the handoff; plain bash exits 7 (control). Cause read: engine.rs:273-275 stores
  the code, main.rs:3311 breaks without reading it, and only gave_up reaches
  process::exit (main.rs:3443). Filed as F-0089. The handoff is correct; the
  typed-exit cell stays unticked until nsh returns the number.
Omarchy 4.0.6 announces crashes: when systemd-coredump records the crash of a
terminal process (SIGSEGV, SIGABRT, SIGILL or SIGBUS) the user unit "Announce
process crashes and offer an AI diagnosis" shows a notification that offers a
claude window. A real nsh crash in the fallback case does the same. MEASURED
2026-10-26: under ulimit -c 0 no core file is written (coredumpctl: COREFILE
none) but the crash is still recorded and still announced, so a crash test
cannot be silent; each real-signal case costs one notification. The exit
status is unaffected.
Real nsh through the copy, pty measurements (2026-10-26; scratch state db, core
limit 0, each run once):
- M1 window close: the pty master is closed; bash ends by signal 1 (SIGHUP), nsh and
  its two core children are gone, no survivors.
- M2 exec reload (C9): exec of the deployed nsh from the child; the prompt returns,
  the nsh pid is unchanged, its parent is still the bash, NSH_ACTIVE=1, and a typed
  exit then ends bash with 0 (F-0089).
- S1 real nsh alone: $$ equals the process pid, and kill -SEGV $$ does not kill nsh.
  nsh catches SIGHUP, SIGINT, SIGBUS, SIGSEGV and SIGWINCH (/proc SigCgt), and its
  own source never names SIGSEGV, so the handler comes from something other than
  nsh's own code (a dependency or the runtime; not identified).
- S2 crash signals sent to the real nsh under the handoff: SIGABRT gives the
  fallback line (status 134) and SIGILL gives it (status 132); bash then answers
  and exit 5 ends bash with 5. SIGSEGV and SIGBUS did not end nsh, so the typed
  exit that followed went to nsh and came back 0 (F-0089).
Closed 2026-10-27: SIGABRT 134 and SIGILL 132 on the real nsh ran twice each, four of
four green (pty script, scratch HOME, core limit 0); SIGSEGV 139 and SIGBUS 135 by the
stand-in status table plus the real 134 and 132 (Christian ruling 2026-10-27); a real nsh
panic exits 101 (F-0096); the always-bash door is the Hyprland bind SUPER + CTRL + SHIFT + B,
shown in a real foot window; the off switch is shown by the real-nsh harness case, and its
real-window check follows the adoption of the block (START HERE step 5).
F-0089 fixed 2026-10-26, commit 7684d9d9 (not pushed): a typed exit N now ends nsh with N.
RED: nsh-test repl_f0089_typed_exit_code_is_the_process_status on the debug build, exit 1,
the only failure: want exit 7, got Some(0). GREEN: cargo fmt --check -p nsh-test -p novashell
-p zero-core clean; cargo build -p novashell -p nsh-test; NSH_BIN=target/debug/nsh nsh-test
--failed: 261 results, All tests passed, exit 0. Shipped nsh 5.3.7 and exec'd into it.
Deployed binary, pty check: real nsh alone, typed exit 7 -> 7; bash + handoff copy, typed
exit 7 -> 7; plain bash control -> 7. nsh-test against the deployed binary: 261 results,
All tests passed; d: 0 failed. The typed-exit cell of the exit-status mapping is
demonstrated on the real binary. Harness finding F-0090: a session the shell ends itself
costs 30s in nsh-test (wait_line_end), so that case runs one session.
F-0090 fixed 2026-10-26, commit e62e9341 (not pushed): the harness kept the pty slave
descriptors alive inside the spawned Command, so the reader never saw EIO; dropping it
after the spawn lets a session the shell ends itself return at once. RED: nsh-test
repl_f0090_session_ended_by_the_shell_returns_promptly on the old harness, 30199 ms, the
only failure. GREEN: 255 ms; the F-0089 case 253 ms (was 30191 ms); 262 of 262 on the debug
build and on the deployed binary; d: 0 failed. Sessions the shell ends itself are now
cheap, so the handoff cases can use them.
Handoff harness and cases, 2026-10-26, commit d0e31e8a (not pushed): nsh-test drives the
handoff through a real bash (run_repl_handoff) with five handoff cases, and the block is
single-sourced at zero/shell/novashell/handoff/bashrc-handoff.sh (747 bytes, sha256 starts
ed135d0accfe). Mutation check: replacing exit "$nsh_status" in that block with true turned
three handoff cases red; with the block restored and cargo rebuilt the suite was green again,
267 of 267.
F-0091 fixed 2026-10-26, commit 11999e42 (not pushed): under the handoff nsh is a plain
member of the foreground group, not the session leader, so a closed window gives it EIO and
no SIGHUP (the lost-terminal path, not the hang-up path). The give-up branch wrote to the dead
terminal with eprintln! and the exit path wrote to stdout; both panic on EIO, the REPL thread
panicked and the process exited 1. Evidence: a stderr-to-file trial showed the panic on both
streams before the fix. Fix: the give-up message is written with an error-ignoring write, a
terminal_lost flag tells the jobs and skips the session-end writes, and the process exits
with the give-up status (74). After the fix: nsh-test 267 of 267, doors green, d: 0 failed.
fsearch, F-0092 and F-0093, found while writing the proof tests, fixed 2026-10-26, commits
a38758e5 (fix) and 8e854abc (six cases), not pushed. RED: five of the six new cases failed on
the unfixed shell and the alternation guard stayed green. GREEN: 273 of 273 on the debug
build; deployed nsh and nsh-test identical to the release build; d: 0 failed; the deployed
fsearch refuses an empty alternative, searches a file argument and still alternates.
Filed 2026-10-27, not fixed here, each needing its own intent: F-0094 (multi-line input
skips alias expansion at execution while the guard judges the expanded form; main.rs:3163
against :3125) and F-0095 (a single-quoted word ending in a backslash is held open by the
REPL; expand.rs:301).
Ruling 2026-10-27 (Christian): the real SIGSEGV 139 and SIGBUS 135 cells are proven by the
stand-in status table plus the real SIGABRT 134 and SIGILL 132 already shown. A signal sent with
kill is caught by nsh (S1) and the block sees only an exit number, so no fault hook is added.
F-0096 found and fixed 2026-10-27, commits 2a0cdb72 (fix) and 93c06c46 (case), not pushed: a
panic on the nsh-repl thread ended the process with status 1, not 101, so the handoff would not
have fallen back to bash. Measured on the deployed 5.3.7 with the real nsh, scratch HOME and
state, stdout on /dev/full (every println panics): exit 1 and Error: REPL thread panicked.
RED: repl_f0096_a_repl_thread_panic_exits_101, the only failure of 274, got code Some(1).
GREEN: 274 of 274 on the debug build; the deployed binary exits 101 on the same trial and the
control exits 0; d: 0 failed. Mutation check: setting the status constant back to 1 turned
that one case red; restored, 274 of 274. This demonstrates the G2 cell: a real nsh panic exits
101. One more observation for F-0094: d as the last word of a chain that spans several lines
ran under sh and printed command not found.

## Handoff contract (G1, 2026-10-26)
The handoff is the block in ~/.bashrc that moves an interactive bash into nsh. After
the change (option (a), see Rulings) it runs nsh as a child of bash. This is what must
hold for every way a shell can start. Baselines were measured on 2026-10-26 against the
current exec handoff. A row is ticked in an evidence line only when its proof output is
pasted. Nothing in ~/.bashrc has been edited.

C1 A login bash that is not interactive (bash --login -c) never reaches the handoff.
   Baseline MEASURED: prints the echo, never execs.
   Proof: env -i HOME="$HOME" TERM="$TERM" bash --login -c 'echo reached'
C2 bash -c never reaches the handoff.
   Baseline MEASURED: prints the echo.
   Proof: env -i HOME="$HOME" TERM="$TERM" bash -c 'echo reached'
C3 A terminal opened from Hyprland starts in nsh.
   Baseline MEASURED: nsh pid 15692 has ppid 15666, which is foot (this window), not bash
   or nsh (exec handoff: no bash parent). After option (a) the parent of nsh is bash and the parent
   of that bash is the terminal.
   Proof: /usr/bin/ps -o pid,ppid,comm -C nsh,bash from a real window, plus the comm of
   the terminal pid.
C4 bash typed inside nsh stays bash.
   Baseline MEASURED: NSH_ACTIVE=1 and shell=bash.
   Proof: bash -ic 'echo $NSH_ACTIVE; echo $0'
C5 A terminal launched from inside nsh inherits NSH_ACTIVE and lands in bash (by reading:
   the handoff exports NSH_ACTIVE and uwsm never exports it into the session).
   RULED 2026-10-26 (Christian): keep today's behavior, the least risk; INT-288
   (SafeShell) revisits it. Any change must keep C4 and the guard that
   stops recursion when nsh spawns bash.
C6 nsh PATH holds the OMARCHY_PATH entries, the mise shims, ~/.local/bin and ~/.cargo/bin.
   Baseline MEASURED: OMARCHY_PATH=/usr/share/omarchy; /usr/share/omarchy/bin, the shims,
   ~/.local/bin and ~/.cargo/bin all present. Six mise install dirs (claude, codex,
   copilot, gh, node 26.7.0, opencode) sit ahead of the shims: copilot is new since
   2026-10-05, when there were five. Per-project versions go stale after cd.
   Proof: the python3 PATH listing, one line per entry.
C7 Each export and bash hook that Omarchy rc sets before the exec carries into nsh, has an
   nsh equivalent, or is recorded as a gap. First the stale mise dirs, then mise, starship,
   zoxide, try, fzf, completions: one line of evidence each, measured when G1 runs.
   Baseline MEASURED: MISE_SHELL=bash and STARSHIP_SHELL=bash leak into nsh; SHLVL=1.
   SETTLED: this window is foot (ps: pid 15666, comm foot), so TERM=xterm-256color and the
   unset GHOSTTY_SHELL_FEATURES say nothing about ghostty. The 2026-10-05 values
   (xterm-ghostty, path,ssh-env,title) came from a ghostty window. SETTLED 2026-10-26: Christian uses foot today;
   ghostty, kitty and Rio are installed and all in scope at G6.
C8 The handoff comment (lines 29-31) cites INT-190 and niri-session. INT-190 is cancelled
   and was about shell_history writers. The correction is proposed with the new handoff
   block; Christian edits his own file.
C9 exec of the deployed nsh from inside nsh still works with bash as parent.
   Proof after the change: exec /home/christian/.local/bin/nsh, then ps shows the same
   bash parent.

Exit-status mapping, crash fallback, off switch and the always-bash door are demonstrated
at G2. Ctrl-Z and job control are demonstrated at G5. Terminal integration parity is G6 (foot, ghostty, kitty, Rio).

## Dependencies
depends_on is empty: nothing must finish first. INT-262 closed on
2026-10-06, so nothing holds cistart.

## START HERE
Written 2026-10-27; supersedes the 2026-10-06 entry. State: cistart is done, G0 is ticked, the
three G2 rulings were taken on 2026-10-26, the G1 contract is written, and nothing else is
ticked. Code is committed through 93c06c46 and not pushed: the handoff harness and cases, the
fixes for F-0089, F-0090, F-0091, F-0092, F-0093 and F-0096, nsh 5.3.7 and nsh-test 2.0.10
deployed, nsh-test 274 of 274, d 0 failed. Left uncommitted on purpose: the findings F-0089 to
F-0096, this intent file and the future/ moves. F-0094 and F-0095 are filed and each needs its
own intent (create it with inta). Open a new chat with: ints 277, then this section.

Next, in order:
1. G2, the always-bash door: add a case where NSH_ACTIVE=1 in the launch environment lands in
   bash with the real nsh (a field on HandoffRun in nsh-test repl.rs), red first, then a
   mutation check that removes the guard clause from the block copy.
2. G2, the Hyprland bind: read how the bindings file defines terminal binds, then PROPOSE the
   bind. Christian applies it; no Omarchy file is touched.
3. Close G2: fold the Still to demonstrate paragraph into the evidence, run the handoff cases
   twice as the row requires, then tick G2 with the evidence.
4. G3, G4, G5 (check each cell of its row against the cases already in nsh-test before ticking),
   G6 (foot, ghostty, kitty, Rio), G7 (a d check that the handoff is present, comparing ~/.bashrc
   with the block file), G8, G9 (propose AGENTS.md and NOVASHELL.md changes only, including the
   TTY2 correction).
5. After all green: Christian adopts the block into his own ~/.bashrc with a backup and an open
   window as the undo path. Nothing touches the live ~/.bashrc before that.

A version bump touches four places: Cargo.toml, Cargo.lock, any copy
hardcoded in source, and zero/registry/tools.toml. nsh-test 2.0.1 missed
the registry until core integrity proposal #22 caught it (2026-10-06).

Carried method: a payload that spans two steps finds its first step by a
marker that survives rustfmt, never by its bytes; one ship at the end of
a fix series; after ship, reload with a second window open.

## Versions
- nsh 5.3.7 -> 5.3.8 (patch)
- nsh-test 2.0.10 -> 2.0.11 (patch)
- zero-core 1.0.4 -> 1.0.5 (patch)
