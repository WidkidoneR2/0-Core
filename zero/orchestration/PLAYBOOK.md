# Project 0 orchestration

Christian decides everything and approves everything. Daily work is
copy-and-paste between Christian and Eric, so Eric sees every step.
Eric talks with Cursor. The intent ledger is the only memory that
survives the chat.

This playbook replaces the INT-304 staffing that named Jarvis as
conductor and Claude as backup. Claude was cancelled as a working
agent on 2026-11-22. There is no orchestration daemon: no tool, no
download, and nothing that runs unattended.

## How we work

The shape is a triangle. Eric sits between Christian and Cursor.
Christian does not talk to Cursor directly. Together they work the
problem and test for failures and passing results. The full rules
are in AGENTS.md. This file states the same roles and the written
handoff format.

| Who | Job |
|---|---|
| Christian | Decides and approves everything. He can override any rule. He runs every command on his laptop, including doors, nsh-test, d, and ship. No gate passes without output from his laptop. He merges onto main and seals. |
| Eric | Grok Bot, Director of Operations. The link between Christian and Cursor. Talks with Christian directly and through copy-and-paste: Eric hands him paste-ready command blocks, and Christian pastes the output back. Reviews. A reviewer may flag or block a proposal until the flag is answered, and never approves. |
| Cursor cloud agents | Do most of the coding, on branches. They do not push to main. Eric reviews the branch; Christian merges and seals. |

## House rules

1. Nothing is applied without Christian's explicit approval of that exact
   change. (AGENTS.md, How we work.)
2. An agent does not touch sudo, sudoers, or a password prompt, and does not
   work around them. A step that needs root is written out for Christian to
   run, with the reason it needs privilege. (AGENTS.md, Sudo.)
3. An agent stops before the boot chain, disk layout, LUKS, or the greeter,
   and before a broad process kill. (AGENTS.md, Stop.)
4. Agents never push to main. They may push a branch. Eric reviews the
   branch; Christian merges onto main and seals. Christian runs every
   command on his laptop. (AGENTS.md, How we work, and Version control.)
5. Secrets are never committed. Secrets and confidential information never
   leave this machine to the internet or to agents. A missing scanner fails
   the commit, and a skipped hook is stated in the next commit body.
   (AGENTS.md, Security, and Version control.)
6. A change to AGENTS.md or this playbook is a proposal until Christian
   asks for that edit. (AGENTS.md, Edit.)
7. Every ship is followed by a reload, and the deployed artifact is what
   gets checked. A green build is not deployed. (AGENTS.md, Intent
   lifecycle, and Build and test.)
8. cistart opens the intent before work of consequence. cicomplete closes
   it only once the evidence exists, never to tidy the end of a session.
   (AGENTS.md, Intent lifecycle.)
9. Every commit that carries Intent: also carries Seal:, and the plan text
   is kept as a note under refs/notes/seals with
   `git notes --ref=seals add --no-stripspace -F`. The flag keeps the bytes.
   (AGENTS.md, Evidence. The command form is the one INT-306 ruled.)
10. A commit message carries no Co-authored-by trailer. This is not a
    sentence in AGENTS.md. It is the rule the sealed commits already follow.

## Ledger

Agents propose. Christian applies.

An agent may write a ProposedStatus. That is a proposal. Christian applies
the status on the intent. An agent does not move, tick, or close an intent
unless he has approved that exact change.

## Handoffs

Daily work is copy-and-paste. When a written handoff is used, it is one
dated file in zero/handoffs/, named YYYY-MM-DD-<agent>.md. Nothing in that
directory runs on its own: no timer, no daemon, and no tool reads it by
itself. (zero/handoffs/README.md.)

A handoff is one file. The writer fills every field.

- Agent. Who wrote this file. eric, cursor, or christian.
- Status. open (the owner has not answered), answered (a reply file
  exists), or closed (Christian has approved it).
- Intent. The ledger item. INT-nnn, or the exact words "no INT yet, propose one".
- Owner. Who must act next. eric, cursor, or christian.
- NeedsHuman. Whether Christian has to act before anyone else moves. yes or no.
- NeedsHumanReason. Why, when NeedsHuman is yes. The word none when it is no.
- Summary. What this handoff is, in one or two sentences.
- Details. The rest, in plain prose.
- Next. The single next action.
- Files. Repo paths this handoff is about, or the word none.
- Links. Other handoffs, intent ids, or commit ids, or the word none.

One writer per file. A reply is a new file. Do not edit someone else's
handoff. Existing files keep the names they were written under.

When a written handoff asks for a decision, these are the named phrases:

- yes. He approves that exact proposal. The owner may apply it, and nothing beside it.
- no. He refuses it. Nothing is applied.
- defer. Set it aside. It stays written, and it is not applied.
- split this. Break the proposal into smaller pieces and bring each one back.
- ledger update. Write the outcome into the intent ledger. This phrase does not apply code.

Christian can also approve or refuse in the chat.

## First-week files (history)

The INT-304 first-week loop was proved by hand in three files that remain
in the tree: zero/handoffs/2026-11-10-jarvis.md,
zero/handoffs/2026-11-10-cursor.md, and
zero/handoffs/2026-11-10-zero-digest.md. They keep the names they were
written under. They are not the live staffing.
