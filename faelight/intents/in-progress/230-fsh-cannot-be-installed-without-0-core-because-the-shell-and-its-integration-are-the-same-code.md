---
id: 230
title: "fsh cannot be installed without 0-Core, because the shell and its integration are the same code"
status: in-progress
type: architecture
priority: high
date: 2026-08-23
tags: [fsh, architecture, portability, boundary, int-227]
---

## Vision
fsh owns shell behaviour. 0-Core integration is an optional adapter.

## ✅ RECON FIRST, and it reframed the intent entirely
Measured 2026-08-23. fsh has TWO path dependencies -- `faelight-core` and `faelight-git` -- and is a
workspace member via the `faelight/rust-tools/*` glob.

⭐⭐ **`faelight_core` IS USED AT 68 SITES, AND ALMOST EVERY ONE IS `paths::`:**

    paths::intents_dir()      paths::rust_tools_dir()   paths::state_db()
    paths::registry_dir()     paths::read_health()      paths::core_root_string()
    paths::changelog_file()   paths::runtime_dir()

Across `commands/mod.rs`, `prompt.rs`, `digest.rs`, `session.rs`, `exec.rs`, `main.rs`, and three
TUIs.

★ **SO THE DEPENDENCY IS NOT A LIBRARY -- IT IS 0-CORE'S DIRECTORY LAYOUT.** The prompt shows the
next intent. `digest` reads the ledger. `health_tui` counts active intents. `nl` reads the registry.

⚠️ **WHICH MEANS THE SHELL IS NOT MERELY LOCATED INSIDE 0-CORE, IT IS FUNCTIONALLY PART OF IT.** The
question is not "how do we move a directory" -- it is **what is fsh without 0-Core?**

## THE RULING (2026-08-23)
**STANDALONE fsh MUST WORK WITHOUT 0-CORE. 0-Core integration MAY make fsh richer, but must not
define whether fsh is a viable shell.**

⚠️ TWO ALTERNATIVES WERE CONSIDERED AND REJECTED, with reasons:
- **VENDOR `paths` into fsh** -- rejected. It creates TWO AUTHORITIES over one layout, and they
  drift. This ledger has removed that shape repeatedly: two alias expanders, three selector call
  sites, five observability instruments.
- **A COMPILE-TIME FEATURE FLAG** -- rejected. It PRESERVES the coupling while making the build
  matrix worse, and sprinkles 0-Core awareness through the shell as `cfg`. ⭐ A RUNTIME INTEGRATION
  ADAPTER is architecturally cleaner, and nothing yet justifies compiling two different products.

★ AND THE FRAMING IS PART OF THE RULING. "Extract fsh from 0-Core" is an implementation
prescription. The boundary is the intent:

    fsh
     |-- shell / runtime / history / completion / prompt / jobs / diagnostics
     |
     `-- integration
          `-- faelight-core / 0-Core

The 68 `paths::` calls are EVIDENCE FOR WHERE THAT ADAPTER MUST EMERGE, not a task list.

## ⚠️ DO NOT SOLVE 68 SITES IN ONE PASS
Classify them by CAPABILITY first, because the classification is the finding:

    core shell state          what fsh genuinely needs to operate at all
    0-Core discovery          intents, tools, registry
    0-Core observability      health, digest, changelog
    0-Core UI enrichment      prompt and TUIs showing ecosystem state
    0-Core execution          exec.rs and related behaviour

⭐ THE EXPECTATION, stated so it can be WRONG: surprisingly little of the 68 will turn out to be
fundamental to being a shell. Every census this week shrank on reading -- 42 portability sites
became 2 real defects, 23 capability sites became 5 questions, 14 service probes became 4.

## ⚠️ THIS DOES NOT GATE THE VM WORK, and that is a deliberate sequencing decision
VM validation proceeds against the CURRENT MONOREPO LAYOUT. A VM clones 0-Core whole and builds fsh
in place.

★ THE REASON IS DIAGNOSTIC, NOT CONVENIENCE. Doing both at once combines two experiments --
"does fsh work in a VM?" and "can fsh survive without 0-Core?" -- and if the VM breaks, nothing says
which boundary failed. Prove the shell runs as part of the existing checkout FIRST. Standalone
ownership is a separate architectural migration.

## ⭐ WHY THIS MATTERS: PACKAGING IS DOWNSTREAM OF THIS BOUNDARY (his plan, 2026-08-23)
He arrived at the destination independently and it sharpens the intent: fsh should be installed the
way bash is -- a versioned package, core files under system paths, user customisation confined to
`~/.config/`. His words: *"treat it like real software from the start: package it, version it, own
the core, and give users clean extension points instead of the source itself."*

    void   ->  xbps template, xbps-install fsh
    arch   ->  PKGBUILD / AUR, or a small personal repo
    both   ->  /usr/bin/fsh + /usr/share/fsh, root-owned; user edits ~/.config/fsh only
    always ->  tagged releases (v0.1.0), never a living checkout of main

⭐⭐ AND MOST OF IT IS ALREADY TRUE, which is worth recording so nobody rebuilds it:
- **The user side EXISTS.** `config.rs:81` reads `~/.config/faelight-shell/config.fsh`, and there is
  already `~/.config/faelight-shell/plugins/` (`mod.rs:7014`) and `.../scripts/` (`mod.rs:16079`).
  Config, plugins and scripts -- all three extension points, in the XDG location, today.
- **The lock is STRONGER than the plan assumes.** fsh is a Rust binary, and on NixOS the deployed
  config is a home-manager symlink into `/nix/store`, which `config.rs:67` already records as
  READ-ONLY. Not root-owned-and-editable -- immutable.
- **Versioning EXISTS.** Semver, `cicomplete` prompting for bumps, `bump-versions`, and INT-102 owns
  the tool-version-versus-forest-release architecture. fsh is at 3.8.4.

⚠️⚠️ **SO THE ONLY THING BLOCKING HIS PLAN IS THIS INTENT.** A PKGBUILD that installs
`/usr/bin/fsh` produces a binary that expects `~/0-core/faelight/intents/` to exist, because 68
sites read 0-Core's layout. **You cannot package what cannot install alone.** Packaging is not a
parallel workstream -- it is the thing this boundary unlocks.

★ AND THE SOCIAL HALF IS RIGHT AND COSTS NOTHING TO ADOPT EARLY: clear ownership, an official
install method, versioned releases, and the expectation that users report bugs rather than editing
an installed copy. That part is documentation, and it can be written the day the boundary lands.

## Success Criteria
- [ ] G1 THE 68 SITES ARE CLASSIFIED BY CAPABILITY, as a committed artifact produced mechanically so
      it can be re-run and diffed. The classification decides the scope; it is not decoration
- [ ] G2 THE ADAPTER BOUNDARY IS NAMED: one place where fsh asks about 0-Core, and everything else
      reads the answer. No second authority over the layout -- `paths` is not copied
- [ ] G3 A 0-CORE-ABSENT fsh STARTS, ACCEPTS INPUT, AND RUNS A COMMAND. Demonstrated, not argued
- [ ] G4 EACH 0-CORE FEATURE DEGRADES VISIBLY WHEN 0-CORE IS ABSENT, per INT-227's invariant: an
      unavailable capability must never become a successful-looking empty result. A prompt with no
      intent to show says nothing, it does not show a wrong one
- [ ] G5 RUNTIME PROOF: a test runs fsh with the 0-Core paths absent or redirected, and asserts the
      shell works and the integrations report their absence. ⚠️ NOT a source-text check
- [ ] G6 NO COMPILE-TIME FEATURE FLAG unless a concrete reason to compile two products is recorded
      here first
- [ ] G7 each gate carries evidence per INT-158

## Non-goals
- Moving the repository. ⚠️ THAT IS THE VERSION THIS INTENT REPLACES. A git remote is trivial; the
  coupling is the work, and moving the code without the boundary would produce a repo that still
  cannot build alone.
- `faelight-git`. It is a second path dependency and a smaller question, asked separately once the
  `faelight-core` boundary exists.
- Removing 0-Core features. The machine that has 0-Core should get the richer shell.
- Blocking VM validation. Explicitly out -- see above.
