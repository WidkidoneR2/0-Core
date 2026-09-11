//! jobs — background job control (Phase 8)
//! Supports: cmd &, jobs, fg N, bg N, kill %N
use colored::*;
use std::time::Instant;

/// What the shell has OBSERVED about a job. Not a guess, and not a default.
///
/// INT-188 STEP 3. Before this, a job was running or absent, and check_completed decided which
/// by asking Child::try_wait() -- waitpid(WNOHANG) with no WUNTRACED. That call is STRUCTURALLY
/// BLIND to a stopped process: Ctrl+Z could suspend a job forever and the table would report it
/// running until the end of the session. The wrong syscall for the question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobState {
    /// Seen alive, or seen nothing and the members are still waitable.
    Running,
    /// WIFSTOPPED. The state this shell could not previously express at all.
    Stopped,
    /// Every member reached a terminal observation.
    Done(JobResult),
    /// NOT A LIFECYCLE STATE. THE SHELL COULD NOT ESTABLISH ONE.
    ///
    /// Running / Stopped / Done are things a PROCESS does. Unknown is a thing the SHELL failed
    /// to do, and conflating them is how "I could not look" becomes "there is nothing there" --
    /// the same collapse INT-192 built check::Skipped for, INT-245 gave the value pipeline a
    /// word for, and INT-246 made the sandbox admit.
    ///
    /// AND THE ARROW THAT DOES NOT EXIST:  Unknown --X--> Done
    ///
    /// Inability to observe death is never evidence of death. A job in this state STAYS IN THE
    /// TABLE with its reason visible. It does not age out: a timeout would convert "the shell
    /// does not know" into "the shell stopped caring", which are different facts and only one
    /// of them is true.
    Unknown(JobObservationError),
}

/// A TERMINAL observation -- how a process actually ended.
///
/// Separate from JobState because Exited and Signaled are facts about a FINISHED process, while
/// Running and Stopped are facts about a live one. Keeping them apart is what lets a job be
/// "stopped, and one member has already exited" without the type fighting the truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobResult {
    Exited(i32),
    /// NOT Exited(128 + n). A signalled death is a different fact from an exit code that happens
    /// to be large, and folding one into the other is how kill -9 becomes indistinguishable
    /// from a program that returned 137 on purpose.
    Signaled(i32),
}

/// Why the shell could not establish a job's state.
///
/// STRUCTURED, NOT A STRING. A UI that has to parse a diagnostic to decide what to show is a UI
/// that will one day parse it wrong. The variants can grow without every consumer breaking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobObservationError {
    /// waitpid failed with this errno.
    Waitpid(i32),
    /// The job was registered with no process group, so there is nothing to ask about. This is
    /// the state INT-188 exists to end; a job here cannot be signalled without signalling the
    /// SHELL, which is why it is named rather than hidden.
    NoProcessGroup,
}

impl std::fmt::Display for JobObservationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobObservationError::Waitpid(e) => write!(f, "waitpid failed: errno {}", e),
            JobObservationError::NoProcessGroup => write!(f, "job has no process group"),
        }
    }
}

/// One process inside a job, and what has been observed about IT.
///
/// PER-MEMBER ACCOUNTING, BECAUSE waitpid IS AN OBSERVATION MECHANISM AND NOT THE JOB'S IDENTITY.
/// The table knows a job consists of pids X, Y, Z in group G. Asking the kernel returns something
/// that happened to ONE of them. Those observations ACCUMULATE; the job's state is derived from
/// them rather than guessed from one anonymous answer.
#[derive(Debug)]
pub struct JobMember {
    pub pid: u32,
    /// None while live. Some once terminal, after which the pid MUST NOT be waited on or
    /// signalled again -- pids are recycled.
    pub result: Option<JobResult>,
    /// True between a WIFSTOPPED and the next WIFCONTINUED.
    pub stopped: bool,
}

#[derive(Debug)]
pub struct Job {
    pub id: JobId,
    pub cmd: String,
    /// The status-bearing stage. For a pipeline this is the LAST one, per POSIX.
    /// The status-bearing stage, WHEN THE TABLE STARTED IT.
    ///
    /// None FOR A SUSPENDED FOREGROUND JOB. That job was started by the execution path and
    /// already waited on with waitpid -- the Child was consumed there and cannot be handed
    /// over. The job is no less real: its pgid and members are what job control actually
    /// needs, and observe() has never read this field.
    ///
    /// WHICH MAKES THE TWO REMAINING READERS THE INTERESTING ONES. fg() waits on this Child
    /// rather than resuming a GROUP, and kill_job() kills this one pid and removes the row
    /// without reaping -- the zombie seen on 2026-09-13. Both are wrong for a pipeline and
    /// wrong for a suspended job; making the field optional is what stops them compiling.
    pub child: Option<std::process::Child>,
    /// Upstream stages of a backgrounded pipeline, in stage order. Empty for a single
    /// command. Held so check_completed can reap them -- registering only the tail would
    /// leave every earlier stage a zombie.
    /// INT-188: NO READER, AND KEPT ANYWAY -- THIS FIELD IS OWNERSHIP, NOT DATA.
    ///
    /// It existed so check_completed could reap upstream stages with up.wait(). observe()
    /// now waits on EVERY member by pid, so nothing reads it. Deleting it would DROP the
    /// Child handles, and a dropped Child is NEVER waited on -- which is precisely how
    /// zerombies are leaked. The handles must outlive the job even though no code asks
    /// them anything.
    ///
    /// The allow is a DECISION, not a suppression: the compiler is right that nothing
    /// reads it, and wrong that it is therefore useless.
    #[allow(dead_code)]
    pub rest: Vec<std::process::Child>,
    /// The OS's job-control identity for this job -- the process group every stage shares.
    ///
    /// ⭐ INT-188 STEP 1. The JobId doc below already names the three-level model -- JobId,
    /// ProcessGroupId, Pid -- and says INT-188 would lean on it. This is the middle one
    /// arriving. Without it the table knows WHICH job and WHICH process, and has no way to
    /// name the thing a signal is actually delivered to.
    ///
    /// ⚠️ None means THIS JOB HAS NO GROUP OF ITS OWN, not "unknown". A job registered
    /// before the group could be established shares the shell's group, which is exactly the
    /// state INT-188 exists to end: signalling it would signal the SHELL. Never treat None
    /// as "use the current group".
    pub pgid: Option<u32>,
    /// Every process in this job, in stage order. members[0] is the group leader.
    pub members: Vec<JobMember>,
    /// The last state the shell OBSERVED. Never inferred from absence.
    pub state: JobState,
    pub started: Instant,
}

impl Job {
    pub fn elapsed(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    /// Ask the kernel about every LIVE member, and fold the answers into one state.
    ///
    /// PER PID, NOT PER GROUP. waitpid(-pgid, ..) returns a status with NO PID attached, so it
    /// can say "something in this job stopped" and never which thing. Every member is a direct
    /// child of this shell, so each can be asked individually -- and the job's state is DERIVED
    /// from what its members reported rather than guessed from one anonymous answer.
    ///
    /// A MEMBER IS ASKED ONCE AND THEN NEVER AGAIN. After a terminal result, its pid is released
    /// by the kernel and may be REUSED. Waiting on it again is at best ECHILD and at worst an
    /// answer about somebody else's program.
    pub fn observe(&mut self) -> JobState {
        use rustix::process::WaitOptions;

        if self.pgid.is_none() {
            return JobState::Unknown(JobObservationError::NoProcessGroup);
        }

        let mut any_live = false;
        let mut any_stopped = false;
        let mut failure: Option<JobObservationError> = None;

        for m in self.members.iter_mut() {
            if m.result.is_some() {
                continue;
            }
            let pid = match rustix::process::Pid::from_raw(m.pid as i32) {
                Some(p) => p,
                None => {
                    failure = Some(JobObservationError::Waitpid(0));
                    continue;
                }
            };
            // WUNTRACED makes a stop visible; CONTINUED makes a resume visible. try_wait
            // passes neither, which is why it could never report either one.
            let opts = WaitOptions::NOHANG | WaitOptions::UNTRACED | WaitOptions::CONTINUED;
            match rustix::process::waitpid(Some(pid), opts) {
                Ok(None) => {
                    // NOTHING NEW -- WHICH IS NOT THE SAME AS NOTHING IS WRONG.
                    //
                    // Measured 2026-09-13: a stopped job reported "stopped" once and then
                    // "continued" on the very next prompt, without anyone resuming it.
                    // waitpid reports a stop ONCE; every poll after that returns Ok(None).
                    // This arm set any_live and ignored m.stopped, so the fold computed
                    // Running and the change detector faithfully announced a resume that
                    // never happened.
                    //
                    // THE FLAG IS THE MEMORY. A stop is an EVENT the kernel delivers once;
                    // "still stopped" is a STATE only the shell remembers. Reading only
                    // fresh events makes the shell forget every fact the instant it is
                    // delivered.
                    any_live = true;
                    if m.stopped {
                        any_stopped = true;
                    }
                }
                Ok(Some(status)) => {
                    if let Some(code) = status.exit_status() {
                        m.result = Some(JobResult::Exited(code as i32));
                        m.stopped = false;
                    } else if let Some(sig) = status.terminating_signal() {
                        m.result = Some(JobResult::Signaled(sig as i32));
                        m.stopped = false;
                    } else if status.stopping_signal().is_some() {
                        m.stopped = true;
                        any_stopped = true;
                        any_live = true;
                    } else {
                        // A CONTINUED report, or something unclassified. Either way the
                        // process is running again, and this is the ONLY place a stop is
                        // cleared -- because it is the only place the kernel said so.
                        m.stopped = false;
                        any_live = true;
                    }
                }
                Err(e) => {
                    // ECHILD IS NOT "THE JOB FINISHED". It means this pid is not a waitable
                    // child of ours -- reaped elsewhere, never ours, or gone in a way we did
                    // not witness. Recording the errno keeps the difference; assuming
                    // completion would manufacture an ending nobody observed.
                    failure = Some(JobObservationError::Waitpid(e.raw_os_error()));
                }
            }
        }

        if let Some(err) = failure {
            return JobState::Unknown(err);
        }
        if any_stopped {
            return JobState::Stopped;
        }
        if any_live {
            return JobState::Running;
        }
        // Every member reached a terminal observation. POSIX gives the job the LAST stage's
        // status, which members.last() is by construction -- spawn order is stage order.
        let last = self
            .members
            .last()
            .and_then(|m| m.result)
            .unwrap_or(JobResult::Exited(-1));
        JobState::Done(last)
    }
}

/// The SHELL's identity for a job. Not a position, not a process.
///
/// ⚠️ WHY A TYPE AND NOT A `usize`. The counter was already correct -- monotonic, never recycled,
/// and both lookups find a job BY IDENTITY before touching an index. What was wrong is that `id`
/// and a vector index were the SAME PRIMITIVE, so nothing stopped one being used as the other.
///
/// ★ THIS SHELL HAS PAID FOR THAT EXACT CLASS ONCE ALREADY: `id + 1` on shell_history meant "the
/// next row", four consumers read it as "the next command", and four predictors were deleted on
/// 2026-08-22 because the arithmetic was wrong in a way nothing could catch. A counter and an
/// offset that share a type invite the same mistake; the compiler can refuse it instead.
///
/// ⭐ AND THE THREE-LEVEL DISTINCTION THIS KEEPS APART, which INT-188 will lean on:
///     JobId           the shell's identity for a job
///     ProcessGroupId  the OS's job-control identity
///     Pid             one individual process
/// A job holds MULTIPLE processes once pipelines and process groups exist, so a pid can never be
/// the shell's identity for one. That distinction was earned in an incident, and `kill`'s own
/// comment records it: parsing any number as a job id made `kill <PID>` a silent no-op, which
/// turned `vm down` into nothing and left two VMs running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JobId(u64);

impl JobId {
    /// The value a person typed. FALLIBLE ON PURPOSE -- see the note on `fg` in engine.rs: parsing
    /// with a fallback turned `fg banana` into `fg 1`, foregrounding an arbitrary job.
    pub fn parse(s: &str) -> Option<JobId> {
        // ONE DEFINITION OF WHAT A JOB ID IS, shared by `fg` and `kill`. The % is stripped here so
        // neither caller has to remember that `kill %2` and `fg 2` name the same thing.
        let t = s.trim().trim_start_matches('%');
        match t.parse::<u64>() {
            Ok(n) if n > 0 => Some(JobId(n)),
            _ => None,
        }
    }

    // ⏭ INT-228 DELIBERATELY SHIPS NO BASE32 ENCODING, and the reason is worth keeping.
    // A Crockford Base32 pair was written here -- it excludes I/L/O/U and DECODES the confusables,
    // so a misread `O` still resolves. Then the encoder had no caller, and the tempting fix was to
    // add a column to `jobs` so it would have one.
    //
    // ⚠️ THAT IS BACKWARDS. It turns an internal capability into a UI change because the
    // implementation happens to exist. And `[2R]` is not another rendering of `2`; it is a NEW
    // IDENTIFIER FORM a person has to learn, which deserves an explicit decision rather than being
    // smuggled in beside a type change.
    //
    // ★ THE RULE: do not create UI to give an unused helper a caller. Create the UI when there is a
    // user-facing requirement, then implement exactly what that requirement needs. INT-188 makes
    // job identifiers visible -- stopped, resumed, moved between foreground and background -- and
    // that is where the whole feature gets defined coherently, encoder and display together.
}

impl std::fmt::Display for JobId {
    /// ⚠️ THE DECIMAL FORM IS WHAT `jobs` PRINTS TODAY, and INT-228 does not renumber anything. A
    /// job that was 3 is still 3. The Base32 rendering exists and is tested; adopting it in the
    /// listing is a SEPARATE, VISIBLE change rather than one smuggled in with a type.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct JobTable {
    jobs: Vec<Job>,
    next_id: u64,
}

impl JobTable {
    pub fn new() -> Self {
        Self {
            jobs: vec![],
            next_id: 1,
        }
    }

    /// Spawn a background job from argv. Returns job id.
    ///
    /// ⚠️ THIS SHAPE CANNOT EXPRESS A REDIRECT -- it builds the Command itself and fixes all three
    /// streams. That is fine for legacy's `cmd &` path, which has no IO plan to apply, but the
    /// spine does. `register` below takes an already-built Command for exactly that reason; this
    /// stays as the argv-shaped convenience over it so legacy's call site is untouched.
    pub fn spawn(&mut self, cmd: &str, args: &[String]) -> std::io::Result<JobId> {
        let mut command = std::process::Command::new(cmd);
        command
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());
        self.register(command, cmd)
    }

    /// Spawn an ALREADY-CONFIGURED command as a background job. Returns job id.
    ///
    /// ★ THE CALLER OWNS THE STDIO, which is the whole point: `cmd > log 2>&1 &` needs its streams
    /// wired before the spawn, and a method that fixes them can never express one. Same boundary
    /// `spawn_with_tee` had to adopt before redirects could work at all.
    ///
    /// ⚠️ `label` is DISPLAY ONLY -- what `jobs` lists and what the completion notice names. A
    /// built Command cannot be asked for a tidy name, so the caller passes the one it already has.
    pub fn register(
        &mut self,
        mut command: std::process::Command,
        label: &str,
    ) -> std::io::Result<JobId> {
        // ⭐ INT-188 STEP 1: THE JOB GETS ITS OWN PROCESS GROUP, and this is the one
        // line that does it for a single background command.
        //
        // process_group(0) means setpgid(0, 0) in the child: new group, and the child is
        // its leader -- so the pgid IS the pid. That identity is why no second lookup is
        // needed to learn the group after spawning.
        //
        // ⚠️ THIS IS THE BACKGROUND PATH ONLY, where process_group is sufficient. The
        // FOREGROUND path cannot use it: giving a child the terminal races with posix_spawn,
        // so that path needs pre_exec and a dual setpgid in both parent and child. Step 4.
        use std::os::unix::process::CommandExt;
        command.process_group(0);
        let child = command.spawn()?;
        let pgid = child.id();
        self.register_chain_with_group(vec![child], label, Some(pgid))
    }

    /// Register an already-spawned CHAIN as one job. The last child carries the status;
    /// the rest are held so they are reaped rather than leaked.
    /// Register a chain whose process group the caller ALREADY ESTABLISHED.
    ///
    /// ⭐ THE PGID ARRIVES AS DATA. jobs.rs does not call setpgid and must not -- for a
    /// pipeline it never saw the spawn, and only the spawner knows stage order. That is the
    /// boundary exec.rs:1157 already declares: it spawns and never learns what a JobTable is.
    pub fn register_chain_with_group(
        &mut self,
        mut children: Vec<std::process::Child>,
        label: &str,
        pgid: Option<u32>,
    ) -> std::io::Result<JobId> {
        let Some(child) = children.pop() else {
            return Err(std::io::Error::other("empty pipeline"));
        };
        let id = JobId(self.next_id);
        self.next_id += 1;
        // Stage order, leader first -- children is upstream stages, child is the tail.
        let mut members: Vec<JobMember> = children
            .iter()
            .map(|c| JobMember {
                pid: c.id(),
                result: None,
                stopped: false,
            })
            .collect();
        members.push(JobMember {
            pid: child.id(),
            result: None,
            stopped: false,
        });
        self.jobs.push(Job {
            id,
            cmd: label.to_string(),
            child: Some(child),
            rest: children,
            pgid,
            members,
            state: JobState::Running,
            started: Instant::now(),
        });
        println!(
            "  {} [{}] {} &",
            "○".bright_cyan(),
            id.to_string().bright_white(),
            label.dimmed()
        );
        Ok(id)
    }

    /// Reconcile every job against the kernel, and announce what changed.
    ///
    /// INT-188 STEP 3. This used to ask child.try_wait() and treat ERR AS DEATH:
    ///
    ///     Err(_) => { completed.push(job.id); }
    ///
    /// That is the collapse this shell keeps finding: an unanswerable question reported as a
    /// definite answer. A job the shell could not ask about was silently declared finished and
    /// dropped from the table.
    ///
    /// THE INVARIANT NOW: a job leaves this table ONLY after a positive terminal observation,
    /// or because the user explicitly discarded it. Never because we failed to look.
    pub fn register_suspended(&mut self, pgid: u32, label: &str) -> JobId {
        let id = JobId(self.next_id);
        self.next_id += 1;
        self.jobs.push(Job {
            id,
            cmd: label.to_string(),
            child: None,
            rest: vec![],
            pgid: Some(pgid),
            members: vec![JobMember {
                pid: pgid,
                result: None,
                stopped: true,
            }],
            state: JobState::Stopped,
            started: Instant::now(),
        });
        println!(
            "\n  {} [{}] {} -- {} ({})",
            "\u{2638}",
            id.to_string().bright_white(),
            label.bright_white(),
            "suspended".bright_yellow(),
            format!("fg {} to resume", id).dimmed()
        );
        id
    }

    pub fn check_completed(&mut self) {
        let mut completed = vec![];
        for job in &mut self.jobs {
            let was = job.state.clone();
            let now = job.observe();
            job.state = now.clone();
            if was == now {
                continue;
            }
            let elapsed = job.started.elapsed().as_secs_f64();
            match &now {
                JobState::Running => {
                    // Resumed after a stop. Worth saying -- the user was told it stopped.
                    if was == JobState::Stopped {
                        println!(
                            "\n  {} [{}] {} -- {}",
                            "▶".bright_cyan(),
                            job.id.to_string().bright_white(),
                            job.cmd.bright_white(),
                            "continued".bright_cyan()
                        );
                    }
                }
                JobState::Stopped => {
                    // THE ANNOUNCEMENT THIS SHELL COULD NEVER MAKE BEFORE TODAY.
                    println!(
                        "\n  {} [{}] {} -- {} ({:.1}s)",
                        "♸".bright_yellow(),
                        job.id.to_string().bright_white(),
                        job.cmd.bright_white(),
                        "stopped".bright_yellow(),
                        elapsed
                    );
                }
                JobState::Done(r) => {
                    match r {
                        JobResult::Exited(0) => println!(
                            "\n  {} [{}] {} -- {} ({:.1}s)",
                            "✅".normal(),
                            job.id.to_string().bright_white(),
                            job.cmd.bright_green(),
                            "done".bright_green(),
                            elapsed
                        ),
                        JobResult::Exited(c) => println!(
                            "\n  {} [{}] {} -- {} ({:.1}s) exit {}",
                            "✗".bright_red(),
                            job.id.to_string().bright_white(),
                            job.cmd.bright_red(),
                            "failed".bright_red(),
                            elapsed,
                            c
                        ),
                        // A SIGNALLED DEATH SAYS SO. This used to render as "exit -1"
                        // because status.code() is None for a signal.
                        JobResult::Signaled(s) => println!(
                            "\n  {} [{}] {} -- {} ({:.1}s)",
                            "⚠".bright_red(),
                            job.id.to_string().bright_white(),
                            job.cmd.bright_red(),
                            format!("killed by signal {}", s).bright_red(),
                            elapsed
                        ),
                    }
                    completed.push(job.id);
                }
                JobState::Unknown(why) => {
                    // THE JOB STAYS. See the JobState::Unknown doc: inability to observe
                    // death is not evidence of death, and a row the shell cannot account
                    // for is EVIDENCE worth keeping visible.
                    println!(
                        "\n  {} [{}] {} -- {}",
                        "[?]".bright_yellow(),
                        job.id.to_string().bright_white(),
                        job.cmd.bright_white(),
                        format!("state unknown: {}", why).yellow()
                    );
                }
            }
        }
        self.jobs.retain(|j| !completed.contains(&j.id));
    }

    /// List all running jobs.
    pub fn list(&self) {
        println!();
        if self.jobs.is_empty() {
            println!("  {} No background jobs", "○".dimmed());
        } else {
            println!("  {}", "Background Jobs".bright_white().bold());
            println!("{}", "  ────────────────────────────────".dimmed());
            for job in &self.jobs {
                // ⭐ INT-188: THE GROUP IS SHOWN, NOT MERELY STORED.
                //
                // A pgid that nothing displays is a field nobody can check. This column is
                // how a user -- and a test -- confirms the job really left the shell's group,
                // by comparing it against `ps -o pid,pgid`.
                //
                // ⚠️ None prints as "no group", never as blank and never as the shell's own.
                // A job sharing the shell's group cannot be signalled without signalling the
                // shell, and a display that hides that would be the same class of lie INT-246
                // spent a session removing from the sandbox.
                let group = match job.pgid {
                    Some(p) => format!("pgid {}", p),
                    None => "no group".to_string(),
                };
                // THE STATE IS THE POINT OF THE TABLE. Without this column a stopped job and a
                // running one render IDENTICALLY -- which is what it looked like before INT-188
                // step 3, and the reason a user could not tell a job working from a job suspended.
                let state = match &job.state {
                    JobState::Running => "running".bright_green(),
                    JobState::Stopped => "stopped".bright_yellow(),
                    JobState::Done(JobResult::Exited(c)) => format!("exited {}", c).dimmed(),
                    JobState::Done(JobResult::Signaled(s)) => {
                        format!("killed by {}", s).bright_red()
                    }
                    // A row the shell cannot account for SAYS SO, in the place a user actually
                    // looks. This is the whole reason Unknown exists.
                    JobState::Unknown(why) => format!("UNKNOWN: {}", why).bright_yellow(),
                };
                println!(
                    "  [{}] {:<9} {}  ({}, {:.0}s elapsed)",
                    job.id.to_string().bright_cyan(),
                    state,
                    job.cmd.bright_white(),
                    group.dimmed(),
                    job.elapsed()
                );
            }
        }
        println!();
    }

    /// Bring job to foreground — wait for it.
    pub fn fg(&mut self, id: JobId) {
        let pos = self.jobs.iter().position(|j| j.id == id);
        match pos {
            None => println!("  {} No job [{}]", "✗".bright_red(), id),
            Some(i) => {
                let cmd = self.jobs[i].cmd.clone();
                println!(
                    "  {} [{}] {} (foreground)",
                    "→".bright_cyan(),
                    id,
                    cmd.dimmed()
                );
                // RESUME THE GROUP, THEN WAIT -- and give it the terminal first.
                //
                // This used to call child.wait(), which is wrong three ways: it waits on one
                // PROCESS rather than the job, it never SIGCONTs so a stopped job would hang
                // the shell forever, and it never hands over the terminal so the resumed job
                // could not read the keyboard.
                let pgid = self.jobs[i].pgid;
                if let Some(g) = pgid {
                    // SIGCONT to the GROUP -- the negative pid is what makes it a group.
                    unsafe {
                        libc::kill(-(g as i32), libc::SIGCONT);
                    }
                    crate::tty::give_terminal(g);
                }
                // Wait the way the foreground path does: a resumed job can be Ctrl+Z'd again.
                let stopped_again = if let Some(g) = pgid {
                    crate::tty::wait_group_foreground(g)
                } else {
                    false
                };
                // TAKE THE TERMINAL BACK ON EVERY PATH, including a second stop.
                crate::tty::take_terminal();
                if stopped_again {
                    self.jobs[i].state = JobState::Stopped;
                    if let Some(m) = self.jobs[i].members.first_mut() {
                        m.stopped = true;
                    }
                    println!(
                        "\n  {} [{}] {} -- {} ({})",
                        "\u{2638}",
                        id.to_string().bright_white(),
                        cmd.bright_white(),
                        "suspended again".bright_yellow(),
                        format!("fg {} to resume", id).dimmed()
                    );
                    return;
                }
                let elapsed = self.jobs[i].elapsed();
                println!(
                    "  {} [{}] {} — done ({:.1}s)",
                    "✅".normal(),
                    id,
                    cmd.bright_green(),
                    elapsed
                );
                self.jobs.remove(i);
            }
        }
    }

    /// `bg <id>` -- let a stopped job carry on IN THE BACKGROUND.
    ///
    /// ⭐ THE WHOLE DIFFERENCE FROM fg IS WHAT IT DOES NOT DO. One signal, then return:
    /// no tcsetpgrp, because the SHELL keeps the terminal, and no wait, because the point
    /// is to get the prompt back while the job runs.
    ///
    /// ⚠️ A BACKGROUND JOB THAT READS THE TERMINAL WILL STOP AGAIN, with SIGTTIN, and
    /// that is CORRECT Unix behaviour rather than a defect -- it is why `vim` in the
    /// background stops instead of fighting the shell for keystrokes. check_completed
    /// will observe the stop and say so.
    pub fn bg(&mut self, id: JobId) {
        let pos = self.jobs.iter().position(|j| j.id == id);
        let Some(i) = pos else {
            println!("  {} No job [{}]", "x".bright_red(), id);
            return;
        };
        // REFUSE RATHER THAN NO-OP. A job that is already running does not need resuming,
        // and silently sending it SIGCONT would look like success while answering nothing.
        if self.jobs[i].state != JobState::Stopped {
            println!(
                "  {} [{}] {} is not stopped -- nothing to resume",
                "x".bright_red(),
                id,
                self.jobs[i].cmd.dimmed()
            );
            return;
        }
        let Some(g) = self.jobs[i].pgid else {
            // No group means nothing to signal WITHOUT signalling the shell. Say so.
            println!(
                "  {} [{}] {} has no process group -- cannot be resumed safely",
                "x".bright_red(),
                id,
                self.jobs[i].cmd.dimmed()
            );
            return;
        };
        // The negative pid is what makes this a GROUP signal.
        unsafe {
            libc::kill(-(g as i32), libc::SIGCONT);
        }
        self.jobs[i].state = JobState::Running;
        for m in self.jobs[i].members.iter_mut() {
            m.stopped = false;
        }
        println!(
            "  {} [{}] {} -- {}",
            "\u{25b6}",
            id.to_string().bright_white(),
            self.jobs[i].cmd.bright_white(),
            "continued in the background".bright_cyan()
        );
    }

    /// Kill a job by id.
    pub fn kill_job(&mut self, id: JobId) {
        let pos = self.jobs.iter().position(|j| j.id == id);
        match pos {
            None => println!("  {} No job [{}]", "✗".bright_red(), id),
            Some(i) => {
                let cmd = self.jobs[i].cmd.clone();
                // KILL THE GROUP, AND REAP IT.
                //
                // child.kill() signalled ONE pid and the row was then removed without a wait,
                // which left a zombie -- observed 2026-09-13 as `sleep <defunct>`. A stopped
                // job also needs SIGCONT first: SIGTERM to a stopped process is queued, not
                // delivered, so without the continue it would sit there stopped and unkilled.
                if let Some(g) = self.jobs[i].pgid {
                    unsafe {
                        libc::kill(-(g as i32), libc::SIGCONT);
                        libc::kill(-(g as i32), libc::SIGTERM);
                    }
                }
                // Reap every handle we hold so nothing is left defunct.
                if let Some(c) = self.jobs[i].child.as_mut() {
                    let _ = c.wait();
                }
                for up in self.jobs[i].rest.iter_mut() {
                    let _ = up.wait();
                }
                println!("  {} [{}] {} killed", "○".dimmed(), id, cmd.dimmed());
                self.jobs.remove(i);
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    pub fn job_count(&self) -> usize {
        self.jobs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ THE DEFECT THIS INTENT ACTUALLY FIXED, asserted at the boundary where it lived.
    /// `fg` parsed with `unwrap_or(1)`, so `fg banana` became `fg 1` and foregrounded whatever job
    /// happened to be first. There is no plausible job to fall back to, so parsing returns None and
    /// the caller must say so.
    #[test]
    fn nonsense_is_not_quietly_a_job() {
        assert_eq!(JobId::parse("banana"), None);
        assert_eq!(JobId::parse(""), None);
        assert_eq!(JobId::parse("-3"), None);
        assert_eq!(
            JobId::parse("0"),
            None,
            "job ids start at 1, so zero is not one"
        );
    }

    /// ONE DEFINITION SHARED BY BOTH DOORS. `kill %2` and `fg 2` name the same job, and neither
    /// caller has to remember where the % is stripped.
    #[test]
    fn the_percent_form_and_the_bare_form_agree() {
        assert_eq!(JobId::parse("%2"), JobId::parse("2"));
        assert_eq!(JobId::parse(" %2 "), JobId::parse("2"));
        assert!(JobId::parse("2").is_some());
    }

    /// ⭐ G2: THE VALUES DO NOT CHANGE. This is a type change, not a renumbering -- a job that was
    /// 3 still displays as 3, and `jobs` output is untouched.
    #[test]
    fn a_job_id_still_displays_as_the_number_it_always_was() {
        let id = JobId::parse("3").expect("3 is a job id");
        assert_eq!(format!("{}", id), "3");
    }
}
