---
id: 164
date: 2026-07-16
type: fix
title: "SSH: PasswordAuthentication=true while the dashboard claims hardened"
status: complete
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
- [x] DECIDE: key-only, or an honest label. Record which and why
<!-- evidence: NEITHER. The measurement offered a third answer and it is better than both.
     commit 6c2ecb26 -> services.openssh.enable = false.
     WHY: `Accepted` appears ZERO times in the ENTIRE sshd journal -- 1667 lines, not a 30-day
     window. sshd ran for months and never had one conversation. fail2ban: Total failed 0, banned 0
     (an internet-reachable host collects thousands of failed bot attempts PER DAY; zero-ever proves
     port 22 was never reachable from outside the house). ss -tnp :22 empty, who empty, `last` shows
     only reboots.
     WHY IT EXISTED: git log -S found 7300ace1 "INT-328: vm user + ssh + git" -- Christian's own read
     ("i think this has to do with the first build of the vm") was right. But `vm ssh` reaches INTO
     the guest and needs the CLIENT. The daemon never did the work.
     Hardening it would mean maintaining a door into a room nobody enters. INT-143's lesson applied
     to a service: the cure is deletion. -->
- [x] If key-only: `ssh christian@localhost` with the key WORKS before password auth is disabled.
      Do not lock yourself out of your own box proving a point
<!-- evidence: THIS GATE FIRED AND IT SAVED THE INTENT FROM ITS OWN TEXT.
     The Problem section above says "id_ed25519 EXISTS -- key auth is available". IT IS NOT.
       ls ~/.ssh/  -> id_ed25519, id_ed25519.pub, known_hosts, known_hosts.old   NO authorized_keys
       ssh -o BatchMode=yes christian@localhost
         -> Permission denied (publickey,password,keyboard-interactive)
     The key is an OUTBOUND identity -- known_hosts proves outbound use. Nothing was ever authorised
     to come IN. Had PasswordAuthentication=false landed on the strength of "the key exists,"
     Christian would have had NO ssh at all -- and the dashboard would have gone right on saying
     "hardened."
     BatchMode=yes is what caught it: it REFUSES password fallback, so success would have meant the
     key works alone. It failed instead. That is the second time in two intents that the intent's own
     text was wrong and the measurement corrected it (INT-143's `time` and bare-python3 were the
     first two). -->
- [x] If key-only: PasswordAuthentication = false, and the SafeShell/TTY path confirmed still
      reachable (INT-056 -- physical console must never depend on sshd)
<!-- evidence: N/A BY A BETTER ROUTE, and the gate's WORRY was measured rather than dismissed.
     Not key-only -- sshd is gone entirely, so PasswordAuthentication no longer exists to set.
     THE GATE'S REAL FEAR -- that the console might depend on sshd -- was checked and is unfounded:
       getent (INT-143 session) proved the login shell is BASH, not fsh, and greetd owns the console.
       SafeShell is faelight.desktop.greetd.safeShell in modules/desktop/greetd.nix -- a different
       module, untouched by this commit. The comment directly above the openssh block says so.
       systemctl is-active greetd -> active, across every deploy tonight.
     AND THE ACTIVATION LOG IS THE RECEIPT that nothing else went with sshd:
       stopping sshd.service / removing group sshd / removing user sshd
       removing /etc/ssh/sshd_config / removing /etc/pam.d/sshd
     PAM's sshd stack went. PAM's login and sudo stacks did not. `sudo -n true` -> SUDO_OK, and
     `nixos-rebuild build` proved sudo survives (sudo-1.9.17p2, a different package entirely, never
     in scope) BEFORE anything switched. -->
- [x] Read what the health check ACTUALLY asserts about "SSH hardened". If it does not read
      PasswordAuthentication at all, that is a second finding and it goes in this intent
<!-- evidence: THE GATE PREDICTED ITS OWN SECOND FINDING, AND THERE WERE THREE.
     checks.rs:533 read the RIGHT settings and got the LOGIC backwards:
         c.contains("PermitRootLogin no") || c.contains("PasswordAuthentication no")
                 TRUE                     ||            false                = TRUE
     ONE `||` where an `&&` belongs. "Hardened" meant "at least one of the two hardening options is
     on." Worse than the other exhibits because it LOOKS rigorous -- it names real settings.
     TWO MORE in the same 20 lines:
       - fail2ban used `systemctl is-active`, but IS-ACTIVE IS NOT IS-PROTECTING. With sshd gone,
         `fail2ban-client status` -> Number of jail: 0, and the dashboard still said "fail2ban OK".
       - `status: if details > 0 { Pass }` -- ANY ONE of firewall/fail2ban/SSH passed the WHOLE check.
       - (latent) contains() matches a COMMENTED line. Not live: NixOS generates the full effective
         config, 20 lines, no comments -- measured against gen 392's real file.
     ALL FIXED (this commit): directive-level parsing not substring; BOTH password doors, because
     `sshd -T` showed kbdinteractiveauthentication yes + usepam yes means PasswordAuthentication=no
     ALONE would not have closed password login -- the classic half-fix; Pass is a real conjunction;
     fail2ban is a FACT ("fail2ban running") never a tick; and an ABSENT directive reads as OPEN,
     because OpenSSH defaults both password doors to yes and a config read cannot prove a negative.
     Fail safe, not fail flattering. -->
- [x] Whichever path: the dashboard's claim and the config agree. Verified by reading both
<!-- evidence: they agree, and the first half happened WITHOUT anyone editing the check:
       BEFORE:  Security: Firewall OK   fail2ban OK   SSH hardened OK
       AFTER sshd+fail2ban removal (gen 393):  Security: Firewall OK
       AFTER the check rewrite (gen 395, DEPLOYED binary, not target/debug per INT-110):
                Security: Firewall OK   sshd off        -> Status::Pass
     The false claims vanished because the things they lied about vanished -- ssh_ok begins with
     sshd.exists(), and /etc/ssh/sshd_config is gone.
     8/8 tests, and TWO OF THEM ARE THE ONLY ONES THAT COUNT:
       the_old_check_called_gen_392_hardened_and_it_was_wrong ... ok
       the_old_check_would_pass_a_config_with_nothing_set     ... ok
     Six tests prove the NEW function right; on their own they do not prove the OLD one was wrong,
     because the old one is gone. So the old logic is preserved verbatim in the test module and
     ASSERTED WRONG against gen 392's real config, read out of the store at
     /nix/var/nix/profiles/system-392-link/etc/ssh/sshd_config. 124 generations, and one of them is
     now evidence.
     Reintroduce the `||` and a test fails, naming the exact file it lied about. -->
- [x] framework16 is a critical-tier dir (INT-112) -- risk-gate will run framework16-boot on this
      commit. Let it
<!-- evidence: ripsecrets Passed, risk-gate Passed, rustfmt Passed on 6c2ecb26 and on this commit.
     And the change was proven with `nixos-rebuild build` BEFORE any switch -- build, not switch: no
     generation, no /boot, nothing signed, nothing activated. The three questions answered before
     risk existed: ssh client survives (same store path -- ssh and sshd ship in ONE package, which is
     why `ls result/sw/bin/sshd` was a badly-designed test and the UNIT tree was the right one),
     sudo survives, sshd.service + sshd-keygen.service vanish (0 in the new tree, 2 in the current).
     `systemctl is-enabled sshd` -> not-found. Stronger than "disabled": there is no unit left to
     start by hand. -->

## What this intent found that it was not filed for
FILED AS: "PasswordAuthentication=true while the dashboard claims hardened." That was the SMALLEST
thing wrong.

**THE QUESTION THAT MATTERED, asked directly: has someone been getting info from my laptop? Is my
login compromised?**
NO. Measured, not inferred. `Accepted` in the ENTIRE sshd journal: ZERO. Not zero in 30 days -- zero
ever. The only three connections ever logged: 192.168.1.1 twice (his own router probing the LAN,
never authenticated) and ::1 once (our own BatchMode test at 23:46, which failed publickey and got
rate-limited by sshd's own srclimit_penalise -- the defenses working). fail2ban: 0 failed, 0 banned.
`last` shows only reboots.
The assistant's framing caused that fear and the framing was wrong: the finding was that sshd is
USELESS, not that it was ABUSED. A door standing open in a room nobody has ever walked into.

**THE COMPLETE LIST OF LISTENING SOCKETS** (ss -tlnp minus loopback, 2026-07-17):
    atticd    127.0.0.1:8080       loopback -- this machine only
    dnsmasq   192.168.122.1:53     libvirt virbr0 -- VM guests only
That is all of it. ZERO network-reachable services. fail2ban was not guarding a house with no doors;
it was guarding an empty lot -- which is why it went too.

**THREE DELETIONS, ONE CURE, ONE NIGHT**: python3's dispatch arm (INT-143), sshd, fail2ban. Each
improved nothing and could only break. The strongest control is the one you deleted.

**AND THE PART THAT OUTLIVES THE DECISION**: fixing the check was worth more than fixing the config.
One closes a door. The other stops the dashboard lying to you about doors -- including doors you open
later for real reasons.

