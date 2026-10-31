---
id: 301
date: 2026-10-31
type: future
title: "Terminal Directory And Title"
status: in-progress
tags: []
---

## Vision
A new terminal window or split opens where you were working, the window title says where you
are, and for every installed terminal it is proved which half of that works because of nsh and
which half the terminal or Omarchy owns. Nothing nsh already does is claimed again, and nothing
outside nsh is hidden.

## The Problem
Seven findings from INT-277 (2026-10-27) describe one contract, the terminal hands the shell a
directory and reads a title and a cwd back. Measured 2026-10-31 on the tree:

- F-0099 (OSC 7) and F-0100 (title): FIXED already. prompt.rs `terminal_marks` writes OSC 2 and
  OSC 7 at every prompt (prompt.rs:61-80), commit c78e7924 (INT-277 G6), proved in foot 1.28.0 and
  ghostty 1.3.1. They show open only because that commit names them in its subject and carries no
  `Fixes:` trailer, and core bug marks a finding solved by the trailer alone.
- F-0102 and F-0106: LIVE. main.rs:2558 moves nsh to the core root at startup unless
  NSH_KEEP_CWD is set, so a directory a terminal hands over never survives, even when OSC 7 and
  foot's spawn-terminal did their part. INT-261 added that rule to stop a restored last_dir; its
  real bug (the second move) is already gone. RULED 2026-10-31 (Christian): honour the launch
  directory unless it is HOME; HOME or nothing still starts in ~/0-core; NSH_KEEP_CWD still
  overrides. This supersedes the 2026-10-27 F-0102 ruling to keep the rule unchanged.
- F-0101: OUTSIDE nsh. omarchy-cmd-terminal-cwd drops the cwd unless the process exe is in
  /etc/shells, and nsh is not (SUPER+Return opens in HOME). No sudo and no Omarchy edits, so this
  intent records a proposal and leaves the finding open.
- F-0107 (Rio ignores OSC 7) and F-0108 (OSC 133 effect not observed in ghostty or Rio): the
  terminal's behaviour, with nsh's side already measured correct.

Out of scope: F-0103 (ssh sends the terminal's own TERM), the same INT-277 group but ssh, not
directory or title.

## The Solution
1. Red first: harness rows that start nsh WITHOUT NSH_KEEP_CWD in a non-HOME directory and in HOME,
   failing on today's binary under their own plan Seal.
2. One pure function `startup_dir(launch, home, core_root, keep)` owns the rule; main.rs calls
   it where it calls `cwd::chdir(&core_root)` today. Still ONE startup rule, now in one place.
3. Prove it in real foot and ghostty windows, and Rio as the documented terminal-side gap.
4. Close F-0099, F-0100, F-0102 and F-0106 with `Fixes:` trailers on the commits that earn
   them; F-0107 and F-0108 through core's own path after recon; F-0101 stays open with a
   written proposal.

## Success Criteria
- [x] G1 Recon re-measured on the shipped binary before any edit: nsh in a real PTY at /tmp writes
      OSC 2 and OSC 7 at the first and a later prompt, with the bytes captured; the one chdir at
      startup and every reader of NSH_KEEP_CWD listed with file:line; the harness cases that set
      NSH_KEEP_CWD listed. Proof: the pasted capture and listing in this gate's evidence.
      <!-- evidence: 2026-10-31 recon on the shipped binary, pasted in the working session: a real PTY at /tmp showed OSC 2 and OSC 7 at the first and a later prompt (prompt.rs:61-80); the startup chdir is main.rs:2558; readers of NSH_KEEP_CWD and the harness cases that set it were listed before any edit. -->
- [x] G2 RED: a harness row starts nsh with NSH_KEEP_CWD unset in a temp directory that is not HOME
      and asserts the shell's cwd is that directory; it fails on the unfixed binary naming the
      directory it landed in (~/0-core). A second row starts in HOME and asserts ~/0-core. Proof:
      the red output under its plan Seal.
      <!-- evidence: red plan Seal e524742810c27001: the new row started nsh from a temp directory that is not HOME and failed on the unfixed binary naming ~/0-core as where it landed. -->
- [x] G3 The rule `startup_dir` has unit tests for: a non-HOME existing directory is kept; HOME
      goes to the core root; HOME reached through a symlink or a trailing slash goes to the core
      root; a missing path or a file goes to the core root; NSH_KEEP_CWD=1 keeps whatever it is
      given; NSH_KEEP_CWD=0 does not keep. Proof: cargo test names and counts.
      <!-- evidence: commit c39fa885, Seal 23d7de0a5c6dd1bb: cargo test -p novashell cwd::tests, 6 passed 0 failed (non-HOME kept; HOME to core root; trailing slash or symlink HOME; unusable path to core root; keep keeps; no HOME keeps). -->
- [x] G4 The G2 rows go green on the same plan, and the INT-261 case
      repl_start_directory_ignores_remembered_last_dir is still green, so the class that rule was
      written for stays fixed. Proof: nsh-test output.
      <!-- evidence: nsh-test --category=repl --failed all passed on the fix plan, including repl_start_directory_ignores_remembered_last_dir, which now launches from $HOME; full run after ship 301/301 (commit c39fa885). -->
- [x] G5 There is exactly one startup directory rule: the comment above it (main.rs:2552),
      `--help` (main.rs:991) and the INT-206 note describe the new rule, and a search shows no
      second startup chdir. Proof: fsearch output.
      <!-- evidence: commit c39fa885: the one startup rule is cwd::startup_dir, called once in main.rs; the comment, keep_launch_cwd doc and --help text describe it; the fsearch listing showed no second startup chdir. -->
- [x] G6 Real windows, process chain labelled as in INT-277 G6: foot, Ctrl+Shift+N from nsh in
      /tmp opens nsh in /tmp; the same from HOME opens ~/0-core; ghostty, a split from /tmp does
      the same. Proof: pwd and the process chain from each window.
      <!-- evidence: demonstrated 2026-10-31 with a PROOF line (cwd, parent chain, hyprctl title) from each window. foot: Ctrl+Shift+N from nsh in /tmp opened nsh in /tmp; the same from HOME gave cwd=/home/christian/0-core chain=nsh <- bash <- foot <- foot <- foot <- Hyprland. ghostty: from /tmp gave cwd=/tmp chain=nsh <- bash <- ghostty <- systemd. SUPER+Return was not measured (F-0101 owns it). -->
- [x] G7 The title and OSC 7 agree with the directory nsh actually landed in on the FIRST prompt
      of a window started in /tmp (no stale core-root title). Proof: captured bytes or the
      window title through hyprctl activewindow.
      <!-- evidence: demonstrated 2026-10-31: the ghostty window started in /tmp read title=christian@christian:/tmp on its first prompt, and the foot window started in HOME read title=christian@christian:~/0-core, so the title agrees with where nsh landed and no stale core-root title shows. -->
- [x] G8 Rio, window opened from nsh in /tmp: the result is measured and written down as the
      terminal-side gap in F-0107 (Rio never sends the directory), not claimed fixed. Proof: the
      process chain and cwd from the Rio window.
      <!-- evidence: demonstrated 2026-10-31: Rio opened from nsh in /tmp landed in cwd=/home/christian/0-core, because Rio never sends the directory (F-0107, a terminal-side gap, not claimed fixed). Christian then uninstalled Rio. F-0108 (OSC 133 in ghostty) is untouched. -->
- [x] G9 Closing the records honestly: the commit that adds the evidence for F-0099 and F-0100
      carries `Fixes: F-0099` and `Fixes: F-0100` and names c78e7924 in its body, and says the
      solved day will read 2026-10-31 not 2026-10-27; the rule commit carries `Fixes: F-0102`
      and `Fixes: F-0106`. Proof: core bug show for each reads solved.
      <!-- evidence: demonstrated 2026-10-31 after c6c1ff53 (Fixes F-0099 and F-0100, names c78e7924) and c39fa885 (Fixes F-0102 and F-0106): core bug show prints state solved and solved 2026-10-31 for all four. The solved day for F-0099 and F-0100 reads 2026-10-31, the day the trailer landed, not 2026-10-27. -->
- [x] G10 F-0107 and F-0108: recon names the command core has for closing a finding without a
      fix; each is closed with the terminal-side reason and the measured evidence, or, if core has
      no such path, a finding is filed and the two stay open. Proof: core bug show.
      <!-- evidence: recon 2026-10-31. core bug has dashboard, welcome, list, show, check, rate and promote, no close command. A closure is the record field closed_without_fix (trace.rs:130, bug.rs), and a later Fixes commit still wins over it. F-0107 was closed that way in a1e5e0f5 with Rio's measured behaviour as the reason and core bug show F-0107 reads closed. F-0108 was deliberately left open, its ghostty half was never observed (the close confirmation is off in the ghostty config and seeing the marks needs a keybind), so core bug show F-0108 reads open. The gate said each is closed, it is met for F-0107 only and F-0108 stays open on purpose. -->
- [x] G11 F-0101: a Proposal section in this intent states the measured cause
      (/proc exe versus /etc/shells), the options, and what each would cost, with no sudo and no
      Omarchy edit applied. The finding stays open. Proof: the section, and core bug show F-0101
      still open.
      <!-- evidence: the Proposal section at the end of this file, from a read-only recon on 2026-10-31 of omarchy-cmd-terminal-cwd, /etc/shells and the handoff. core bug show F-0101 read state open after a1e5e0f5. No sudo and no Omarchy edit was made. -->
- [ ] G12 Doors: after one ship, nsh on PATH, nsh-test all passing, d 0 failed, core intent
      validate all valid, `bugs:` welcome count lower by the findings closed here. Proposals to
      NOVASHELL.md and AGENTS.md are written in this intent, not applied. Proof: the door output.

## Proposal: F-0101 (SUPER+Return opens in HOME)

Measured 2026-10-31, read-only, nothing applied.

- omarchy-cmd-terminal-cwd (/usr/share/omarchy/bin) gets the terminal's pid from hyprctl
  activewindow, takes that pid's child with the highest pid (pgrep -P, tail -n1), reads
  /proc/<child>/cwd, and keeps it only if /proc/<child>/exe is a line in /etc/shells. Otherwise,
  or if the directory is gone, it prints HOME. Kitty is the one terminal it asks directly.
- /etc/shells lists sh, bash, rbash, git-shell and systemd-home-fallback-shell. nsh is not there.
- Under the bash handoff (bashrc-handoff.sh) the terminal's child is bash, which is listed, and
  nsh runs as bash's child. So the /etc/shells check passes and the directory it reads is bash's,
  which never moves when you cd in nsh. Listing nsh in /etc/shells would only matter if nsh were
  the terminal's direct child, so on its own it changes nothing. The handoff is what decides today.
- Ctrl+Shift+N in foot and a ghostty split already keep the directory (INT-301), because they use
  the terminal's own spawn and OSC 7, not this script.
- Not measured: the command SUPER+Return runs. bindings.lua:35 only names the foot fallback, so
  that binding has to be read before option 1 or 2.

Options, with what each costs:

1. A binding of your own in ~/.config/hypr/bindings.lua (your file, no Omarchy edit) that runs a
   small script of ours in ~/.local/bin. Cost: the script, reading the current binding first, and
   keeping it in step if Omarchy changes how it launches a terminal.
2. nsh publishes its directory, for example one file per nsh pid under $XDG_RUNTIME_DIR/nsh,
   written at each prompt and removed on exit, so the script in option 1 can find the right nsh
   under the focused window. Cost: a small write per prompt, cleanup, and the pid mapping. It is
   only useful together with option 1.
3. exec nsh from the handoff and list nsh in /etc/shells. Cost: needs sudo, which is yours to run
   and never mine, and it drops the bash fallback when nsh crashes. Not recommended.
4. Ask Omarchy upstream to follow the focused terminal's deepest descendant, or its OSC 7, instead
   of the last direct child. Cost: nothing here, but it depends on them.
5. Leave it. SUPER+Return opens in HOME, and a new window or split from inside a terminal already
   keeps the directory.

Nothing is applied and no option is chosen. F-0101 stays open.

