I am building Faelight Shell because every shell I have used gets in my way. bash asks me to remember what I typed yesterday. zsh asks me to configure what the shell should already know. nushell asks me to learn a new language to get back what I had. None of them are mine. Faelight Shell is.
Faelight Shell is shortened to `fsh` in commands and configuration. When this document says "fsh," it means Faelight Shell -- not fish, not any fork of fish, not any other project. The full name is Faelight Shell. The binary is `fsh`. The shell is mine.
This document is what Faelight Shell is, what it is not, and the rules I follow when changing it. It is short on purpose. If this document gets long, I have drifted.
Faelight Shell is a shell that remembers, reasons, and refuses to lie.
- It remembers: every command, every failure, every pattern, in a real database with a real schema. Not a ~/.history file. Queryable structured data.
- It reasons: it watches what I do, learns my patterns, and tells me what comes next -- but only when confidence and frequency are both high enough to earn the interruption.
- It refuses to lie: when something fails, it says so. When confidence is low, it stays silent. When data shows a pattern is weak, Faelight Shell does not pretend it is strong.
Every decision in Faelight Shell is either documented in an intent, recorded in a commit, or written into this file. Nothing happens because it "seemed like a good idea." The forest remembers.
- It is not a wrapper around bash. It does not translate my commands into bash and run them. When Faelight Shell handles a command, Faelight Shell owns it.
- It is not configurable into something else. There is no plugin system that can change what the shell is. There are builtins, and there is sh fallback for what builtins do not yet cover. That is all.
- It is not magic. Every suggestion has a citation. Every prediction has a confidence score. Every pattern has a frequency count I can read in the database with a SELECT.
- It is not finished. It ships in pieces. Each piece is demonstrated before it is called done.
These do not change. When I am tempted to break them for a feature, I do not ship the feature.
1. **Nothing runs without explicit human authorization.** Every action with consequence asks. Every automation is proposed, never taken. The shell executes; I decide.
2. **Every suggestion cites its source.** When Faelight Shell says "you usually run fg commit next," it means it counted. It can show me the count. It can show me the confidence. It can show me when it learned it. If it cannot, it does not speak.
3. **Silence is a valid answer.** When a threshold is not met, the shell stays quiet. Loud shells train users to ignore them. Faelight Shell does not train me to ignore it.
4. **Data that looks stored is actually stored.** If Faelight Shell writes a row to state.db, a direct sqlite3 query must find it. No silent failures, no swallowed errors, no `.ok()` on a path that matters.
5. **One working example beats ten designed features.** I implement the smallest thing that actually works before I plan the next thing.
Trust is not a setting. It is a history.
- Every intent documents what I set out to do and whether I actually did it. Gates are checked only when the thing they describe has been demonstrated on real data.
- Every schema migration is backed up before it runs, verified after it completes, and committed with a message that names what changed.
- Every pattern the shell learns has a frequency count and a confidence score. Neither is for show -- both are read before Faelight Shell decides whether to speak.
- Every friction point I hit goes into INT-245. When I hit the same thing twice, I stop and fix it before continuing.
Concrete example of the last point: Faelight Shell's `save_history_entry` silently dropped database errors during a schema migration. Shell history froze for an hour before I noticed. The bug was in one line using `.ok()` to swallow the error. That one line is now a friction item in INT-245. Invariant 4 exists because of that bug.
I do not claim Faelight Shell is the best shell. I claim it is becoming the shell I want to use every day, forever. The test is simple: do I still reach for Faelight Shell when it would be easier to open zsh?
So far, yes.
The path is plain:
- Ship one verifiable improvement at a time.
- Break nothing that already works.
- Document every decision where the next version of me can find it.
- Keep Faelight Shell faster, quieter, and more honest than what I replace it with.
I am not building this for an audience. I am building it because I want a shell that works the way a shell should work, and the only way to get one is to write it.
When I am tired, or late, or tempted to skip a verification step because "it will probably work" -- I stop. I commit what I have. I lock core. I come back tomorrow.
A shell built in fatigue becomes a shell I cannot trust. And a shell I cannot trust is not Faelight Shell.
---
*This file changes when my thinking changes -- not when the code changes. If the code drifts from this document, either the code is wrong or this document needs revision. Both require deliberate action, not drift.*
🌲
