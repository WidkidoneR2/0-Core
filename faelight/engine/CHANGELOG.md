# Changelog -- core (engine)

## 🌱 3.2.0
- 🗑️ Removed the Arch-era subsystems: package-update domain, stow/link (9-subcommand `core link`), the profile tool, and the arch-audit security scan.
- 🌲 Doctor entropy now detects NixOS store-path drift instead of reading pacman -- the health engine speaks Nix.
- ♻️ Relocated into the unified `faelight/` tree; state routed through the central paths module.

## 3.x -- Friday Becomes Central
- ✨ Friday: the reasoning engine -- observes, correlates, and speaks only when confident.
- ✨ Intent ledger with gate enforcement, health scoring, integrity checks, forecasting.
- ✨ Native domains unified under one engine: health, intent, integrity, prediction, decisions, strategy.

## Earlier
- ⚙️ The forest's single Rust engine -- grown one capability generation at a time.
