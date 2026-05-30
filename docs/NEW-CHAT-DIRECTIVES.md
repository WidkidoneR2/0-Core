# New Chat Directives -- Faelight Forest
# For: NixOS Migration + R&D Environment
# Written: 2026-05-30
# Author: Christian + Claude (Day 3 Session)

---

## WHO YOU ARE TALKING TO

Christian is the sole developer of Faelight Forest (0-Core), a custom
Arch Linux environment built almost entirely in Rust, running on a
Framework 16 AMD laptop with Niri as the Wayland compositor.

GitHub: https://github.com/WidkidoneR2/0-Core
Forest version: 14.1.0
Intents complete: 284
Health: 100%

The project's thesis: one human + AI partnership can build what teams
of dozens normally build. Graydon Hoare (Rust creator) is watching.
Kernel 8.0 beta is imminent. The work matters.

---

## THE PHILOSOPHY -- READ THIS FIRST

**Understanding over convenience.**
**Manual control over automation.**
**Intentional design.**
**The forest remembers.**

Nothing is rushed. Nothing is skipped. Every gate is demonstrated,
not just implemented. If a gate cannot be built, discuss it first.
Never silently defer.

One step at a time. One gate at a time. One intent at a time.

---

## HOW WE WORK -- MANDATORY RULES

### Before touching anything
1. Run `d` to check forest health
2. Run `unlock-core` before editing any files
3. Run `lock-core` when done
4. Always `cistart INT-NNN` before starting intent work
5. Always `cicomplete INT-NNN` when intent is complete

### Python file editing (CRITICAL)
- Always use `python3 << 'PYEOF'` with single-quoted delimiter
- Always use binary mode for Rust files: `open(path, 'rb'/'wb')`
- Never use `\u{XXXX}` in Python generating Rust -- use literal UTF-8
- Always count replacements: `print(f"Found: {count}")`
- If count is 0, the anchor didn't match -- check exact text first
- Never use sed chains -- one failure, switch to Python

### Checking before editing
```bash
# Always read before writing
sed -n 'START,ENDp' path/to/file.rs
grep -n "pattern" path/to/file.rs | head -10
```

### Building
```bash
# Core domain build order:
# mod.rs → commands.rs → parser.rs → cli/mod.rs → dispatcher.rs → domains/mod.rs → build → deploy

# Always build before deploy
cargo build --release 2>&1 | grep "^error" | head -5

# Always deploy after build
deploy <tool-name> 2>/dev/null | tail -3

# Always check health after deploy
d
```

### Committing
```bash
fg done "INT-NNN: description of what was accomplished"
```

---

## CURRENT FOREST STATE

**Terminals:**
- `Mod+Return` → faelight-term (forest terminal, Rust, GPU-rendered)
- `Mod+Alt+Return` → faelight-ade (Forest ADE, fsh + Friday split)
- Alacritty available for comparison

**Shell:** fsh is the daily driver. zsh retained as fallback.

**Key files:**
- Shell: `~/0-core/rust-tools/faelight-shell/src/main.rs`
- Engine: `~/0-core/engine/src/`
- State DB: `~/0-core/runtime/state.db`
- Niri config: `~/0-core/03-interfaces/stow/niri/.config/niri/config.kdl`
- Alacritty: `~/.config/alacritty/alacritty.toml`

**3 state.db files exist** -- real one is `~/0-core/runtime/state.db`

**Font:** JetBrainsMono Nerd Font 12px -- documented in `docs/forest-typography.md`

---

## WHAT COMES NEXT -- THE SEQUENCE

### Phase 1: faelight-notify v5 (INT-301)
**Goal:** Rebuild the notification daemon as a proper Wayland layer-shell tool.

Before migrating to NixOS, faelight-notify v5 needs to be built on Arch.
This ensures the forest's notification system is solid before the migration.

**Study first (INT-347):**
- noti-rs/noti -- pure Rust Wayland notification daemon, study the architecture
- Noctalia notification system -- study history and DND patterns

**Build:**
- Layer-shell native (not D-Bus proxy hack)
- Per-app urgency colors matching forest palette
- Notification history (Friday-aware)
- Do Not Disturb mode
- Hot-reload config
- No external dependencies beyond Rust + Wayland

**Gate:** `d` shows faelight-notify v5 healthy before moving to R&D

---

### Phase 2: R&D Environment (INT-328)
**Goal:** VM-based sandbox before touching the Framework laptop.

The Framework laptop runs Arch Linux today. NixOS will replace it.
Before migrating the real machine, we build and test everything in a VM.

The R&D environment serves as:
- Safe experimentation space (hypothesis → test → gate → graduate)
- NixOS learning without risk to production system
- Proof that the forest can be reproduced from a flake

**Steps:**
1. Install NixOS minimal in the existing VM (no GUI initially)
2. Create `flake.nix` declaring the entire forest
3. Create `hosts/framework16/` configuration
4. Create `modules/forest/` for fsh, faelight-tools, friday
5. Create `modules/security/` for LUKS, firewall, hardening
6. Get `d` showing 100% health inside the VM
7. Every issue documented as it appears
8. Gate: forest works in VM before touching Framework

### Phase 3: NixOS Security Architecture
**Goal:** Faelight Forest on NixOS is more secure than on Arch, not less.

Security is declarative on NixOS -- not configured manually, declared.
Every security decision is in Git, visible, auditable, reproducible.

**The security stack:**
- LUKS2 full disk encryption (declared in `boot.initrd.luks`)
- Firewall (declared in `networking.firewall`)
- Kernel hardening (declared in `boot.kernel.sysctl`)
- fail2ban (declared in `services.fail2ban`)
- No implicit services -- everything opt-in

Graydon Hoare is interested in the security architecture of Friday+fsh.
The trust contracts (INT-186 Delegation Engine) become security contracts
on NixOS. Friday's access to state.db is bounded and auditable.

### Phase 4: NixOS on Framework (Production)
**Gate:** R&D VM passes all tests first. No exceptions.

Migration sequence:
1. Fresh NixOS minimal ISO (not graphical installer)
2. LUKS2 + disko.nix declarative partitioning
3. `nixos-install --flake .#framework16`
4. One command. The entire forest reproduced.
5. state.db carried forward intact
6. Fresh intent ledger starting at INT-001
7. BIOS update after NixOS is stable (Framework 16 v4.02 available)

**Take photos at every step.** The migration will be documented
with real pictures for step-by-step verification.

### Phase 5: Pinnacle Compositor (Post-NixOS)
**Goal:** i3 ownership model on Wayland.

Niri carries the forest to NixOS (stable, well-understood).
Pinnacle replaces Niri after NixOS is stable.

Pinnacle = declare window management logic yourself in Lua.
That's the i3 ownership model -- you control every window placement.

With Kernel 8.0 beta + Pinnacle + Friday:
- Friday has direct access to compositor state via D-Bus
- Forest self-healing (INT-327) becomes real
- The desktop graph is visible to Friday's intelligence layer

This is what Graydon is watching. A compositor + shell + intelligence
layer running on Kernel 8.0 beta. A demonstration of a different way
systems can work.

---

## THE 0-CORE NIXOS STRUCTURE
0-core/
├── flake.nix                    # declares the entire forest
├── hosts/
│   └── framework16/
│       ├── configuration.nix    # system declaration
│       ├── hardware-config.nix  # auto-generated
│       └── disko.nix            # declarative partitioning
├── modules/
│   ├── forest/
│   │   ├── fsh.nix              # shell as login shell
│   │   ├── faelight-tools.nix   # all tools as derivations
│   │   ├── friday.nix           # friday daemon service
│   │   └── alacritty.nix        # terminal config
│   ├── security/
│   │   ├── luks.nix             # LUKS2 encryption
│   │   ├── firewall.nix         # UFW rules declared
│   │   └── hardening.nix        # kernel hardening, sysctl
│   └── desktop/
│       └── niri.nix             # compositor (→ pinnacle.nix later)
├── users/
│   └── christian/
│       └── home.nix             # home-manager
├── pkgs/
│   └── faelight/                # custom derivations for all tools
├── r-and-d/                     # SEPARATE from forest production
│   ├── experiments/             # hypothesis → test → gate → graduate
│   └── graduated/               # experiments that earned production
├── rust-tools/                  # unchanged -- Rust source
├── engine/                      # unchanged -- core engine
├── intents/                     # unchanged -- the ledger
└── runtime/
└── state.db                 # unchanged -- the forest's memory

**R&D is SEPARATE from production.** Experiments never touch the
production forest until they pass all gates. This is non-negotiable.

---

## WHAT DOES NOT CHANGE

- The intent system (cistart/cicomplete/fg done)
- The gate philosophy (demonstrated not just implemented)
- The forest's tool names and commands
- state.db (carried forward intact)
- The philosophy (understanding over convenience)
- The language (Rust, always Rust)

---

## KNOWN DEFERRALS (build on NixOS)

These were deliberately deferred -- not abandoned:

1. **faelight-bar ADE indicator** (INT-346 Phase 6)
   → faelight-bar v4 (INT-344) rebuilds the bar on NixOS

2. **faelight-term v4 native split panes** (INT-346 final)
   → faelight-term v4 is the real ADE with native splits

3. **Friday ADE deep integration** (INT-346 Phase 5)
   → INT-320 Friday v3 on NixOS

4. **faelight-notify v5** (INT-301)
   → layer-shell rebuild, study noti-rs first (INT-347)

5. **Pinnacle compositor** (INT-343)
   → after NixOS is stable with Niri

6. **Alacritty vs faelight-term decision**
   → 2 weeks daily driving both on NixOS, then decide

---

## STUDY INTENTS (before building)

- INT-347: Noctalia Shell + noti-rs (bar patterns, notification daemon)
- INT-348: Ewwii (widget system as faelight-bar v4 foundation)

Study = read source, extract patterns, document findings.
Study ≠ adopt. The forest owns its tools.

---

## CLOSING PRINCIPLE

"The forest does not fear the storm. It knows how to grow back."

One step at a time.
One gate at a time.
One intent at a time.
No shortcuts.
No rushing.
Pictures at every NixOS milestone.

The forest that runs on Kernel 8.0 beta was built with integrity
from the first commit. That integrity doesn't stop now. 🌲
