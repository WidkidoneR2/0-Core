---
id: 164
date: 2026-07-16
type: fix
title: "SSH: PasswordAuthentication=true while the dashboard claims hardened"
status: planned
tags: [fix, bugfix]
---

## Vision
The dashboard says "SSH hardened". Make that true, or make the dashboard stop saying it.

## The Problem -- MEASURED 2026-07-16
nix/hosts/framework16/configuration.nix:128-132:

    services.openssh = {
      enable = true;
      settings.PasswordAuthentication = true;    <-- this
      settings.PermitRootLogin = "no";
    };

The health dashboard reports: `Security Hardening   Security: Firewall OK  fail2ban OK  SSH hardened OK`
Port 22 is open (configuration.nix:100). ~/.ssh/id_ed25519 EXISTS -- key auth is available and
password auth is the weaker option that would normally be off once keys work.

HONEST RISK -- MEASURED, NOT DRAMATISED. This is not a fire:
    ss -tlnp            -> sshd listening 0.0.0.0:22 and [::]:22
    fail2ban-client     -> jail sshd active, Total failed: 0, Total banned: 0
ZERO failed attempts, ever. On an internet-exposed host that number is in the thousands within
a day. This machine is LAN-only behind NAT, fail2ban is armed and has had nothing to do, and
root login is already off. The assistant initially called this a "live exposure" -- that was
overstated, and measuring it is what corrected it.

THE BUG IS THE CLAIM, NOT THE PORT. A dashboard that says "hardened" while password auth is on
teaches you to trust a green light that is not checking what its label implies. That is the
same disease as INT-119's "unskippable" hook that was never installed, and the three "mirrors
framework16" comments found false in one evening.

## The Solution
Either turn password auth off (keys work, the key exists), or change what the health check
asserts so the label matches reality. Both are honest. Pick one.

## Success Criteria
- [ ] DECIDE: key-only, or an honest label. Record which and why
- [ ] If key-only: `ssh christian@localhost` with the key WORKS before password auth is
      disabled. Do not lock yourself out of your own box proving a point
- [ ] If key-only: PasswordAuthentication = false, and the SafeShell/TTY path confirmed still
      reachable (INT-056 -- physical console must never depend on sshd)
- [ ] Read what the health check ACTUALLY asserts about "SSH hardened". If it does not read
      PasswordAuthentication at all, that is a second finding and it goes in this intent
- [ ] Whichever path: the dashboard's claim and the config agree. Verified by reading both
- [ ] framework16 is a critical-tier dir (INT-112) -- risk-gate will run framework16-boot on
      this commit. Let it
