---
id: 063
date: 2026-01-18
type: future
title: ""Formalized Trust Levels - OPEN / LOCKED / SEALED Core States""
status: planned
tags: [[v7.9, security, governance, core, trust]]
---

Vision

Introduce explicit, enforceable trust levels for the 0-core system to clearly define when the system is mutable, protected, or cryptographically sealed.

Faelight Forest should not only look intentional — it should know when it is safe, stable, and authoritative.

This elevates 0-core from “protected dotfiles” to a personal reference operating system.

The Problem

The current protection model is binary:

🔓 unlocked — anything allowed

🔒 locked — filesystem made immutable

This lacks nuance and does not distinguish between:

active development

daily trusted use

known-good, reference states

As the system grows in complexity, this creates ambiguity around:

when changes are acceptable

what state is considered authoritative

how to guarantee integrity over time

The Solution

Introduce three formal trust levels, each with explicit meaning and enforcement:

Level	Meaning	Enforcement
🔓 OPEN	Active development	No restrictions
🔒 LOCKED	Daily trusted use	No edits, no installs
🧊 SEALED	Known-good reference state	Hash-verified, mutation denied
🔓 OPEN

Used during active development

Editing configs, installing packages, refactoring tools

No restrictions enforced

🔒 LOCKED

Default daily-driver state

~/0-core immutable (chattr +i)

Git commits and package installs blocked

Visible status in:

Starship prompt

faelight-bar

🧊 SEALED

A cryptographically verifiable, known-good state.

Planned enforcement:

Hash tree of ~/0-core

Snapshot of pacman package versions

Kernel + initramfs checksum

Verification on demand (and optionally on boot)

Any mutation denied unless explicitly unsealed

Breaking seal must be:

manual

explicit

logged as an intent or incident

Implementation Ideas

Extend core-protect to support trust tiers

New commands:

seal-core

unseal-core

verify-core

Store seal metadata in typed TOML

Integrate verification into:

dot-doctor

faelight-bar (status indicator)

Optional: fail fast if seal verification fails

Success Criteria

 Three trust levels formally defined and documented

 System can switch between OPEN / LOCKED / SEALED

 SEALED mode prevents all mutation without explicit break

 Seal verification detects any drift

 Trust level visible in prompt and bar

 Seal break is intentional, logged, and reviewable
