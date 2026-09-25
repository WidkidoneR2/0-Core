//! cgroup v2 resource enforcement for sandbox policies.
//!
//! ⭐ INT-246. The policy engine declared four restrictions and enforced one. This is the module
//! that makes `max_memory_mb` real: a per-session cgroup under the delegated user slice, with the
//! child joined to it before it execs.
//!
//! ⚠️ MEASURED 2026-09-12 BEFORE ANY OF THIS WAS WRITTEN, and it changed the design:
//!
//!     memory.max = 32MB                        -> a 200MB allocation SUCCEEDED
//!     memory.max = 32MB, memory.swap.max = 0   -> Killed, rc=137, memory.events oom_kill 1
//!
//! cgroup v2 reclaims before it kills. With swap available a process over memory.max is SWAPPED
//! rather than stopped, so the cap behaves as a swap threshold. Both writes or the limit is not a
//! limit -- and a cap that writes a real kernel file and still lets the process run would be
//! WORSE than the decoration it replaced, because it would look verified.

use std::path::PathBuf;

/// Where this user's delegated cgroup subtree lives, if there is one.
///
/// Read from /proc/self/cgroup rather than assembled from a uid, because the shell may be nested
/// somewhere unexpected (a scope, a container) and the delegated root is wherever systemd put us,
/// not wherever a formula says it should be.
fn delegated_root() -> Option<PathBuf> {
    let own = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    // cgroup v2 is the "0::" line. Anything else is a v1 controller and not our business.
    let rel = own
        .lines()
        .find_map(|l| l.strip_prefix("0::"))?
        .trim()
        .to_string();
    let mut here = PathBuf::from("/sys/fs/cgroup").join(rel.trim_start_matches('/'));
    // Walk up to the nearest ancestor we may create children in. The leaf scope we are running
    // in is usually not writable by us, but the user@N.service above it is.
    loop {
        if here.join("cgroup.subtree_control").exists() {
            // ⚠⭐ subtree_control, NOT controllers. MEASURED 2026-09-12 after the first
            // wiring failed with "could not set memory.max: Permission denied".
            //
            //   cgroup.controllers      what this cgroup MAY use
            //   cgroup.subtree_control what its CHILDREN may use -- and therefore whether a
            //                          child has a memory.max file at all
            //
            // user@1000.service lists memory in controllers and NOT in subtree_control, so a
            // cgroup created there has no memory.max to write. app.slice, one level down, has
            // it in both. Reading the wrong file chose a directory where the cap could be
            // created and never applied -- which the report caught, and is exactly the failure
            // this intent exists to prevent.
            let ctrl =
                std::fs::read_to_string(here.join("cgroup.subtree_control")).unwrap_or_default();
            if ctrl.contains("memory") {
                // Probe rather than assume: delegation is a permission, not a declaration.
                let probe = here.join("zz-devbox-writable-probe");
                if std::fs::create_dir(&probe).is_ok() {
                    let _ = std::fs::remove_dir(&probe);
                    return Some(here);
                }
            }
        }
        if !here.pop() || here == PathBuf::from("/sys/fs/cgroup") {
            return None;
        }
    }
}

/// A cgroup this run owns, and removes when it is done.
pub struct Cgroup {
    path: PathBuf,
}

impl Cgroup {
    /// Create the cgroup and apply the limits a policy declares.
    ///
    /// Returns the cgroup and the list of DEGRADATIONS -- limits that were asked for and could
    /// not be applied. An empty list means every limit named is really in force.
    pub fn create(session_id: &str, memory_mb: Option<u64>) -> (Option<Self>, Vec<String>) {
        let mut degraded = Vec::new();

        if memory_mb.is_none() {
            return (None, degraded);
        }

        let Some(root) = delegated_root() else {
            degraded.push(
                "memory: no writable cgroup v2 subtree is delegated to this user, so no limit \
                 could be applied"
                    .to_string(),
            );
            return (None, degraded);
        };

        let path = root.join(format!("devbox-{}", session_id));
        if let Err(e) = std::fs::create_dir_all(&path) {
            degraded.push(format!(
                "memory: could not create cgroup {}: {}",
                path.display(),
                e
            ));
            return (None, degraded);
        }

        let cg = Cgroup { path };

        if let Some(mb) = memory_mb {
            let bytes = mb.saturating_mul(1024 * 1024);
            // ⚠️ BOTH WRITES OR NEITHER COUNTS. See the module header: memory.max alone is a
            // swap threshold. If swap.max fails the cap is not a memory cap, and saying so is
            // the whole point of this intent.
            match std::fs::write(cg.path.join("memory.max"), bytes.to_string()) {
                Ok(()) => match std::fs::write(cg.path.join("memory.swap.max"), "0") {
                    Ok(()) => {}
                    Err(e) => degraded.push(format!(
                        "memory: cap of {}MB is NOT enforced -- memory.swap.max could not be set \
                         to 0 ({}), so the process will be swapped rather than stopped",
                        mb, e
                    )),
                },
                Err(e) => degraded.push(format!("memory: could not set memory.max: {}", e)),
            }
        }

        (Some(cg), degraded)
    }

    /// The file a process writes its own pid into to join.
    pub fn procs_file(&self) -> PathBuf {
        self.path.join("cgroup.procs")
    }

    /// Did the kernel kill anything in here for exceeding memory?
    ///
    /// Read AFTER the child exits. A bare 137 tells a user their command was killed; this is how
    /// the report can say the cap was the reason.
    pub fn oom_killed(&self) -> bool {
        std::fs::read_to_string(self.path.join("memory.events"))
            .map(|s| {
                s.lines()
                    .find_map(|l| l.strip_prefix("oom_kill "))
                    .and_then(|n| n.trim().parse::<u64>().ok())
                    .unwrap_or(0)
                    > 0
            })
            .unwrap_or(false)
    }
}

impl Drop for Cgroup {
    /// ⚠️ CLEANUP ON THE PATH THAT ALWAYS EXECUTES. INT-204 records this trap for the test
    /// harness -- clean in Drop, not after the happy branch, because the happy branch is not
    /// where the files get left behind. rmdir only succeeds once the cgroup is empty, which it
    /// is by the time the child has been waited on.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.path);
    }
}
