---
id: 045
date: 2026-01-07
type: future
title: "policy-audit - Policy Enforcement Without Automation"
status: cancelled
tags: [rust, intent, policy, enforcement, v5.3]
---

## The Vision

A policy engine that evaluates system state against declared intent. No automatic fixing — just truth.

## Why

Intent is documented beautifully. But documentation doesn't enforce.
intent-guard closes the loop: intent becomes verifiable policy.

## Philosophy

**Intent stays human-authored. Enforcement stays manual.**

This is not automation. This is awareness.

## How It Works

### 1. Intent Constraints

Add optional constraints to intent files:

```yaml
[constraints]
no_auto_updates = true
network = "vpn-only"
filesystem = "immutable"
```

### 2. Policy Evaluation

```bash
intent-guard check
```

Output:

```
🟢 filesystem: immutable (compliant)
🟡 network: vpn-only → currently disconnected (drift)
🔴 auto_updates: disabled → pamac running (violation)
```

### 3. No Auto-Fix

Never fixes automatically. Suggests manual action:

```
Suggested: systemctl --user stop pamac-daemon
```

## Constraint Types

| Constraint        | Checks                           |
| ----------------- | -------------------------------- |
| `filesystem`      | lsattr immutable flag            |
| `network`         | VPN connection status            |
| `no_auto_updates` | Background update services       |
| `profile`         | Current profile matches expected |
| `services`        | Required services running        |

## Success Criteria

- [ ] Parses intent constraint blocks
- [ ] Evaluates system state
- [ ] Reports compliant/drift/violation
- [ ] Never auto-fixes
- [ ] Clear manual action suggestions

---

_Intent declared. Truth revealed. Action remains yours._ 🌲

## Cancellation Reason

dot-doctor already evaluates system state against expected configuration. intent-guard handles policy enforcement at the command level. entropy-check detects drift. Adding a fourth tool for the same purpose violates the single-responsibility principle. Cancelled in favour of consolidation.
