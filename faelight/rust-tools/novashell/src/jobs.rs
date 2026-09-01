//! jobs — background job control (Phase 8)
//! Supports: cmd &, jobs, fg N, bg N, kill %N
use colored::*;
use std::time::Instant;

#[derive(Debug)]
pub struct Job {
    pub id: JobId,
    pub cmd: String,
    /// The status-bearing stage. For a pipeline this is the LAST one, per POSIX.
    pub child: std::process::Child,
    /// Upstream stages of a backgrounded pipeline, in stage order. Empty for a single
    /// command. Held so check_completed can reap them -- registering only the tail would
    /// leave every earlier stage a zombie.
    pub rest: Vec<std::process::Child>,
    pub started: Instant,
}

impl Job {
    pub fn elapsed(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
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
        let child = command.spawn()?;
        self.register_chain(vec![child], label)
    }

    /// Register an already-spawned CHAIN as one job. The last child carries the status;
    /// the rest are held so they are reaped rather than leaked.
    pub fn register_chain(
        &mut self,
        mut children: Vec<std::process::Child>,
        label: &str,
    ) -> std::io::Result<JobId> {
        let Some(child) = children.pop() else {
            return Err(std::io::Error::other("empty pipeline"));
        };
        let id = JobId(self.next_id);
        self.next_id += 1;
        self.jobs.push(Job {
            id,
            cmd: label.to_string(),
            child,
            rest: children,
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

    /// Check all jobs for completion — announce finished ones.
    /// Call this before every prompt render.
    pub fn check_completed(&mut self) {
        let mut completed = vec![];
        for job in &mut self.jobs {
            match job.child.try_wait() {
                Ok(Some(status)) => {
                    let elapsed = job.started.elapsed().as_secs_f64();
                    let code = status.code().unwrap_or(-1);
                    if code == 0 {
                        println!(
                            "\n  {} [{}] {} — {} ({:.1}s)",
                            "✅".normal(),
                            job.id.to_string().bright_white(),
                            job.cmd.bright_green(),
                            "done".bright_green(),
                            elapsed
                        );
                    } else {
                        println!(
                            "\n  {} [{}] {} — {} ({:.1}s) exit {}",
                            "✗".bright_red(),
                            job.id.to_string().bright_white(),
                            job.cmd.bright_red(),
                            "failed".bright_red(),
                            elapsed,
                            code
                        );
                    }
                    // Reap upstream stages before dropping the job -- their writer end is
                    // already closed, so they have exited or are about to.
                    for up in &mut job.rest {
                        let _ = up.wait();
                    }
                    completed.push(job.id);
                }
                Ok(None) => {} // still running
                Err(_) => {
                    completed.push(job.id);
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
                println!(
                    "  [{}] {}  ({:.0}s elapsed)",
                    job.id.to_string().bright_cyan(),
                    job.cmd.bright_white(),
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
                let _ = self.jobs[i].child.wait();
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

    /// Kill a job by id.
    pub fn kill_job(&mut self, id: JobId) {
        let pos = self.jobs.iter().position(|j| j.id == id);
        match pos {
            None => println!("  {} No job [{}]", "✗".bright_red(), id),
            Some(i) => {
                let cmd = self.jobs[i].cmd.clone();
                let _ = self.jobs[i].child.kill();
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
