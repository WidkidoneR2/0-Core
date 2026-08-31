I am building NovaShell because every shell I have used gets in my way. bash asks me to remember what I typed yesterday. zsh asks me to configure what the shell should already know. nushell asks me to learn a new language to get back what I had. None of them are mine. NovaShell is.
NovaShell is shortened to `nsh` in commands and configuration. It was Faelight Shell and `fsh` until September 2026, and the rename is the reason this paragraph is shorter than it was: `fsh` is fish's abbreviation, and a name that needed a paragraph explaining it was not fish had already told me something. The full name is NovaShell. The binary is `nsh`. It is its own project, not a Faelight component. The shell is mine.
This document is what NovaShell is, what it is not, and the rules I follow when changing it. It is short on purpose. If this document gets long, I have drifted.
NovaShell is a shell that remembers, reasons, and refuses to lie.
- It remembers: every command, every failure, every pattern, in a real database with a real schema. Not a ~/.history file. Queryable structured data.
- It reasons: it watches what I do, learns my patterns, and tells me what comes next -- but only when confidence and frequency are both high enough to earn the interruption.
- It refuses to lie: when something fails, it says so. When confidence is low, it stays silent. When data shows a pattern is weak, NovaShell does not pretend it is strong.
Every decision in NovaShell is either documented in an intent, recorded in a commit, or written into this file. Nothing happens because it "seemed like a good idea." The forest remembers.
- It is not a wrapper around bash. It does not translate my commands into bash and run them. When NovaShell handles a command, NovaShell owns it.
- It is not configurable into something else. There is no plugin system that can change what the shell is. There are builtins, and there is sh fallback for what builtins do not yet cover. That is all.
- It is not magic. Every suggestion has a citation. Every prediction has a confidence score. Every pattern has a frequency count I can read in the database with a SELECT.
- It is not finished. It ships in pieces. Each piece is demonstrated before it is called done.
These do not change. When I am tempted to break them for a feature, I do not ship the feature.
1. **Nothing runs without explicit human authorization.** Every action with consequence asks. Every automation is proposed, never taken. The shell executes; I decide.
2. **Every suggestion cites its source.** When NovaShell says "you usually run fg commit next," it means it counted. It can show me the count. It can show me the confidence. It can show me when it learned it. If it cannot, it does not speak.
3. **Silence is a valid answer.** When a threshold is not met, the shell stays quiet. Loud shells train users to ignore them. NovaShell does not train me to ignore it.
4. **Data that looks stored is actually stored.** If NovaShell writes a row to state.db, a direct sqlite3 query must find it. No silent failures, no swallowed errors, no `.ok()` on a path that matters.
5. **One working example beats ten designed features.** I implement the smallest thing that actually works before I plan the next thing.
Trust is not a setting. It is a history.
- Every intent documents what I set out to do and whether I actually did it. Gates are checked only when the thing they describe has been demonstrated on real data.
- Every schema migration is backed up before it runs, verified after it completes, and committed with a message that names what changed.
- Every pattern the shell learns has a frequency count and a confidence score. Neither is for show -- both are read before NovaShell decides whether to speak.
I do not claim NovaShell is the best shell. I claim it is becoming the shell I want to use every day, forever. The test is simple: do I still reach for NovaShell when it would be easier to open zsh?
So far, yes.
The path is plain:
- Ship one verifiable improvement at a time.
- Break nothing that already works.
- Document every decision where the next version of me can find it.
- Keep NovaShell faster, quieter, and more honest than what I replace it with.
I am not building this for an audience. I am building it because I want a shell that works the way a shell should work, and the only way to get one is to write it.
When I am tired, or late, or tempted to skip a verification step because "it will probably work" -- I stop. I commit what I have. I lock core. I come back tomorrow.
A shell built in fatigue becomes a shell I cannot trust. And a shell I cannot trust is not NovaShell.
---
*This file changes when my thinking changes -- not when the code changes. If the code drifts from this document, either the code is wrong or this document needs revision. Both require deliberate action, not drift.*
🌲