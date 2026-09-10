//! Terminal ownership -- who the kernel delivers Ctrl+C and Ctrl+Z to.
//!
//! INT-188 step 4. ONE OWNER FOR EVERY tcsetpgrp. Scattering these calls is how a shell ends
//! up without a terminal and no one can say which call lost it.

use std::sync::atomic::{AtomicBool, Ordering};

static TTOU_IGNORED: AtomicBool = AtomicBool::new(false);

/// Is this shell interactive -- does it OWN a terminal it can hand over?
///
/// Job control is a property of a TERMINAL SESSION, not of a shell binary. `nsh -c`, a script,
/// and a piped stdin have no terminal to give away, so they must NEVER call tcsetpgrp. This is
/// the same split fish makes, and it is not a limitation: there is nothing to hand over.
///
/// THREE CONDITIONS, ALL REQUIRED:
///   - stdin is a tty            there IS a terminal
///   - we can read its foreground group   we are ALLOWED to ask
///   - that group is OURS         we are the one in charge of it right now
///
/// The third is the subtle one: nsh running AS a background job of another shell has a tty and
/// is not in charge of it. Handing the terminal around from there would fight the real owner.
pub fn is_interactive() -> bool {
    use rustix::termios::{isatty, tcgetpgrp};
    let stdin = rustix::stdio::stdin();
    if !isatty(stdin) {
        return false;
    }
    match tcgetpgrp(stdin) {
        Ok(fg) => fg == rustix::process::getpgrp(),
        // rustix refuses pid 0 here rather than handing back an invalid Pid -- it happens on
        // some ptys. NOT interactive is the safe reading: we could not establish ownership,
        // and acting as though we own the terminal on a failed question is how it gets lost.
        Err(_) => false,
    }
}

/// Ignore SIGTTOU, once per process.
///
/// WITHOUT THIS THE SHELL SUSPENDS ITSELF. A background process calling tcsetpgrp is sent
/// SIGTTOU by the kernel -- and the moment the shell hands the terminal to a child, the shell
/// IS a background process. Taking the terminal back would stop the shell stone dead.
pub fn ignore_ttou_once() {
    if TTOU_IGNORED.swap(true, Ordering::SeqCst) {
        return;
    }
    unsafe {
        libc::signal(libc::SIGTTOU, libc::SIG_IGN);
        // SIGTTIN for the same reason: a background read of the terminal.
        libc::signal(libc::SIGTTIN, libc::SIG_IGN);
    }
}

/// Give the terminal to a process group. Returns false if it could not be done.
///
/// Never called without is_interactive() first.
pub fn give_terminal(pgid: u32) -> bool {
    ignore_ttou_once();
    let Some(p) = rustix::process::Pid::from_raw(pgid as i32) else {
        return false;
    };
    rustix::termios::tcsetpgrp(rustix::stdio::stdin(), p).is_ok()
}

/// Take the terminal back for the shell.
///
/// ⚠️ CALL THIS ON EVERY EXIT PATH FROM A WAIT, NOT ONLY THE HAPPY ONE. A job that STOPS
/// leaves the shell without a terminal if the restore sits after a success branch -- the same
/// class as the cgroup cleanup that had to move into Drop in INT-246. The recovery from getting
/// this wrong is `exec bash` from another tty.
pub fn take_terminal() -> bool {
    ignore_ttou_once();
    rustix::termios::tcsetpgrp(rustix::stdio::stdin(), rustix::process::getpgrp()).is_ok()
}

/// Wait for a foreground child, RETURNING WHEN IT STOPS as well as when it dies.
///
/// Returns (status, stopped). When stopped is true the child is STILL ALIVE and the status is
/// a placeholder -- nothing may treat it as an exit code.
///
/// ⭐ THIS IS WHY Ctrl+Z RETURNS THE PROMPT. std::process::Child::wait() blocks until the
/// child TERMINATES. A stopped child never terminates, so the shell would sit in wait()
/// forever with the terminal handed away -- a hang, not a suspend. WUNTRACED is the flag that
/// makes a stop a RETURNABLE EVENT rather than silence.
///
/// ⚠️ EINTR IS NOT AN ANSWER. A signal arriving mid-wait is an interruption of the QUESTION,
/// not a reply to it. Retrying is the only honest response; treating it as a failure would let
/// the shell reclaim the terminal while the child is still using it.
pub fn wait_foreground(
    child: &mut std::process::Child,
) -> (std::io::Result<std::process::ExitStatus>, bool) {
    use rustix::process::{waitpid, Pid, WaitOptions};
    use std::os::unix::process::ExitStatusExt;

    let Some(pid) = Pid::from_raw(child.id() as i32) else {
        return (child.wait(), false);
    };

    loop {
        match waitpid(Some(pid), WaitOptions::UNTRACED) {
            Ok(Some(status)) => {
                if status.stopping_signal().is_some() {
                    // STOPPED. The child is alive. 148 is 128 + SIGTSTP, the conventional
                    // shell rendering -- but the bool is what callers must read.
                    return (Ok(std::process::ExitStatus::from_raw(148 << 8)), true);
                }
                if let Some(code) = status.exit_status() {
                    return (
                        Ok(std::process::ExitStatus::from_raw((code as i32) << 8)),
                        false,
                    );
                }
                if let Some(sig) = status.terminating_signal() {
                    return (Ok(std::process::ExitStatus::from_raw(sig as i32)), false);
                }
                return (Ok(std::process::ExitStatus::from_raw(0)), false);
            }
            // No options means this blocks; None should not happen, but retrying is safe.
            Ok(None) => continue,
            Err(e) if e.raw_os_error() == libc::EINTR => continue,
            Err(e) => {
                return (
                    Err(std::io::Error::from_raw_os_error(e.raw_os_error())),
                    false,
                );
            }
        }
    }
}
