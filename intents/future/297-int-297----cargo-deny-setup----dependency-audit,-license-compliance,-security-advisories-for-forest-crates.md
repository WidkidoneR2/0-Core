---
id: 297
title: "cargo-deny setup -- dependency audit, license compliance, security advisories"
status: in-progress
date: 2026-05-12
tags: [cargo-deny, security, audit, license, dependencies, rust, forest]
---

The forest has a public repo watched by serious people.
Every dependency is a trust decision.
cargo-deny enforces that trust programmatically.

The deny.toml file exists but is empty.
This intent fills it with meaning.

---

WHY NOW

faelight-term v3 added significant new dependencies:
  wgpu 24 -- GPU rendering
  alacritty_terminal 0.24 -- PTY + grid
  cosmic-text 0.12 -- text shaping
  glyphon 0.8 -- wgpu text rendering
  wl-clipboard-rs 0.9 -- clipboard

Each of these pulls in dozens of transitive dependencies.
Without cargo-deny, a vulnerable or license-incompatible
dependency could silently enter the forest.

---

WHAT CARGO-DENY DOES

Four checkers, all configured in deny.toml:

1. advisories -- checks RustSec advisory database
   Alerts when any dependency has a known CVE or vulnerability
   Forest should have zero known vulnerabilities

2. licenses -- enforces license policy
   Forest policy: MIT and Apache-2.0 only
   No GPL, no proprietary, no unknown licenses
   Exceptions documented explicitly

3. bans -- prevents duplicate or banned crates
   No two versions of the same crate
   Explicit deny list for crates we never want

4. sources -- controls where crates come from
   Only crates.io allowed (no git deps, no local patches in prod)
   Exceptions documented explicitly

---

INSTALLATION

cargo install cargo-deny --locked

Add to CI/pre-push workflow:
  cargo deny check

---

DENY.TOML CONFIGURATION

Forest-appropriate settings:

[advisories]
  db-path = "~/.cargo/advisory-db"
  db-urls = ["https://github.com/rustsec/advisory-db"]
  vulnerability = "deny"
  unmaintained = "warn"
  unsound = "deny"
  yanked = "warn"

[licenses]
  unlicensed = "deny"
  allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception",
           "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-DFS-2016",
           "CC0-1.0", "Zlib"]
  deny = ["GPL-2.0", "GPL-3.0", "AGPL-3.0", "LGPL-2.0", "LGPL-3.0"]
  copyleft = "warn"

[bans]
  multiple-versions = "warn"
  wildcards = "deny"
  highlight = "all"

[sources]
  unknown-registry = "deny"
  unknown-git = "deny"
  allow-registry = ["https://github.com/rust-lang/crates.io-index"]

---

INTEGRATION WITH FOREST WORKFLOW

Add to pre-push hook (scripts/pre-push):
  cargo deny check advisories
  cargo deny check licenses

Add to doctor checks:
  cargo deny check --quiet advisories 2>/dev/null
  Reports: "X known vulnerabilities" or "All dependencies clean"

Add to faelight-release:
  Run cargo deny check before publishing
  Block release if any critical advisories exist

---

GATES

[ ] cargo-deny installed (cargo deny --version works)
[ ] deny.toml filled with forest-appropriate configuration
[ ] cargo deny check passes with zero errors
[ ] cargo deny check added to pre-push hook
[ ] doctor check includes dependency audit status
[ ] Zero known vulnerabilities in forest dependencies
[ ] All licenses MIT/Apache-2.0 compatible
[ ] deny.toml committed and documented

---

NOTE ON DENY.TOML IN REPO

The existing empty deny.toml is a workspace-level file.
It applies to ALL crates in the workspace.
This is correct -- one policy for the entire forest.
