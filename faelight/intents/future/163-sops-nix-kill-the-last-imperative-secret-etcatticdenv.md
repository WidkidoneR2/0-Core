---
id: 163
date: 2026-07-16
type: arch
title: "sops-nix: kill the last imperative secret (/etc/atticd.env)"
status: planned
tags: [security, secrets, nix, attic]
---

## Vision
No secret on this machine is hand-written. `/etc/atticd.env` is the last one, and it violates
0-Core's own invariant.

## The Problem -- MEASURED 2026-07-16, not proposed
nix/modules/services/atticd.nix says it in its own comments (lines 9-12):

    # Secret: JWT RS256 signing secret lives in /etc/atticd.env (root-only, NOT in git,
    # ...
    # then write:  ATTIC_SERVER_TOKEN_RS256_SECRET_BASE64=<that-value>  into /etc/atticd.env
      environmentFile = "/etc/atticd.env";

That file was written BY HAND, ONCE, and nothing declares it. INT-061's charter states the
invariant it breaks:

    "Declarative-over-imperative: the OS-level registry is expressed in Nix
     (flake/profiles/modules/hosts); NO IMPERATIVE DRIFT."

/etc/atticd.env IS the imperative drift, sitting in the middle of a declarative system.
Rebuild this laptop from the flake onto a fresh disk and atticd comes up broken -- silently,
because nothing checks it -- until someone remembers a step that exists only in a comment.
The Attic cache is not cosmetic: framework16 pulls its crane dependency closure from
127.0.0.1:8080, and INT-043 replaced Cachix with it precisely because Cachix could not serve
those paths (proven 2026-07-07, 667-path closure).

WHAT IS *NOT* A PROBLEM -- do not solve these, they are already right:
  - users.users.christian has NO password in the repo. configuration.nix:91: "Password set at
    install (passwd) -- never in this public repo." Correct.
  - /var/lib/sbctl/keys/*.key (INT-161 Secure Boot signing keys) are OUTSIDE the repo, backed
    up to FORESTBACKUP with a proven round trip. Correct, and they must STAY out: GitHub is
    public, and publishing db.key means anyone can sign a binary this firmware will boot.
  - The Attic PUBLIC key in configuration.nix is public by definition. Correct.
  - Framework's factory .esl certs in nix/hosts/framework16/secureboot-factory/ are public
    X.509. Correct.
  - ripsecrets passes -- and since INT-119's repair (2026-07-16, d9b9b4d7) it ACTUALLY RUNS on
    every commit. Before that it had never run once.
This intent has exactly ONE target. That is why it is worth doing.

## The Solution
sops-nix or agenix. Encrypt the secret INTO the repo; decrypt at activation. Both are good.
sops-nix is the likelier fit (age backend, and it handles the .env format atticd wants
directly). CHOOSE WITH A REASON WRITTEN DOWN -- do not pick by popularity.

DO NOT file a second intent for agenix. sops-nix vs agenix is a CHOICE INSIDE THIS ONE. Two
intents for one problem is how INT-110/112 happened, and how INT-113/119 shipped the same bug
twice.

## Success Criteria
- [ ] sops-nix vs agenix chosen, with the reason recorded (not "it is popular")
- [ ] The age/GPG identity itself lives OUTSIDE the repo, and its location + backup is written
      down. Whatever decrypts the secrets is now the real secret -- do not solve one imperative
      file by creating another
- [ ] ATTIC_SERVER_TOKEN_RS256_SECRET_BASE64 encrypted in-repo, decrypted at activation
- [ ] atticd.nix's environmentFile points at the sops-managed path; lines 9-12's hand-write
      instructions DELETED, not left to rot
- [ ] THE REAL TEST: a fresh VM built from this flake brings atticd up with ZERO hand-editing.
      Not "it evaluates" -- the service starts and serves. Prove it in the VM, not on metal
- [ ] The existing /etc/atticd.env value is preserved, not regenerated -- server.db (10MB,
      since 2026-07-07) holds tokens signed by the CURRENT key. A new key invalidates them
- [ ] nix flake check green; RISK.toml tier for any new secrets dir decided (INT-112)
