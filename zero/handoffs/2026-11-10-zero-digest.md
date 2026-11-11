# The Zero Digest: three knocks, then the door

- Agent: Jarvis
- Status: open
- Intent: INT-311
- Owner: Christian
- NeedsHuman: yes
- NeedsHumanReason: G1, G2, and G3 need a ruling before cistart 311
- Summary: The Conductor sent INT-311 to Cursor. Cursor came back with the tree, not a patch. Three questions are waiting, and the door stays shut until you answer.
- Details:

  ```
        ______
       |      |
       | 311  |
       |  __  |
       | |  ||
       | |__||
       |______|
  ```

  🚪 This is the first week, done by hand.

  On the day INT-307 closed, 57ca26ea, the finding screen was finished and the ledger study was still a study. The next commit, e32c7755, filed four intents and said they came from that study. One of them is INT-311, "A doors command that does not ship." It sits in future/, status planned, every gate open. Nobody has run cistart 311. That is still correct.

  Jarvis wrote the first knock: zero/handoffs/2026-11-10-jarvis.md. Read the intent. Reply in a new file. Write no code. The file still says Status open. It has one writer, and the writer was Jarvis, so the reply did not touch it. The reply is what calls it answered.

  Cursor wrote the second knock: zero/handoffs/2026-11-10-cursor.md. The recon holds. There is no doors command. The pieces a person retypes are real, and they do not know about each other.

  | Knock | What it is | Last word we have |
  |---|---|---|
  | cargo test -p core | the engine's tests | 277, recorded 2026-11-10, not re-run today |
  | cargo test -p novashell -p nsh-test | one invocation, or the registry feature stays off | 258 and 6, same record |
  | the nsh-test binary | the suite, after that same paired build | the push gate budgets about a minute |
  | core intent validate | the ledger's own door | 0.01s here, 348 valid |
  | d | core doctor run | its own door, not a copy |
  | cargo test -p zero-doctor | 95 tests of the doctor engine | proposed out of the set |
  | ship | a release copy onto PATH, no rollback | never called from here |

  The push hook already runs the suite and cargo test -p core, then the seal check. It does not run the rest. That gap is the whole intent.

  What happens the moment you rule: three RULED lines land in INT-311, in your words. Then, and only then, cistart 311 is allowed, and the command itself can be written, red first, still not shipping. Until those lines exist, the door in the drawing does not open. ProposedStatus stays planned. Nothing in this digest applies a change.

  Three questions. Each one takes an approval phrase: yes, no, defer, split this, or ledger update.

  1. The name. Proposal: doors. The word is free. yes names it doors. no sends the name back. defer leaves G1 open.

  2. The home. Proposal: a script at zero/scripts/doors, not a core subcommand. A core command would hold the runtime lock for the whole run, and core does not start the test suite today. The script must not call ship and must not reload. yes takes the script. no, if you want core doors instead, say so. defer leaves G2 open.

  3. zero-doctor. Proposal: no. Leave the 95 tests out. d stays core doctor run either way. yes accepts leaving them out. no means put cargo test -p zero-doctor in the set. defer leaves G3 open.
- Next: Answer the three questions, each with an approval phrase.
- Files: zero/intents/future/311-a-doors-command-that-does-not-ship.md, zero/handoffs/2026-11-10-jarvis.md, zero/handoffs/2026-11-10-cursor.md
- Links: INT-311, INT-304, zero/handoffs/2026-11-10-jarvis.md, zero/handoffs/2026-11-10-cursor.md
