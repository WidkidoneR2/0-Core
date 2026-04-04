---
id: 187
date: 2026-04-03
type: arch
title: "Delegation Engine — Trust Contracts and Safe Autonomy Simulation"
status: in-progress
tags: [delegation, trust, autonomy, simulation, contracts, v13, v14]
priority: high
depends_on: [156, 185, 186]
---

## The Core Tension
"Nothing runs without explicit human authorization" conflicts with v13 Autonomy.
This intent resolves that conflict through controlled delegation.
Not full autonomy. Not no autonomy. Defined autonomy.

## The Hard Truth
Delegation only works after judgment is proven reliable.
Do not build full delegation until:
  1. Confidence scoring is live (INT-186)
  2. contextd observes real patterns (INT-185)
  3. 30+ days of fsh daily driver usage (INT-179)

Build delegation simulation first. Earn trust through accuracy. Then activate.

## Delegation Simulation (Phase 1 — build this first)
core delegate simulate "restart faelight-notify"

Output:
  Action: restart faelight-notify service
  Would execute? YES
  Confidence: 0.72 | Risk: MEDIUM
  Reason: similar past fixes worked, failure pattern matches
  Rollback: systemctl --user start faelight-notify
  Outcome: NOT EXECUTED (simulation only)

Run simulations for 14+ days. Measure accuracy.
Only activate real delegation after simulation accuracy >= 85%.

## Trust Contract Schema
Each delegatable action type has a contract:
  action_type:     "auto-checkpoint"
  risk_level:      LOW | MEDIUM | HIGH | CRITICAL
  confidence_gate: minimum confidence to auto-execute (e.g. 0.85)
  requires_rollback: true/false
  rollback_action: what to do if wrong
  max_frequency:   how often can this fire (e.g. once per session)
  human_notify:    always | on-failure | never

## Risk Thresholds
LOW:      auto-execute if confidence >= 0.85
MEDIUM:   propose + confirm if confidence >= 0.75, else alert
HIGH:     always propose, never auto-execute
CRITICAL: always alert, human must act

## Hard Boundaries (never crossed — ever)
- Never commit without human confirmation
- Never delete files
- Never modify core config without unlock
- Never act outside defined contract
- Never bypass human on destructive actions
- Never execute if integrity < 95%

## Rollback Guarantee
Every auto-executed action must have a typed rollback:
  pub enum RollbackAction {
      RestartService { name: String },
      RestoreFile    { path: PathBuf, backup: PathBuf },
      RevertDb       { checkpoint: String },
      RunCommand     { cmd: String },
  }

If rollback fails -> immediate alert, suspend all delegation.

## Commands
core delegate simulate <action>   — test without executing
core delegate contracts           — list all trust contracts
core delegate history             — what has been delegated
core delegate accuracy            — simulation accuracy over time
core delegate activate <contract> — enable real delegation (after 85% sim accuracy)
core delegate suspend             — pause all delegation instantly

## Activation Gate
Delegation is NOT activated until:
  simulation_accuracy >= 85% over 14+ days
  confidence_system live (INT-186)
  contextd observing (INT-185)
  integrity >= 95%
  human explicitly runs: core delegate activate <contract>

## Why Simulation First
You are building a system that acts on your behalf.
Before it acts, it must prove it knows when it is right.
Simulation = proof without risk.

## Gate Check
✅ core delegate simulate live and accurate
✅ Trust contract schema defined for each action type
✅ Risk threshold system enforced
✅ Hard boundaries encoded and tested
✅ Typed rollback for every auto-executable action
⬜ Simulation accuracy tracked over 14+ days
⬜ Activation gate enforced (85% accuracy before real delegation)
✅ core delegate contracts/history/accuracy/suspend live

## The Phrase
"Delegation is not permission.
It is earned trust, precisely scoped,
with a guaranteed way back.
The forest acts because you said it could —
and stops the moment you say otherwise." 🌲
