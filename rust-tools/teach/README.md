# teach v1.0.0

🎓 **Interactive Learning System** - Master the Faelight Forest through guided lessons, quizzes, and daily wisdom.

## Philosophy

> "The forest teaches those who listen."

teach transforms 0-Core from a collection of tools into a **living textbook**. Learn the philosophy, master the workflows, understand the why behind every decision.

## The Vision

From [Intent 007](../../INTENT/complete/007-teaching-mode-v410.md):
> "0-Core philosophy is 'simple but easy to learn.' New users need guidance."

teach evolved from cancelled [Intent 014](../../INTENT/cancelled/014-teach-advanced---system-wisdom.md) and [Intent 029](../../INTENT/cancelled/029-teach-advanced.md), incorporating their best ideas:
- Interactive lessons
- Quiz mode
- Progress tracking
- Daily wisdom
- Achievement system

## Features

- 🎓 **8 Interactive Lessons** - From philosophy to advanced workflows
- 🎯 **Quiz Mode** - Test your knowledge (10 questions)
- 📊 **Progress Tracking** - See what you've learned
- 💡 **Daily Wisdom** - Random tips and insights
- 🏆 **Achievements** - Earn badges for completion
- 🎲 **Random Lesson** - Surprise learning
- 📚 **Topic Index** - Quick reference
- 🌈 **Beautiful TUI** - Powered by gum

## Usage

### Start Learning
```bash
teach                    # Interactive menu
teach --begin            # Start from lesson 1
teach --random           # Random lesson
```

### Test Knowledge
```bash
teach --quiz             # Take the quiz (10 questions)
teach --quiz --hard      # Expert mode (coming soon)
```

### Track Progress
```bash
teach --stats            # View your progress
teach --tips             # Daily wisdom tip
```

### Get Help
```bash
teach --help             # Show all options
teach --version          # Show version
teach --health           # Check dependencies
```

## Lessons

### 1. Welcome & Philosophy 🌲
Learn the core principles:
- Manual control over automation
- Intent over convention
- Understanding over convenience
- Recovery over perfection

**Duration:** 3 minutes  
**Difficulty:** Beginner

### 2. Directory Structure 📁
Master the numbered system:
- `~/0-core/` - Configuration
- `~/1-src/` - Source code
- `~/2-projects/` - Active work
- `~/3-archive/` - Completed
- `~/4-media/` - Media files

**Duration:** 2 minutes  
**Difficulty:** Beginner

### 3. Core Tools 🔧
Essential utilities:
- `doctor` - System health checks
- `workspace-view` - Workspace intelligence
- `safe-update` - Safe system updates
- `entropy-check` - Drift detection
- `faelight-git` - Git intelligence

**Duration:** 5 minutes  
**Difficulty:** Beginner

### 4. Daily Workflow 📅
Morning routine & habits:
- Health checks
- Workspace monitoring
- Profile management
- Weekly updates

**Duration:** 3 minutes  
**Difficulty:** Intermediate

### 5. System Profiles 🎮
Switch entire system states:
- 🏠 `default` - Balanced
- 🎮 `gaming` - Performance
- 💼 `work` - Focus mode
- 🔋 `low-power` - Battery saver

**Duration:** 4 minutes  
**Difficulty:** Intermediate

### 6. Intent Ledger 📜
Decision tracking system:
- `decisions/` - Architecture
- `experiments/` - Trials
- `philosophy/` - Core beliefs
- `future/` - Roadmap
- `incidents/` - Lessons learned

**Duration:** 5 minutes  
**Difficulty:** Intermediate

### 7. Customization 🎨
Make it yours:
- Create custom profiles
- Add new tools
- Modify workflows
- Extend the system

**Duration:** 6 minutes  
**Difficulty:** Advanced

### 8. Useful Aliases ⚡
Power-user shortcuts:
- Navigation aliases
- Git shortcuts
- System commands
- Core protection

**Duration:** 4 minutes  
**Difficulty:** Beginner

## Quiz Mode

Test your knowledge with 10 carefully crafted questions covering:
- Core philosophy
- Tool usage
- Workflow patterns
- System architecture
- Best practices

**Passing score:** 7/10 (70%)  
**Perfect score achievement:** 🏆 Forest Scholar

### Sample Questions
```
Q: What's the core philosophy of 0-Core?
A) Automation over control
B) Manual control over automation ✓
C) Convenience over understanding
D) Perfection over recovery

Q: Which tool checks system health?
A) health-check
B) system-doctor
C) doctor ✓
D) check-health
```

## Progress Tracking

Your learning journey is saved to:
```
~/.local/state/faelight/learning.json
```

Tracks:
- ✅ Lessons completed
- 📊 Quiz scores
- 🏆 Achievements earned
- 📅 Last studied date
- ⏱️ Total learning time

### Achievements

Earn badges for milestones:

- 🌱 **First Steps** - Complete lesson 1
- 🌿 **Directory Master** - Complete lesson 2
- 🔧 **Tool Proficient** - Complete lesson 3
- 📅 **Workflow Expert** - Complete lesson 4
- 🎮 **Profile Pro** - Complete lesson 5
- 📜 **Intent Scholar** - Complete lesson 6
- 🎨 **Customization Guru** - Complete lesson 7
- ⚡ **Alias Ninja** - Complete lesson 8
- 🎓 **Full Course** - Complete all 8 lessons
- 🏆 **Forest Scholar** - Perfect quiz score (10/10)
- 🌲 **Faelight Master** - 100% completion + perfect quiz

## Daily Wisdom

Get random tips about:
- System philosophy
- Tool usage
- Workflow optimization
- Best practices
- Hidden features
```bash
teach --tips
```

**Example output:**
```
💡 Daily Wisdom from the Faelight Forest

"Use workspace-view --watch for live monitoring of your workspaces.
Real-time intelligence beats switching between them!"

—Lesson 3: Core Tools 🌲
```

## Statistics Dashboard

View your learning progress:
```bash
teach --stats
```

**Example output:**
```
🎓 Your Learning Journey
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 Progress: 75% (6/8 lessons)
🎯 Quiz Score: 8/10 (80%)
🏆 Achievements: 7/11

Completed Lessons:
  ✅ 1. Welcome & Philosophy
  ✅ 2. Directory Structure
  ✅ 3. Core Tools
  ✅ 4. Daily Workflow
  ✅ 5. System Profiles
  ✅ 6. Intent Ledger
  ⬜ 7. Customization
  ⬜ 8. Useful Aliases

Recent Achievements:
  🎮 Profile Pro (2026-01-22)
  📜 Intent Scholar (2026-01-22)
  🎓 Full Course (not yet earned)

Next Goal: Complete all lessons for 🎓 Full Course
```

## Flags
```
--begin              Start from lesson 1
--random             Random lesson
--quiz               Take the quiz
--stats              View progress & achievements
--tips               Daily wisdom tip
--reset              Reset all progress (confirmation required)
--health             Check dependencies (gum)
--version, -v        Show version
--help, -h           Show help
```

## Requirements

**Required:**
- `gum` - TUI components ([charm.sh/gum](https://github.com/charmbracelet/gum))

**Optional:**
- `doctor` - For health check lessons
- `workspace-view` - For workspace lessons
- `safe-update` - For update lessons

**Install gum:**
```bash
yay -S gum
```

## Files Created
```
~/.local/state/faelight/learning.json    Progress tracking
~/.local/state/faelight/quiz-scores.json Quiz history
```

## Integration

### With Other Tools

teach references and integrates with:
- `doctor` - Health checks
- `workspace-view` - Workspace monitoring
- `safe-update` - System updates
- `entropy-check` - Drift detection
- `faelight-git` - Git intelligence
- `intent` - Intent ledger
- `profile` - System profiles

### In Daily Workflow
```bash
# Morning: Get daily wisdom
teach --tips

# Learning session
teach

# Check progress
teach --stats

# Test knowledge
teach --quiz
```

## Design Principles

### 1. Interactive, Not Passive
Learning by doing. Every lesson is hands-on with real system interaction.

### 2. Progressive Difficulty
Start simple (philosophy), build to advanced (customization).

### 3. Persistent Progress
Your journey is saved. Resume anytime, track achievements.

### 4. Immediate Feedback
Quiz answers explained. Progress visible. Achievements earned.

### 5. Living Documentation
Lessons update as tools evolve. Always current, never stale.

## Learning Paths

### New User Path
```
1. Welcome & Philosophy (understand the why)
2. Directory Structure (know where things are)
3. Core Tools (learn essential commands)
4. Daily Workflow (establish habits)
```

**Time:** ~15 minutes  
**Goal:** Productive on day 1

### Power User Path
```
5. System Profiles (master state management)
6. Intent Ledger (understand decision tracking)
7. Customization (extend the system)
8. Useful Aliases (work faster)
```

**Time:** ~20 minutes  
**Goal:** Full system mastery

### Quick Refresh Path
```
teach --tips          # Daily wisdom
teach --random        # Random lesson
teach --quiz          # Test retention
```

**Time:** 2-5 minutes  
**Goal:** Stay sharp

## Common Workflows

### First Time Setup
```bash
# Install dependency
yay -S gum

# Start learning
teach --begin

# Complete all lessons
# (follow the guided flow)

# Test knowledge
teach --quiz

# Check progress
teach --stats
```

### Daily Learning Habit
```bash
# Morning wisdom
teach --tips

# Weekly quiz
teach --quiz

# Random refresher
teach --random
```

### Review Specific Topic
```bash
# Launch teach
teach

# Select lesson from menu
# e.g., "5. System Profiles"
```

## Troubleshooting

### "gum is required"
**Cause:** gum not installed  
**Fix:** `yay -S gum`

### Progress not saving
**Cause:** Directory doesn't exist  
**Fix:** teach creates it automatically, check permissions

### Lessons reference old tools
**Cause:** teach needs updating  
**Fix:** This is v1.0.0 with current tool versions

## Philosophy in Practice

**Scenario:** New user joins the Faelight Forest.

**Without teach:**
```bash
# Read docs
# Try commands
# Break things
# Fix things
# Learn slowly
```

**With teach:**
```bash
teach --begin
# 30 minutes later: Full understanding
# Philosophy internalized
# Tools mastered
# Workflows established
teach --quiz
# Knowledge verified
# Ready to work
```

**This is intentional stewardship.**

## Comparison

### vs README.md
- ❌ Static documentation
- ❌ No progress tracking
- ❌ No verification

### vs Man Pages
- ✅ Reference documentation
- ❌ No philosophy
- ❌ No learning path

### vs teach
- ✅ Interactive learning
- ✅ Progress tracking
- ✅ Knowledge verification
- ✅ Philosophy embedded
- ✅ Achievement system
- ✅ Daily wisdom

## Future Possibilities

From cancelled Intent 014:
- 🎯 Recovery guides (disk failure, boot issues, config breaks)
- 🎯 Migration guides (new hardware, different distro)
- 🎯 Debug walkthroughs (Wayland, network, audio)
- 🎯 Expert mode quiz (harder questions)
- 🎯 Code lessons (extend with Rust)
- 🎯 Video lessons (recorded walkthroughs)

## Exit Codes

- `0` - Success
- `1` - Error (missing dependency, etc.)

## Contributing

Have a lesson idea? Found outdated information? Want to add quiz questions?

1. Update the lesson in `src/main.rs`
2. Test with `teach`
3. Submit intent or PR

**Lesson guidelines:**
- Clear, concise explanations
- Real examples from actual system
- Progressive difficulty
- 2-6 minutes per lesson
- Philosophy woven in

## Easter Eggs

Hidden features for explorers:
- Try running teach at specific times
- Complete all lessons in one session
- Get perfect quiz score on first try
- Discover the secret command

*The forest rewards the curious.* 🌲

---

**Learn the forest. Master the system. Become the steward.** 🎓🌲
