---
id: 165
date: 2026-07-16
type: feature
title: "AppArmor: confinement layer, and how it relates to faelight-sandbox"
status: planned
tags: [feature, rust, faelight]
---

## Vision
A confinement layer -- IF it adds something faelight-sandbox does not already provide.

## The Problem -- and the first gate is whether there IS one
AppArmor confines applications by profile: a compromised app cannot read what its profile does
not allow. Real defence in depth, easy to enable on NixOS (security.apparmor.enable).

BUT: this machine ALREADY HAS faelight-sandbox, with FIVE ACTIVE POLICIES (health dashboard,
2026-07-16: "faelight-sandbox deployed -- 5 policies active"). Its policies live in
faelight/registry/sandbox-policies.toml (INT-061's second-wave sweep repointed the loader there
after it had been reading a dead Arch-era path since the migration -- it was BROKEN and nobody
noticed, which is worth remembering before adding a second confinement system).

So the honest first question is not "how do we add AppArmor" but "WHAT DOES APPARMOR CONFINE
THAT FAELIGHT-SANDBOX DOES NOT?"
Plausible answer: they are different layers. faelight-sandbox governs what FOREST TOOLS may do
(a policy engine for this project's own code); AppArmor governs what ANY binary may do at the
kernel level, including things we did not write -- browsers, mullvad, virt-manager, fwupd.
If that is the answer, both are justified and the boundary should be written down.
If the answer is "nothing meaningful", CANCEL THIS INTENT. That is a legitimate outcome and a
better one than a second confinement system nobody maintains. INT-110 was cancelled for less.

## The Solution
Decide the boundary first. Then, if justified: security.apparmor.enable, start with profiles
that ship in nixpkgs, add our own only where measured.

## Success Criteria
- [ ] FIRST: what does AppArmor confine that faelight-sandbox does not? Written down, with the
      boundary between the two stated plainly. If the answer is "nothing", cancel and say so
- [ ] Which binaries actually need confining, named, with a reason each. Not "enable AppArmor"
      -- a profile per named binary
- [ ] Enabled in the VM FIRST. AppArmor denials are silent by design; a bad profile breaks an
      app in a way that looks like a bug, not a policy. The VM is the proving ground (INT-112:
      breaking it costs a test run, not a laptop)
- [ ] Complain-mode before enforce-mode, with the denial log read. Do not enforce a profile
      whose denials you have not looked at
- [ ] greetd/mango/fsh confirmed unaffected -- a confinement profile that breaks login is a
      lockout, and login is critical tier (INT-112)
- [ ] The health dashboard's Security section reports AppArmor honestly -- it must read real
      state, not the fact that we enabled an option. See INT-164 for why that distinction
      matters
