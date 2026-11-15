---
id: 316
date: 2026-11-15
type: future
title: "Friday knows the machine"
status: planned
tags: [friday, ledger]
format: 2
---

## Vision
Stage 2 of six. She understands Omarchy and Project 0 as a map. Friday stays
a personal assistant with intelligence and no AI: no LLM and no embeddings.
She suggests only. Nothing here runs a command, starts a daemon, or adds a
timer.

## The Problem
Friday's facts are written by hand. `seed_knowledge` and `seed_linux_knowledge`
insert rows with source `seed` and `seed_linux`. A doctor run stores a health
total. It does not write one `friday_knowledge` row per probe, and the table
has no expiry.

`sync_knowledge_meta` labels every fact `system='zero'`. pacman, Hyprland, and
systemd are this machine. A fact or a seed that says NixOS describes a machine
that is not here.

`core friday map` reads the `friday_map` table. It does not build tool, crate,
and intent from the tree, and it does not ask `git log --name-only
--grep=INT-NNN`. `impact` looks that table up by name. It does not name the
intents and tools a file touches.

No reasoning rule reads `/var/log/pacman.log`. A probe that changes after an
upgrade is not cited.

`check_milestones` announces a commit mark as `{count} commits. Past the
{threshold} mark. {message}`, and the 2,500 message itself starts with `2500
commits.` The same number is said twice, with no thousands separator.

## The Solution
Probe facts are built from the doctor's per-probe results, with source
`probe` and an expiry. Nothing in that set is hand-written. pacman, Hyprland,
and systemd are `system='native'`. No Friday fact or seed says NixOS.

`core friday map` builds the tool, crate, and intent graph from the tree and
from `git log --name-only --grep=INT-NNN`. `core friday map impact <file>`
names the intents and tools that file touches. The tree is colored: tools
cyan, crates green, intents magenta, and a ✦ on anything changed in the last
24 hours.

A rule may cite a probe change. Christian ruled yes under D1 on 2026-11-15.
The example is "Reboot Needed went from pass to warn after a pacman -Syu".
The rule reads the read-only, world-readable `/var/log/pacman.log`. The path
is injectable for tests. No root.

The milestone line names each number once: "5,415 commits. Past the 2,500
mark."

## Success Criteria
- [ ] G1 Probe facts. After a doctor run, friday_knowledge gets rows with
      source='probe' and an expiry, built from the doctor's per-probe
      results. No hand-written facts.
      ask:
      ruling:
      evidence:
- [ ] G2 Native labels. pacman, Hyprland, and systemd are system='native'.
      No Friday fact or seed says NixOS.
      ask:
      ruling:
      evidence:
- [ ] G3 Map. `core friday map` builds the tool, crate, and intent graph
      from the tree and from `git log --name-only --grep=INT-NNN`.
      `core friday map impact <file>` names the intents and tools that file
      touches. The tree is colored: tools cyan, crates green, intents
      magenta, and a ✦ on anything changed in the last 24 hours.
      ask:
      ruling:
      evidence:
- [ ] G4 Probe-citing rule. Christian ruled yes under D1 on 2026-11-15. A
      rule may cite a probe change, for example "Reboot Needed went from
      pass to warn after a pacman -Syu", reading the read-only,
      world-readable /var/log/pacman.log. The path is injectable for tests.
      No root.
      ask:
      ruling:
      evidence:
- [ ] G5 Milestone wording names each number once, for example "5,415
      commits. Past the 2,500 mark."
      ask:
      ruling:
      evidence:
- [ ] G6 Doors. On the laptop, after ship: cargo test -p core, cargo test
      -p novashell, nsh-test, and core intent validate.
      ask:
      ruling:
      evidence:

## Out of scope
- Stages 3 through 6.
- An LLM call or an embedding.
- A daemon, a socket, or a timer.
- Root, sudo, or writing the pacman log.
- Rewriting the doctor's probes.

## Versions
- none. The red tests do not change a crate version.
