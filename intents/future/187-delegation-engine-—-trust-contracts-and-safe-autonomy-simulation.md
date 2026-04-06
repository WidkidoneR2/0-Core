---
id: 187
date: 2026-04-03
type: in-progress
title: "Delegation Engine — Trust Contracts and Safe Autonomy Simulation"
status: in-progress
tags: [delegation, trust, autonomy, simulation, contracts, capabilities, v13, v14]
---
"Nothing runs without explicit human authorization" conflicts with v13 Autonomy.
This intent resolves that conflict through controlled delegation.
Not full autonomy. Not no autonomy. Defined autonomy.
You are not building a delegation engine.
You are building a trust calculus system.
Where confidence, risk, history, and reversibility
combine into a single decision:
"Is the forest earned the right to act here?"
Every piece of this system answers that question differently.
The answer must be yes on all dimensions before anything executes.
Delegation only works after judgment is proven reliable.
Do not build full delegation until:
  1. Confidence scoring is live (INT-186)
  2. contextd observes real patterns (INT-185)
  3. 30+ days of fsh daily driver usage (INT-179)
Build delegation simulation first. Earn trust through accuracy. Then activate.
The original design used a single "accuracy >= 85%" gate.
That is insufficient. A system can hit 85% while still being unsafe.
Example failure: always suggest "restart service" → works often enough → 85%
But misses deeper issues → fragile system in production.
The correct accuracy model has three independent dimensions:
accuracy:
action_match:        0.88
outcome_success:     0.82
calibration_error:   0.06
Activation requires ALL THREE gates:
activation_requirements:
action_match       >= 0.85
outcome_success    >= 0.80
calibration_error  <= 0.10
This is the real safety bar.
After every simulation, log what actually happened:
counterfactual_log:
timestamp:         <unix>
proposed_action:   what the engine suggested
human_action:      what you actually did
match:             true/false
outcome:           what happened after your action
proposed_outcome:  what it predicted would happen
Without this, accuracy is a guess.
With this, accuracy is measured truth.
This data feeds directly into all three accuracy dimensions above.
The original design used string-based action types.
String-based delegation can drift — "restart service" is ambiguous.
Typed capabilities are precise and enumerable:
```rust
pub enum Capability {
    RestartService {
        name: String,
        allowed_services: Vec<String>,
        max_per_hour: u8,
    },
    CreateCheckpoint {
        tag: String,
        max_per_session: u8,
    },
    NotifyUser {
        message: String,
        urgency: Urgency,
    },
    RunDiagnostic {
        check_name: String,
        read_only: bool,
    },
}
```
Delegation is no longer "can it run commands?"
Delegation is "what capabilities exist, and under what constraints?"
This is a capability-based security model.
Every action the forest can take is enumerated.
Nothing outside this list can execute.
The original rollback schema included:
```rust
RunCommand { cmd: String }
```
This is effectively unrestricted shell execution.
It bypasses the entire contract system.
It is eliminated. Replaced by structured rollback types only:
```rust
pub enum RollbackAction {
    RestartService  { name: String },
    RestoreFile     { path: PathBuf, backup: PathBuf },
    RevertDb        { checkpoint: String },
    // RunCommand removed — too dangerous
}
```
If a rollback cannot be expressed as a structured type,
the action cannot be auto-executed.
Original: rollback paths were declared but not tested.
New rule: if rollback is untested, the action cannot auto-execute.
Every rollback path must be exercised in simulation before activation.
The simulation must prove:
  1. The rollback path is reachable
  2. The rollback returns the system to a known good state
  3. The rollback completes within acceptable time
If rollback fails in simulation → action is marked non-delegatable.
If rollback fails in production → suspend all delegation immediately.
Original: hard limits were written as policy.
Policy can be bypassed by bugs or edge cases.
New: hard limits are enforced at the execution layer.
Protected paths (immutable unless explicitly unlocked):
~/0-core/engine/*
~/0-core/scripts/*
~/.config/core/*
~/0-core/runtime/state.db
Blocked operations (never executed by delegation):
Any git commit, push, or destructive git operation
File deletion of any kind
Any operation on protected paths without unlock token
Any operation that raises integrity below 95%
The enforcement happens before capability dispatch.
Intent logic cannot override it.
Only explicit human unlock can override it.
Each delegatable capability has a contract:
capability:        Capability::RestartService { ... }
risk_level:        LOW | MEDIUM | HIGH | CRITICAL
confidence_gate:   minimum confidence to auto-execute
requires_rollback: true (and rollback must be simulation-verified)
rollback_action:   typed RollbackAction (no RunCommand)
max_frequency:     how often can this fire (e.g. once per session)
human_notify:      always | on-failure | never
LOW:      auto-execute if action_match >= 0.85 AND calibration_error <= 0.10
MEDIUM:   propose + confirm if outcome_success >= 0.80, else alert
HIGH:     always propose, never auto-execute
CRITICAL: always alert, human must act
core delegate simulate "restart faelight-notify"
Output:
  Capability: RestartService { name: "faelight-notify", allowed: ["faelight-*"] }
  Would execute?    YES
  Action match:     0.81 | Outcome prediction: 0.76 | Calibration error: 0.08
  Risk:             MEDIUM
  Reasoning:        similar past fixes worked, failure pattern matches
  Rollback:         RestartService { name: "faelight-notify" } [VERIFIED]
  Counterfactual:   logged for accuracy tracking
  Outcome:          NOT EXECUTED (simulation only)
Run simulations for 14+ days. Measure all three accuracy dimensions.
Only activate after ALL THREE gates pass.
core delegate simulate <action>   — test without executing
core delegate contracts           — list all trust contracts
core delegate history             — what has been delegated
core delegate accuracy            — all three accuracy dimensions
core delegate activate <contract> — enable real delegation (after gates pass)
core delegate suspend             — pause all delegation instantly
core delegate counterfactuals     — show ground truth comparison log
✅ core delegate simulate live and accurate
✅ Trust contract schema defined for each action type
✅ Risk threshold system enforced
✅ Hard boundaries encoded and tested
✅ Typed rollback for every auto-executable action
⬜ Simulation accuracy tracked over 14+ days — clock running (started 2026-04-03)
⬜ Activation gate enforced — gate requires action_match >= 0.85, outcome_success >= 0.80, calibration_error <= 0.10
✅ core delegate contracts/history/accuracy/suspend live
⬜ Three-dimensional accuracy implemented (action_match / outcome_success / calibration_error)
⬜ Counterfactual tracking live — human_action vs simulated_action logged
⬜ Typed capabilities replacing string action types
⬜ RunCommand rollback eliminated from schema
⬜ Rollback paths simulation-verified before activation allowed
⬜ Hard boundaries enforced at execution layer (not just policy)
"Delegation is not permission.
It is earned trust, precisely scoped,
with a guaranteed way back.
The forest acts because you said it could —
and stops the moment you say otherwise.
But first: it must prove it knows when it is right.
Not once. Not on average.
On all three dimensions, simultaneously.
Confidence without calibration is arrogance.
Action without outcome is noise.
Trust without proof is hope.
This system earns the right to act." 🌲
