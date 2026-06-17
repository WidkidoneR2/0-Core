---
id: 064
date: 2026-06-16
type: feature
title: \"faelight-logout: candy-neon Wayland power menu\"
status: complete
tags: [logout, power-menu, wayland, mango, gtk4, python, neon-candy, int-033, ux]
version: TBD
priority: medium
---

## Why
MangoWM has no graceful power / logout UI -- today, leaving the session means
typing a systemctl command or killing the compositor by hand. This intent adds
a forest-styled power menu: a fullscreen overlay with four big candy-neon
squares -- Shutdown, Reboot, Logout, Lock -- each a symbol inside a square with
its label beneath, launched by a mango keybind, dismissed with Esc.

Deliberate decision recorded here: this piece is built in Python, not Rust, and
that is fine. The ~98%-Rust thesis is already proven; it does not need
re-proving on a logout box. Python + GTK4 is the pragmatic, best-looking path to
the candy-neon overlay, and choosing the right tool for the job IS the
philosophy, not a departure from it.

## What Already Exists
- INT-033 neon-candy semantic colors (theme.rs, prompt.rs, faelight-fm,
  faelight-bar) -- the palette this menu reuses so it matches the rest of the
  forest.
- MangoWM keybind system -- ~/.config/mango/config.conf, deployed declaratively
  via home-manager; a keybind launches the menu.
- Reference only: trizen/oblogout-py3 -- a Gtk3/Cairo logout menu for Openbox on
  X11, last released 2020, with dead HAL/ConsoleKit backends. A useful VISUAL/UX
  reference (fullscreen overlay, icon buttons + labels, per-button commands) but
  it will NOT run on Wayland; we rebuild Wayland-native, not fork.
- Login screen: OUT OF SCOPE. A separate login solution is already in the works.
  This intent touches ONLY the logout/power menu and makes NO changes to greetd
  or the login path -- which is exactly what keeps it lockout-risk-free.

## Vision
A fullscreen overlay over MangoWM showing four big neon squares in a 2x2 grid.
Each square: a clear symbol inside, a word label beneath (Shutdown / Reboot /
Logout / Lock), bright forest candy-neon styling (glowing borders, rounded
squares) drawn from the INT-033 palette. Keyboard-navigable (arrows + a letter
per action) and mouse-clickable. Esc cancels with no action taken.

## Approach
- Stack: Python + GTK4 + gtk4-layer-shell. The layer-shell binding gives a
  proper Wayland overlay surface (fullscreen, above normal windows) under
  MangoWM -- the Wayland-native equivalent of what oblogout did on X11.
- Styling: GTK CSS, colors pulled from the INT-033 neon-candy tokens so the menu
  is visually continuous with the prompt, bar, and faelight-fm. Neon look =
  bright accent borders + glow (box-shadow) on rounded square buttons.
- Actions (subprocess):
    Shutdown -> systemctl poweroff
    Reboot   -> systemctl reboot
    Logout   -> end the mango session (terminate the logind session / quit the
                compositor) so we return to the login path
    Lock     -> invoke a Wayland screen locker (chosen in Phase 0)
- UI safety: Esc always cancels; no action fires without an explicit click or
  confirm-key, so the overlay can never trigger an accidental shutdown just by
  appearing.
- Packaging: the app + its deps (python3, pygobject, gtk4, gtk4-layer-shell, the
  chosen locker) are added declaratively, and the mango keybind to launch it is
  added to config.conf declaratively. Nothing imperative.

## Phases
Phase 0 -- pick the Lock backend
  Choose the Wayland screen locker for the Lock square (e.g. gtklock or
  swaylock), package it declaratively, confirm it actually locks under MangoWM.
  Gate: a Wayland locker is chosen and locks/unlocks correctly under MangoWM

Phase 1 -- functional overlay (correctness before beauty)
  Minimal GTK4 + layer-shell fullscreen overlay with four buttons wired to the
  four commands. Verify EACH action does exactly the right thing -- test
  Shutdown / Reboot deliberately (destructive to the session), confirm Logout
  returns to the login path, confirm Lock locks. Esc cancels cleanly.
  Gate: overlay launches as a fullscreen Wayland surface over MangoWM
  Gate: each of the four actions (Shutdown, Reboot, Logout, Lock) verified to
        perform the correct system action
  Gate: Esc cancels with no action taken (no accidental power events)

Phase 2 -- candy-neon styling
  Apply the INT-033 neon-candy palette via GTK CSS: 2x2 grid of big rounded
  squares, symbol + label each, glowing neon borders, readable contrast.
  Gate: the menu renders as four big candy-neon squares with symbol + label,
        visually consistent with the INT-033 forest palette

Phase 3 -- declarative packaging + keybind
  Package the app and deps declaratively; add the mango keybind to launch it
  (config.conf, home-manager). Survives a clean rebuild + fresh session.
  Gate: app + deps install declaratively and the mango keybind launches the menu
        after a rebuild

Phase 4 -- scope guard (login stays separate)
  Confirm this intent changed nothing in greetd / the login path; the login
  work remains its own separate effort.
  Gate: no greetd or login-path changes are part of this intent

## Gates
- [ ] Wayland locker chosen and locks/unlocks correctly under MangoWM (Phase 0)
- [ ] overlay launches as a fullscreen Wayland surface over MangoWM (Phase 1)
- [ ] all four actions verified: Shutdown, Reboot, Logout, Lock (Phase 1)
- [ ] Esc cancels with no action taken -- no accidental power events (Phase 1)
- [ ] menu renders as four big candy-neon squares, symbol + label, INT-033
      palette (Phase 2)
- [ ] app + deps + mango keybind all declarative, survive a rebuild (Phase 3)
- [ ] no greetd / login-path changes in this intent (Phase 4 scope guard)

## Depends On
  INT-033 (neon-candy palette -- the menu reuses these tokens)

## The Rule
"Leaving should feel like the forest, too --
 four bright doors, clearly marked, no accidental goodbyes." 🌲
