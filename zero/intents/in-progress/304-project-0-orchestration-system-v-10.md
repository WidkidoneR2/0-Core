---
id: 304
date: 2026-11-07
type: future
title: "Project 0 Orchestration System v 1.0"
status: in-progress
tags: [job, handoff, novashell, devbox, omarchy]
---

## Vision
One written way for Christian, Jarvis, Claude and any later agent to hand work
to each other. Christian talks to one front door (the Conductor); each agent has
one job; handoffs are dated files in the repo; the intent ledger stays the only
source of truth. v1 is proven by hand before anything is automated.

## The Problem
Work moves by copy and paste between Christian, Jarvis and Claude. Long pastes
arrive empty or cut off, approvals and open items live only in chat, and chat
is lost when a session ends. Nobody can see at a glance who is working and who
is waiting on Christian.

## The Solution (v1 only)
1. zero/orchestration/PLAYBOOK.md: philosophy, roles and the 12 house rules.
   Roles for v1: Conductor = Jarvis (no new bot), coding = Cursor cloud
   agents, Claude = backup. This supersedes the filed line "NovaShell / coding
   = Claude". On 2026-11-09 Christian decided that Jarvis and Cursor have
   point and Claude takes the backseat. Ledger, DevBox, Omarchy and Hub roles
   are written down but not staffed in v1. The playbook is written from this
   intent alone.
2. Handoff schema in PLAYBOOK.md. The fields are Agent, Status, Intent, Owner,
   NeedsHuman, NeedsHumanReason, Summary, Details, Next, Files, Links. Files
   live at zero/handoffs/YYYY-MM-DD-<agent>.md. Single writer per file.
3. Ledger rule: agents propose, Christian applies. Agents only propose
   ProposedStatus; Christian applies.
4. Approval phrases: yes, no, defer, split this, ledger update.
5. The first-week loop, run by hand: one Conductor handoff routing a real
   intent, one specialist reply, one blended digest.

## Rules
- Nothing is applied without Christian's explicit approval. No sudo.
- Every file lands through a reviewed fpatch Plan with a Seal he has seen.
- v1 adds no tool, no download, no daemon and nothing that runs unattended.
- Existing intents keep their format; core intent validate stays green.
- Every handoff names its intent (INT-nnn) or says "no INT yet, propose one".

## Out of scope (v1, each its own later intent)
- A doctor check, and nsh status / nsh conductor.
- Infomarchy or any Omarchy plugin install.
- Any agent writing handoffs on its own; scheduling.
- Herdr (kept under INT-303 G9 rules); a direct Jarvis to Claude link.

## Success Criteria
- [ ] G0 The recon in The Problem is read against the tree and either stands or
      is corrected here before any other gate. Proof: a dated line in this file
      saying the recon stands, or the correction.
- [ ] G1 PLAYBOOK.md written at zero/orchestration/PLAYBOOK.md with philosophy,
      v1 roles table and the 12 house rules. Proof: the Plan Seal and commit.
RULED 2026-11-10 (Christian): roles for v1 are Conductor = Jarvis, coding = Cursor cloud agents, Claude = backup. This supersedes the filed "NovaShell / coding = Claude". On 2026-11-09 Christian decided that Jarvis and Cursor have point and Claude takes the backseat.
- [ ] G2 Handoff schema, ledger rules and approval phrases written in
      PLAYBOOK.md, plus zero/handoffs/README.md naming the path and file rules.
      Proof: the Plan Seal and commit.
RULED 2026-11-10 (Christian): there is no external spec. The playbook is written from this intent alone: the house rules, the handoff schema fields listed in The Solution, the ledger rule that agents propose and Christian applies, the approval phrases, and the first-week loop.
- [ ] G3 One valid example handoff in zero/handoffs/ with every required field.
      Proof: the file and a field check by hand.
- [ ] G4 First-week loop by hand: a Conductor handoff routing INT-311, the
      specialist reply, and a Zero Digest that blends both with the ledger as
      a story, not a list. Proof: the three files and commit.
RULED 2026-11-10 (Christian): the G4 loop routes INT-311. The digest is named Zero Digest.
- [ ] G5 Christian reviews the playbook, schema and loop files and rules keep,
      change or stop, as a RULED line with the date. Proof: the RULED line.
- [ ] G6 Doors after any ship: nsh-test all passing, d 0 failed, core intent
      validate all valid, git status clean. Every INT-304 commit carries
      Intent: and Seal:, with the note under refs/notes/seals, and core intent
      seals check on the range reads intact, then push. These doors are run by
      hand until INT-311 lands. Then cicomplete 304.
      Proof: the door output and the close commit.
