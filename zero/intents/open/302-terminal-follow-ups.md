---
id: 302
date: 2026-10-31
type: future
title: "Terminal Follow-Ups"
status: planned
tags: [nsh, novashell, ghostty, agent, f0101]
---

## Vision
Opening a terminal from the keyboard puts you where you were working, on every terminal installed,
and the notes the project keeps about it say what is true. Three loose ends from INT-301 are closed
here the honest way: F-0101 is decided and either fixed or knowingly accepted, F-0108 is measured
in a real ghostty window with a control, and the proposals for docs/NOVASHELL.md and AGENTS.md are
reviewed by Christian and applied only if he says so. No agent runs sudo, nothing of Omarchy's own
is edited, and nothing new is downloaded.

## The Problem
INT-301 (closed 2026-10-31, 0d25fe75) made a new window or split opened from inside a terminal
keep its directory (foot and ghostty proved). It left three things open on purpose.

1. F-0101, SUPER+Return opens in HOME. Measured 2026-10-31, read only.
   - /usr/share/omarchy/bin/omarchy-cmd-terminal-cwd gets the focused window's pid from hyprctl
     activewindow, takes that pid's child with the highest pid (pgrep -P, tail -n1), reads
     /proc/<child>/cwd, and keeps it only if /proc/<child>/exe is a line in /etc/shells. If not,
     or the directory is gone, it prints HOME. Kitty is the one terminal it asks directly.
   - /etc/shells lists sh, bash, rbash, git-shell and systemd-home-fallback-shell. nsh is not there.
   - Under the bash handoff (zero/shell/novashell/handoff/bashrc-handoff.sh) the terminal's child
     is bash, which is listed, and nsh runs as bash's child, not exec'd. So the /etc/shells check
     passes and the directory read is bash's, which never moves when you cd in nsh. Listing nsh
     in /etc/shells would change nothing on its own. The handoff decides today.
   - Not measured: the command SUPER+Return runs. ~/.config/hypr/bindings.lua:35 only says foot
     is the fallback on SUPER+CTRL+SHIFT+RETURN "for when Ghostty will not open", so the main
     terminal is probably Ghostty, but the binding itself has not been read.
2. F-0108, the terminal-side effect of nsh's OSC 133 marks. nsh emits all four marks (A, B, C, D)
   and the harness checks them. foot jumps between prompts on them in a real window (measured
   2026-10-27). In ghostty the one visible effect, the close confirmation, is switched off by
   confirm-close-surface=false in Christian's own ghostty config, so it shows nothing either way.
   Seeing the marks work there needs a prompt-jump key. Rio, the other terminal, is uninstalled.
3. Two sets of proposals written in INT-301 and applied nowhere. docs/NOVASHELL.md and
   docs/public/NOVASHELL.md say nothing about where nsh starts. AGENTS.md says a commit carries
   Fixes: F-x when it closes a finding (line 219) and that open or solved is read from git (rule 1,
   line 225), but not that a subject-line mention does not close a finding, that a late trailer
   dates the fix to the trailer's day, or how to close a finding without a fix.
   The proposed text, verbatim:

   For docs/NOVASHELL.md and docs/public/NOVASHELL.md, one short subsection:
       Starting directory. A new nsh starts in the directory the terminal handed it, so a new
       window or split opened from a shell in /tmp starts in /tmp. If that directory is your home
       directory, or it no longer exists or is not a directory, nsh starts in ~/0-core instead.
       NSH_KEEP_CWD=1 keeps whatever directory nsh was started in, home included. The window
       title and the directory the terminal is told about (OSC 7) are updated at every prompt.
   For AGENTS.md, two sentences:
   1. After the Fixes: sentence at line 219: "Only a Fixes: trailer closes a finding; naming it in
      a subject line does not. Put the trailer on the commit that makes the fix, because the
      solved day is the day of the commit that carries it."
   2. In the findings section: "A finding that is not a defect, or is another program's
      behaviour, is closed without a fix through its closed_without_fix field, written by a
      reviewed plan with the reason and the evidence. core has no close command. A later Fixes:
      trailer still wins over a closure."
   Evidence for them: F-0099 and F-0100 were fixed on 2026-10-27 by c78e7924, which named them
   only in its subject, and read solved on 2026-10-31 once c6c1ff53 carried the trailers. F-0107
   was closed through its record field in a1e5e0f5. The full recon is in
   zero/intents/complete/301-terminal-directory-and-title.md.

Out of scope: F-0103 (ssh sends the terminal's own TERM). Rio. Anything that needs sudo.

## The Solution
Three parts, in this order, one concern per change, red first wherever code changes.
A. F-0101. Read the binding, re-measure the chain in foot and ghostty, then Christian chooses
   among the options below (or one the recon finds). Only then is anything built.
   1. A binding of his own in ~/.config/hypr/bindings.lua that runs a small script of ours in
      ~/.local/bin. Cost: the script, reading the current binding first, and keeping it in step
      if Omarchy changes how it launches a terminal.
   2. nsh publishes its directory, one file per nsh pid under $XDG_RUNTIME_DIR/nsh, written at
      each prompt and removed on exit, so the script in option 1 can find the right nsh under the
      focused window. Cost: a small write per prompt, cleanup, the pid mapping. Only useful with 1.
   3. exec nsh from the handoff and list nsh in /etc/shells. Needs sudo, which is Christian's to
      run and never an agent's, and drops the bash fallback when nsh crashes. Not recommended.
   4. Ask Omarchy upstream to follow the focused terminal's deepest descendant, or its OSC 7,
      instead of the last direct child. Costs nothing here, depends on them.
   5. Leave it. SUPER+Return opens in HOME, and a window or split from inside a terminal already
      keeps the directory.
B. F-0108. Measure the OSC 133 effect in a real ghostty window with a control, without editing
   his ghostty config if the command line can carry the key. Record what was seen.
C. The proposals. Recon where the text belongs and how the public copy is made, Christian rules on
   each text, and only what he approves is applied, by a reviewed plan.

## Success Criteria
- [ ] G1 Recon of F-0101 re-measured, nothing assumed from INT-301. (a) The command SUPER+Return
      runs, from hyprctl binds and ~/.config/hypr/bindings.lua, with file:line. (b) In a foot
      window and in a ghostty window, with nsh sitting in /tmp: the terminal's pid, its child
      pids with their exe and cwd, which child omarchy-cmd-terminal-cwd would pick (highest pid),
      and what it prints, worked out by reading /proc and running the script's own steps, not by
      guessing. (c) The process chain labelled as in INT-301 G6 (nsh <- bash <- terminal).
      (d) Which terminals are installed. Proof: the pasted output, labelled per terminal.
- [ ] G2 Christian rules on F-0101: which option (1 to 5, or a new one from G1), written here as a
      RULED line with the date. If he picks 5, G3 to G6 are recorded as not needed with that
      reason; if he picks 4, the ask is drafted here and the finding stays open. Proof: the RULED
      line. No code is written before this line exists.
- [ ] G3 RED, only if the ruled option changes nsh or adds a script: a failing test or harness row
      under its own plan Seal that names what it got (for option 2, the runtime file is missing
      at a prompt; for option 1, the script prints HOME for a window whose nsh is in /tmp). Proof:
      the red output under its Seal. If no code is involved, this gate says so and why.
- [ ] G4 The ruled option is built through reviewed plans with Seals, one concern per plan. No
      sudo. No file of Omarchy's is touched: the sha256 of /usr/share/omarchy/bin/omarchy-cmd-
      terminal-cwd is taken before the first plan and again at the end and is the same. Any edit
      to ~/.config/hypr/bindings.lua is its own plan, shown and approved first, with a revert plan
      written beside it. Proof: the Seals, and the two hashes.
- [ ] G5 Real windows: SUPER+Return pressed with a ghostty window focused and nsh in /tmp opens a
      new window in /tmp; the same from foot; the same with the focused nsh in HOME gives the core
      root through the INT-301 rule; and with nothing focused the result is recorded. Each with a
      PROOF line (cwd, process chain, title via hyprctl activewindow). Proof: those lines.
- [ ] G6 Doors for part A after one ship: nsh on PATH is the shipped build, nsh-test all passing,
      d 0 failed, core intent validate all valid, the Omarchy script hash unchanged. Proof: the
      door output.
- [ ] G7 F-0108 recon, read only: ghostty version; ghostty's own list of its actions to find the
      prompt-jump action by its real name (ghostty +list-actions, confirmed with ghostty +help);
      the keybinds already in his ghostty config with file:line; the confirm-close-surface line;
      whether ghostty accepts a key on its command line (--keybind=...). Proof: the pasted output.
- [ ] G8 The control is written down before the test: a ghostty window started with the jump key
      given on the command line (so his config is not edited), nsh with its marks, a few commands
      run, the key pressed, and the result read; then a control in the same kind of window with a
      shell that emits no OSC 133 (bash --norc) and the same key. If the command line cannot carry
      the key, a temporary key in his config is a separate plan, shown and approved first, with
      its revert. Proof: the procedure in this file, then the observation for both.
- [ ] G9 F-0108 is recorded honestly. If the jump works in nsh and not in the control, F-0108 is
      closed through its record field with that evidence (not a fix, no Fixes: trailer). If it
      does not work in nsh, a red row for the marks comes first and the fix carries Fixes:
      F-0108. If the key cannot be tested, the finding stays open and says what is missing.
      Proof: core bug show F-0108.
- [ ] G10 Proposals recon, read only: the headings of docs/NOVASHELL.md and
      docs/public/NOVASHELL.md and where startup or environment variables are described; how the
      public copy is made (a script, a mirror, or by hand) from git log and the tree; AGENTS.md
      lines 210 to 235 as they stand; and the exact anchors the plans would use. Proof: the
      pasted output and the exact placement proposed for each text.
- [ ] G11 Christian reviews and rules, per text: the NOVASHELL subsection (apply, change, or drop),
      AGENTS.md sentence 1, AGENTS.md sentence 2. His wording wins. Each ruling is written here
      with the date. Nothing in docs/ or AGENTS.md changes before these lines exist.
- [ ] G12 Only what he approved is applied, each file by its own reviewed fpatch Plan with a Seal
      and the apply line sent after he has seen the plan. The ten devshell laws and every other
      line of AGENTS.md stay byte for byte (diff shows only the added sentences). The public copy
      of NOVASHELL.md matches the way it is made. Proof: the plans, the Seals, and git diff.
- [ ] G13 The records: F-0101 closes by the route that is true for the ruled option, a Fixes:
      F-0101 trailer on the commit that fixes it, or its record field with the reason if it was
      accepted, or it stays open with the upstream ask named. F-0108 as in G9. Proof: core bug
      show F-0101 and F-0108, and core bug welcome with the new count.
- [ ] G14 Final doors after one ship: nsh-test all passing, d 0 failed, core intent validate all
      valid, git status clean, then cicomplete 302 with its version bumps. Proof: the door output
      and the close commit.

## Constraints (Christian's rules, restated here so a new chat has them)
- No agent ever runs or sends sudo. A step that needs it is written out for Christian to run.
- Never modify Omarchy's own code. ~/.config/hypr/bindings.lua and his ghostty config are his own
  files and change only through a reviewed plan he has seen.
- AGENTS.md and docs/NOVASHELL.md are his. Proposals only until he rules (G11); applied only by a
  reviewed plan (G12). The ten devshell laws are not touched.
- Nothing new is downloaded and no new tooling; use what is on the box.
- One concern per change, red first, doors before every commit, one ship at the end.
- Payloads: every block decoded and compared byte for byte before it is sent, and the apply line
  sent only after he has seen the plan and its Seal.
