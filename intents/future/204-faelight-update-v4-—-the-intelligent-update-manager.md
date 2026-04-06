---
id: 204
date: 2026-04-06
type: planned
title: "faelight-update v4.0.0 — The Intelligent Update Manager"
status: planned
tags: [faelight-update, updates, intelligence, risk, drift, preflight, v4]
---
faelight-update v3 checks for updates.
faelight-update v4 understands them.
Not just "here are 12 updates" but:
- Which ones matter right now
- What risk they carry
- Whether the system is ready
- What will change and how
- What to do after
Every update gets a risk level:
  🔴 Critical  — kernel, systemd, glibc, mesa (reboot likely)
  🟡 Important  — git, neovim, rust, daily-use tools
  🔵 Optional   — AUR, dev tools, plugins, npm, pip
Display:
  📊 Update Summary
  ────────────────────────────────────
  🔴 Critical:  2 (linux, systemd)
  🟡 Important: 5 (git, neovim, rust)
  🔵 Optional: 12 (AUR, npm, pip)
  ────────────────────────────────────
Track how stale the system is getting:
  🕒 Last full upgrade: 2 days ago
  📉 Drift score: LOW  (0-3 days = LOW, 4-7 = MEDIUM, 8+ = HIGH)
Computed from:
- Days since last pacman -Syu
- Number of pending updates
- Whether any critical packages are pending
Before updating, check system readiness:
  ⚠️  Pre-Update Warnings
  ────────────────────────
  • 2 .pacnew files need review (run: pacdiff)
  • Mirrorlist older than 14 days (run: reflector)
  • Disk space: 91% used — clean before updating
  • Partial upgrade risk: pacman db out of sync
Show timing per section:
  ⏱️  Check Performance
  ────────────────────────
  pacman check:    3.2s
  AUR (paru):      6.8s
  cargo:           2.1s
  neovim:          0.4s
  total:          12.5s
  faelight-update --maintain
Runs:
- pacman -Sc (cache cleanup)
- Remove orphan packages
- Clean cargo cache
- Vacuum systemd journal
- Report what was cleaned
After every run:
  💡 Suggestions
  ────────────────────────
  • Run pacdiff to resolve 2 config files
  • Reboot recommended (kernel updated)
  • 12 orphan packages found — run: pacman -Rns $(pacman -Qtdq)
  • Mirrorlist stale — run: reflector
At the top of every run:
  🧬 System Profile
  ────────────────────────
  Host:    faelight
  Kernel:  6.8.7-arch1-1
  Shell:   fsh v0.6.0
  WM:      Niri
  Uptime:  3h 21m
  Health:  100%
Every run logged to state.db:
  core events filter domain=update
  Last 5 runs, packages updated, duration, outcome
Show exactly what would change:
  faelight-update --preview
  📦 pacman:
    + linux 6.8 → 6.9
    + mesa 24.0 → 24.1
  🦀 cargo:
    + ripgrep 13.0 → 14.0
📦 System Packages
🔷 AUR Packages
🦀 Cargo Tools
📝 Neovim Plugins
🌲 0-Core Workspace
🦀 Rust Toolchain
📦 NPM Packages
🐍 Python Packages
📁 Yazi Packages
🔄 Git Repositories
⚡ Firmware
📦 Flatpak
faelight-update bumped to 4.0.0 in Cargo.toml and registry.
⬜ Version bumped to 4.0.0 in Cargo.toml and registry
⬜ Risk categorization — 🔴🟡🔵 levels on all updates
⬜ System Identity header on every run
⬜ Drift score — days since last upgrade + freshness label
⬜ Pre-flight warnings — disk, mirror age, pacnew, partial upgrade
⬜ Performance breakdown — timing per section
⬜ Maintenance mode — faelight-update --maintain
⬜ Suggestions section after every run
⬜ Preview mode -- faelight-update --preview
⬜ Update history logged to state.db
⬜ All original icons preserved exactly
⬜ deploy faelight-update and d passes 100%
"An update manager that only lists packages
is a weather app that shows only temperature.
v4 shows the storm coming,
whether you are ready for it,
and what to pack." 🌲
