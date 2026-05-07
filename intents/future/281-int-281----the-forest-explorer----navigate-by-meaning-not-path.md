---
id: 281
title: "The Forest Explorer -- Navigate by Meaning Not Path"
status: planned
date: 2026-05-07
tags: [forest-explorer, fm, navigation, semantic, intelligence, future, vision]
depends_on: [280]
---
This intent begins only if INT-280 (faelight-fm v2 evolution) 
proves insufficient to deliver the vision.
Or it begins after INT-280 succeeds,
as the next leap.
---
THE VISION
Not:
  /home/christian/0-core/rust-tools/faelight-lock/src/main.rs
But:
  Active Work          -- what you are building right now
  Recent Risk          -- files changed near health events
  Intent Clusters      -- files grouped by what you were building
  Session Artifacts    -- outputs, logs, checkpoints from this session
  Operational Drift    -- files that should have been touched but weren't
  Knowledge Paths      -- how you got from A to B, replayable
The primary navigation is by meaning.
The filesystem path is secondary -- still accessible, never primary.
---
WHAT CHANGES
The mental model shifts:
  FROM: where is this file?
  TO:   what does this file mean to the forest?
The views change:
  FROM: directory tree
  TO:   semantic clusters, intent groups, risk surfaces
The language changes:
  FROM: ls, cd, find
  TO:   explore active, explore risk, explore intent 243
The filesystem is still there.
You can always navigate by path.
But the default view is forest-native.
---
PRIMARY VIEWS
Active Work:
  Files touched in the last 4 hours
  Files in the current intent's domain
  Files with uncommitted changes
  The work right now, surfaced immediately
Recent Risk:
  Files changed within 1 hour of a health drop
  Files that appear in contradiction reports
  Files touched during incidents
  The danger zones of the forest
Intent Clusters:
  Files grouped by the intent that created or modified them
  Click INT-243: see every file the lock screen build touched
  Navigate the history of the forest through its work
Session Artifacts:
  Checkpoints from today
  Log files from this session
  Deploy records
  The evidence of today's work
Operational Drift:
  Files Friday expected to be touched but weren't
  Stale files in active domains
  Things the forest planned but hasn't done
  The gap between intention and reality
Knowledge Paths:
  How did we get from A to B?
  Replay the sequence of file changes that built a feature
  Understand causality through file history
---
GATES (future, after INT-280)
[ ] Primary view shows Active Work not filesystem tree
[ ] Intent Clusters navigation works -- INT-243 shows its files
[ ] Recent Risk surfaces files near health events
[ ] Session Artifacts shows today's checkpoints and logs
[ ] Operational Drift identifies files that should have been touched
[ ] Knowledge Paths replays a build sequence
[ ] Filesystem path navigation still accessible (press / or f5)
[ ] Friday panel shows context for every view
"The forest explorer does not show you
where files are.
It shows you what they mean.
The path is just the address.
The meaning is the forest." 🌲
