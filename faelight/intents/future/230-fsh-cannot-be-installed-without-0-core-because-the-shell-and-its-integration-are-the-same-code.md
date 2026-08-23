---
id: 230
title: "fsh cannot be installed without 0-Core, because the shell and its integration are the same code"
status: planned
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
