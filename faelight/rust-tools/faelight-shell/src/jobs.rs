//! jobs — background job control (Phase 8)
//! Supports: cmd &, jobs, fg N, bg N, kill %N
use colored::*;
use std::time::Instant;

#[derive(Debug)]
pub struct Job {
    pub id: usize,
    pub cmd: String,
    pub child: std::process::Child,
    pub started: Instant,
}

impl Job {
    pub fn elapsed(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }
}

pub struct JobTable {
    jobs: Vec<Job>,
    next_id: usize,
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
    pub fn spawn(&mut self, cmd: &str, args: &[String]) -> std::io::Result<usize> {
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
    ) -> std::io::Result<usize> {
        let child = command.spawn()?;
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.push(Job {
            id,
            cmd: label.to_string(),
            child,
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
    pub fn fg(&mut self, id: usize) {
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
    pub fn kill_job(&mut self, id: usize) {
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
