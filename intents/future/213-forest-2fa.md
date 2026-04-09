---
id: 213
date: 2026-04-09
type: planned
title: "Forest 2FA — The System Knows Only You"
status: planned
tags: [security, 2fa, totp, authentication, strongbox, hardening]
---
The forest protects your life's work.
But right now, anyone who sits at your keyboard can unlock-core.
2FA changes that: even with physical access, the forest requires
a second factor that only you carry.
Tier 1 — requires 2FA:
  unlock-core           — before any core modification
  git push (to main)    — before any push to the canonical repo
  faelight-release      — before any version release
Tier 2 — optional, configurable:
  deploy <critical-tool> — deploying core, faelight-daemon
  core self apply        — accepting self-transformation proposals
Time-based One-Time Password (RFC 6238).
The same algorithm used by banks, GitHub, Google.
Works completely offline — pure cryptographic math.
How it works:
  1. Forest generates a secret key (stored encrypted in state.db)
  2. Forest displays a QR code — you scan with StrongBox once
  3. StrongBox and the forest now share the secret
  4. Every 30 seconds both generate the same 6-digit code
  5. Before sensitive operations: forest asks for your code
  6. You open StrongBox, type the 6 digits, done
No network required. No external service. No phone dependency after setup.
StrongBox stores it alongside your other credentials — one place, your control.
  core 2fa setup          — generate secret, display QR code for StrongBox
  core 2fa verify <code>  — verify a TOTP code
  core 2fa status         — show 2FA protection status
  core 2fa disable        — remove 2FA (requires current valid code)
unlock-core script becomes:
  1. Check if 2FA is enabled
  2. If yes: prompt for code from StrongBox
  3. Verify code against TOTP algorithm
  4. If valid: unlock. If invalid: deny with reason.
StrongBox supports TOTP natively (it is an TOTP-compatible authenticator).
Setup is one QR code scan.
After that: StrongBox generates the code, you type it.
No app to build, no API to call — just standard TOTP protocol.
  totp-rs = "5"  — pure Rust TOTP implementation, no C deps
⬜ totp-rs added to core dependencies
⬜ core 2fa setup — generates secret, displays QR code
⬜ Secret stored encrypted in state.db (not plaintext)
⬜ core 2fa verify — validates 6-digit TOTP code
⬜ unlock-core checks 2FA before unlocking
⬜ git push hook checks 2FA before push to main
⬜ core 2fa status — shows what is protected
⬜ core 2fa disable — requires valid code to remove
⬜ Backup codes generated at setup (in case phone lost)
⬜ 2FA bypass documented (recovery procedure)
"The forest knows your values.
The forest knows your patterns.
The forest knows your history.
Now the forest knows your face.
Or at least your phone.
Security is not paranoia.
It is respect for what you have built." 🌲
