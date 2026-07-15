//! faelight-vm -- snapshot/rollback for the forest's proving ground (INT-027).
//!
//! The START of the organic Rust migration: NEW capability built in Rust; the bash
//! script (INT-077/079) stays the front door and forwards these verbs here.
//!
//! TWO pieces of state must move together (INT-027, 2026-07-15):
//!   1. faelight-vm.qcow2         -- disk. Internal snapshots via `qemu-img snapshot`.
//!   2. faelight-vm-efi-vars.fd   -- OVMF EFI variables (raw; qemu-img cannot snapshot it).
//!      Since useEFIBoot landed, this holds boot entries and will hold Secure Boot keys
//!      (INT-059). A rollback restoring the disk but NOT the firmware vars is a LIE.
//! Both or neither.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const C_OK: &str = "\x1b[38;2;64;224;208m";
const C_ERR: &str = "\x1b[38;2;255;130;168m";
const C_DIM: &str = "\x1b[2m";
const C_RST: &str = "\x1b[0m";
const AUTO: &str = "auto-";
const PRUNE_DAYS: i64 = 14;

fn ok(m: &str) { println!("  {C_OK}✓{C_RST} {m}"); }
fn info(m: &str) { println!("  {C_DIM}→ {m}{C_RST}"); }
fn err(m: &str) -> ! { eprintln!("  {C_ERR}✗{C_RST} {m}"); std::process::exit(1); }

fn state_dir() -> PathBuf {
    std::env::var("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/state"))
        .join("faelight-vm")
}
fn disk() -> PathBuf { state_dir().join("faelight-vm.qcow2") }
fn efivars() -> PathBuf { state_dir().join("faelight-vm-efi-vars.fd") }
fn efivars_for(tag: &str) -> PathBuf { state_dir().join(format!("faelight-vm-efi-vars.fd.{tag}")) }

/// Scan /proc for a live faelight-vm qemu. The port check is a FALSE signal
/// (qemu binds the forward port before the guest boots) -- process truth only.
fn vm_pids() -> Vec<u32> {
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir("/proc") else { return out };
    for e in rd.flatten() {
        let name = e.file_name();
        let Some(pid) = name.to_str().and_then(|s| s.parse::<u32>().ok()) else { continue };
        let Ok(raw) = fs::read(e.path().join("cmdline")) else { continue };
        let cmd = String::from_utf8_lossy(&raw).replace('\0', " ");
        if cmd.contains("qemu-system") && cmd.contains("faelight-vm") { out.push(pid); }
    }
    out
}

fn require_down(action: &str) {
    let pids = vm_pids();
    if !pids.is_empty() {
        let l: Vec<String> = pids.iter().map(|p| p.to_string()).collect();
        err(&format!(
            "VM is RUNNING (PIDs: {}). Cannot {action} a live disk -- the image would tear.\n    Run: vm down",
            l.join(" ")
        ));
    }
}

fn require_disk() {
    if !disk().exists() {
        err(&format!("No VM disk at {}. Run: vm build && vm up", disk().display()));
    }
}

fn valid_tag(tag: &str, allow_auto: bool) -> Result<(), String> {
    if tag.is_empty() { return Err("empty tag".into()); }
    if !allow_auto && tag.starts_with(AUTO) {
        return Err(format!("'{AUTO}' is a reserved prefix (tool-made safety snapshots). Pick another tag."));
    }
    if !tag.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c)) {
        return Err(format!("bad tag '{tag}': use letters, digits, - _ . only"));
    }
    Ok(())
}

fn qemu_img(args: &[&str]) -> Result<String, String> {
    let out = Command::new("qemu-img").args(args).output()
        .map_err(|e| format!("qemu-img not runnable: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn backing_file() -> String {
    qemu_img(&["info", &disk().to_string_lossy()])
        .ok()
        .and_then(|s| s.lines().find(|l| l.starts_with("backing file:"))
            .map(|l| l.trim_start_matches("backing file:").trim().to_string()))
        .unwrap_or_else(|| "(none)".into())
}

/// (id, tag, "YYYY-MM-DD")
fn snapshots() -> Vec<(String, String, String)> {
    let Ok(out) = qemu_img(&["snapshot", "-l", &disk().to_string_lossy()]) else { return vec![] };
    let mut v = Vec::new();
    for line in out.lines() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t.len() < 3 || t[0] == "ID" || line.starts_with("Snapshot list") { continue }
        let date = t.iter().find(|s| s.len() == 10 && s.matches('-').count() == 2)
            .map(|s| s.to_string()).unwrap_or_else(|| "?".into());
        v.push((t[0].to_string(), t[1].to_string(), date));
    }
    v
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}
fn today_days() -> i64 {
    (SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) / 86400) as i64
}
fn age_days(date: &str) -> Option<i64> {
    let p: Vec<i64> = date.split('-').filter_map(|s| s.parse().ok()).collect();
    if p.len() != 3 { return None }
    Some(today_days() - days_from_civil(p[0], p[1], p[2]))
}

/// Snapshot BOTH state files. Atomic: if the EFI copy fails, undo the disk snapshot.
fn do_snapshot(tag: &str, auto: bool) {
    require_down("snapshot");
    require_disk();
    if let Err(e) = valid_tag(tag, auto) { err(&e); }
    if snapshots().iter().any(|(_, t, _)| t == tag) {
        err(&format!("snapshot '{tag}' already exists. Delete it first: vm delete {tag}"));
    }
    if let Err(e) = qemu_img(&["snapshot", "-c", tag, &disk().to_string_lossy()]) {
        err(&format!("disk snapshot failed: {e}"));
    }
    if efivars().exists() {
        if let Err(e) = fs::copy(efivars(), efivars_for(tag)) {
            let _ = qemu_img(&["snapshot", "-d", tag, &disk().to_string_lossy()]);
            err(&format!("EFI vars copy failed ({e}) -- disk snapshot rolled back.\n    Refusing a half-snapshot: firmware state must move with the disk."));
        }
    } else {
        info("no EFI vars file yet (VM never booted?) -- disk only");
    }
    ok(&format!("snapshot '{tag}' created {C_DIM}(disk + EFI vars){C_RST}"));
}

fn cmd_rollback(tag: &str) {
    require_down("rollback");
    require_disk();
    if !snapshots().iter().any(|(_, t, _)| t == tag) {
        err(&format!("no snapshot '{tag}'. See: vm snapshots"));
    }
    // Safety net, same pattern as cistart/cicomplete auto-checkpoints. Pruned by `vm prune`.
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let auto_tag = format!("{AUTO}pre-rollback-{stamp}");
    info(&format!("auto-snapshotting current state as '{auto_tag}'"));
    do_snapshot(&auto_tag, true);

    if let Err(e) = qemu_img(&["snapshot", "-a", tag, &disk().to_string_lossy()]) {
        err(&format!("disk rollback failed: {e}"));
    }
    let src = efivars_for(tag);
    if src.exists() {
        if let Err(e) = fs::copy(&src, efivars()) {
            err(&format!("DISK rolled back but EFI vars restore FAILED: {e}\n    State is now MIXED. Restore by hand: cp {} {}", src.display(), efivars().display()));
        }
    } else {
        info("no EFI vars saved for this tag -- firmware state left as-is");
    }
    ok(&format!("rolled back to '{tag}' {C_DIM}(disk + EFI vars){C_RST}"));
    info(&format!("undo: vm rollback {auto_tag}"));
}

fn cmd_list() {
    require_disk();
    let snaps = snapshots();
    println!("  {C_DIM}disk:    {}{C_RST}", disk().display());
    println!("  {C_DIM}backing: {}{C_RST}", backing_file());
    if snaps.is_empty() { info("no snapshots yet -- create one: vm snapshot <tag>"); return }
    println!("\n  {C_DIM}ID   TAG                              DATE         AGE   KIND{C_RST}");
    for (id, tag, date) in &snaps {
        let age = age_days(date).map(|d| format!("{d}d")).unwrap_or_else(|| "?".into());
        let kind = if tag.starts_with(AUTO) { format!("{C_DIM}auto{C_RST}") } else { "manual".into() };
        let efi = if efivars_for(tag).exists() { "" } else { "  ⚠ no EFI vars" };
        println!("  {id:<4} {tag:<32} {date}   {age:<5} {kind}{efi}");
    }
    println!("\n  {C_DIM}{} snapshot(s){C_RST}", snaps.len());
}

fn cmd_delete(tag: &str) {
    require_down("delete a snapshot on");
    require_disk();
    if !snapshots().iter().any(|(_, t, _)| t == tag) { err(&format!("no snapshot '{tag}'")); }
    if let Err(e) = qemu_img(&["snapshot", "-d", tag, &disk().to_string_lossy()]) {
        err(&format!("delete failed: {e}"));
    }
    let _ = fs::remove_file(efivars_for(tag));
    ok(&format!("deleted '{tag}'"));
}

/// Prune ONLY auto-* snapshots. Manual tags are deliberate -- never auto-removed.
fn cmd_prune(days: i64, all: bool, dry: bool) {
    require_disk();
    let victims: Vec<_> = snapshots().into_iter()
        .filter(|(_, t, _)| t.starts_with(AUTO))
        .filter(|(_, _, d)| all || age_days(d).map(|a| a >= days).unwrap_or(false))
        .collect();
    if victims.is_empty() {
        ok(&format!("nothing to prune {C_DIM}(auto-snapshots {}){C_RST}",
            if all { "none exist".into() } else { format!("all newer than {days}d") }));
        return;
    }
    if !dry { require_down("prune snapshots on"); }
    for (_, tag, date) in &victims {
        if dry {
            println!("  {C_DIM}would prune{C_RST} {tag} {C_DIM}({date}){C_RST}");
        } else if let Err(e) = qemu_img(&["snapshot", "-d", tag, &disk().to_string_lossy()]) {
            eprintln!("  {C_ERR}✗{C_RST} {tag}: {e}");
        } else {
            let _ = fs::remove_file(efivars_for(tag));
            ok(&format!("pruned {tag} {C_DIM}({date}){C_RST}"));
        }
    }
    if dry { info(&format!("{} would be pruned (dry run)", victims.len())); }
}

/// Is the GUEST's sshd actually accepting? (INT-027, 2026-07-15)
///
/// The bash `port_open` (>/dev/tcp/127.0.0.1/$PORT) is a LIE: qemu's user-mode
/// networking binds the host forward port the instant qemu starts, so it goes true
/// while the guest is still in OVMF. It proves qemu launched, not that the guest is up.
/// Since useBootLoader/useEFIBoot those are ~40s apart.
///
/// The honest test: read the SSH BANNER. Only a live sshd sends "SSH-2.0-...".
/// qemu with no guest listener accepts the TCP connection and closes it -- zero bytes.
fn ssh_ready(port: u16) -> bool {
    use std::io::Read;
    use std::net::{Shutdown, SocketAddr, TcpStream};
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let Ok(mut s) = TcpStream::connect_timeout(&addr, Duration::from_millis(800)) else {
        return false; // nothing listening at all
    };
    let _ = s.set_read_timeout(Some(Duration::from_millis(1200)));
    let mut buf = [0u8; 64];
    let n = s.read(&mut buf).unwrap_or(0);
    let _ = s.shutdown(Shutdown::Both);
    n > 0 && buf[..n].starts_with(b"SSH-")
}

/// Poll until the guest's sshd answers, or give up. Truth over speed.
fn cmd_wait_ready(port: u16, timeout_s: u64, quiet: bool) {
    let start = SystemTime::now();
    for i in 1..=timeout_s {
        if ssh_ready(port) {
            let secs = start.elapsed().map(|d| d.as_secs()).unwrap_or(i);
            if !quiet { ok(&format!("guest is UP {C_DIM}(sshd answered on port {port} after {secs}s){C_RST}")); }
            return;
        }
        if !quiet && i == 5 { info("waiting for the guest to boot (OVMF -> systemd-boot -> kernel -> sshd)..."); }
        std::thread::sleep(Duration::from_secs(1));
    }
    err(&format!(
        "guest did not answer ssh within {timeout_s}s.\n    The port may be bound by qemu while the guest is stuck -- check the console/log."
    ));
}

/// A process belonging to THIS VM -- qemu or any of its siblings (INT-159, 2026-07-15).
///
/// The bash script matched on the NAME `qemu-system-x86_64` alone. That is why a zombie swtpm
/// (pid 79844) survived `vm down`, inherited the launcher's lock fd, and held it invisibly:
/// swtpm is not qemu, so vm_pids could not see it, the janitor could not clean it, and vm debug
/// reported "qemu alive: 0 / lock HELD" -- the symptom with no way to learn more.
/// Scope is the STATE DIR in the cmdline, not the name: qemu carries file=<state>/faelight-vm.qcow2,
/// swtpm carries --tpmstate dir=<state>/faelight-vm-swtpm. Anything else in there is ours too.
struct VmProc {
    pid: u32,
    kind: String,
    cmd: String,
    holds_lock: bool,
}

fn lock_path() -> PathBuf { state_dir().join("vm.lock") }

/// Does this pid hold an fd on the launch lock? (An flock dies with its holder --
/// a HELD lock with no live holder is an orphaned file, safe to remove.)
fn holds_lock(pid: u32) -> bool {
    let Ok(target) = fs::canonicalize(lock_path()) else { return false };
    let Ok(rd) = fs::read_dir(format!("/proc/{pid}/fd")) else { return false };
    rd.flatten().any(|f| fs::read_link(f.path()).map(|l| l == target).unwrap_or(false))
}

fn vm_procs() -> Vec<VmProc> {
    let state = state_dir().to_string_lossy().to_string();
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir("/proc") else { return out };
    for e in rd.flatten() {
        let Some(pid) = e.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else { continue };
        let Ok(raw) = fs::read(e.path().join("cmdline")) else { continue };
        let cmd = String::from_utf8_lossy(&raw).replace('\0', " ").trim().to_string();
        if cmd.is_empty() || !cmd.contains(&state) { continue }
        let exe = fs::read_link(e.path().join("exe")).unwrap_or_default();
        let exe = exe.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        // CONTAINS, not starts_with: NixOS makeWrapper means the real ELF is
        // `.qemu-system-x86_64-wrapped` -- it starts with a DOT. starts_with("qemu-system")
        // made qemu INVISIBLE here while unwrapped swtpm matched fine (2026-07-15: `procs`
        // listed swtpm on a live VM and no qemu; `kill` would have orphaned the VM).
        // This is the banked forest rule: wrapped binaries need -f/substring matching, not
        // exact-name matching. `ss -ltnp` gave it away -- comm was ".qemu-system-x8".
        let kind = if exe.contains("qemu-system") { "qemu" }
                   else if exe.contains("swtpm") { "swtpm" }
                   else if cmd.contains("run-faelight-vm-vm") { "launcher" }
                   else { continue };
        out.push(VmProc { pid, kind: kind.into(), cmd, holds_lock: holds_lock(pid) });
    }
    out.sort_by_key(|p| p.pid);
    out
}

fn cmd_procs() {
    let procs = vm_procs();
    if procs.is_empty() {
        ok("no faelight-vm processes running");
        // NOTE: do NOT advise `vm unlock` just because vm.lock exists. An flock is released when
        // its holder DIES -- the FILE always survives. "file present, nobody holding" is the
        // NORMAL state after every clean `vm down`. Advising a fix here would fire every single
        // time and train the user to ignore the tool. `vm debug` tests the flock itself and
        // correctly reports "free". Only a live holder matters, and cmd_procs lists those above.
        return;
    }
    println!("  {C_DIM}PID      KIND      LOCK{C_RST}");
    for p in &procs {
        let l = if p.holds_lock { format!("{C_OK}HOLDS{C_RST}") } else { format!("{C_DIM}-{C_RST}") };
        println!("  {:<8} {:<9} {}", p.pid, p.kind, l);
        println!("      {C_DIM}{}{C_RST}", &p.cmd.chars().take(96).collect::<String>());
    }
    println!("\n  {C_DIM}{} process(es){C_RST}", procs.len());
}

/// Kill everything this VM spawned -- qemu AND swtpm AND the wrapper. Not by name.
fn cmd_kill() {
    let procs = vm_procs();
    if procs.is_empty() { ok("nothing to kill"); return }
    for p in &procs {
        let r = Command::new("kill").args(["-TERM", &p.pid.to_string()]).status();
        match r {
            Ok(s) if s.success() => ok(&format!("stopped {} (pid {})", p.kind, p.pid)),
            _ => eprintln!("  {C_ERR}✗{C_RST} failed to stop {} (pid {})", p.kind, p.pid),
        }
    }
    std::thread::sleep(Duration::from_millis(600));
    let left = vm_procs();
    if left.is_empty() { ok("all faelight-vm processes gone"); }
    else {
        for p in &left {
            let _ = Command::new("kill").args(["-KILL", &p.pid.to_string()]).status();
            info(&format!("SIGKILL sent to stubborn {} (pid {})", p.kind, p.pid));
        }
    }
}

/// The escape hatch that did not exist. Refuses if a LIVE process holds the lock.
fn cmd_unlock() {
    let holders: Vec<&VmProc> = Vec::new();
    let procs = vm_procs();
    let live: Vec<&VmProc> = procs.iter().filter(|p| p.holds_lock).collect();
    let _ = holders;
    if !live.is_empty() {
        for p in &live {
            eprintln!("  {C_ERR}✗{C_RST} pid {} ({}) still HOLDS the lock -- not a stale lock.", p.pid, p.kind);
        }
        err("refusing to unlock a live VM. Run: vm down");
    }
    if !lock_path().exists() { ok("no lock file -- nothing to clear"); return }
    match fs::remove_file(lock_path()) {
        Ok(_) => ok("orphaned lock cleared"),
        Err(e) => err(&format!("could not remove lock: {e}")),
    }
}

fn usage() -> ! {
    println!("faelight-vm -- snapshot/rollback for the proving ground (INT-027)

  vm snapshot <tag>     snapshot disk + EFI vars (VM must be down)
  vm rollback <tag>     restore a snapshot (auto-snapshots current state first)
  vm snapshots          list snapshots
  vm delete <tag>       delete a snapshot
  vm procs              list EVERY process this VM spawned (qemu, swtpm, launcher) + who
                        holds the launch lock -- scoped by state dir, not by process name
  vm kill               stop them ALL (qemu AND swtpm AND wrapper)
  vm unlock             clear an ORPHANED lock (refuses if a live process holds it)
  vm wait-ready         block until the GUEST's sshd answers (reads the SSH banner --
                        the port alone is a lie: qemu binds it before the guest boots)
                        [--port N] [--timeout N] [--quiet]
  vm prune [--days N]   remove auto-* snapshots older than N days (default {PRUNE_DAYS})
           [--all]      remove ALL auto-* regardless of age
           [--dry-run]  show what would go, touch nothing

  Snapshots move BOTH the disk and the OVMF EFI vars -- both or neither.
  Manual tags are never auto-pruned. '{AUTO}' is reserved for the tool.");
    std::process::exit(0)
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = a.iter().map(|s| s.as_str()).collect();
    match args.as_slice() {
        [] | ["-h"] | ["--help"] | ["help"] => usage(),
        ["snapshot", tag] | ["snap", tag] => do_snapshot(tag, false),
        ["rollback", tag] => cmd_rollback(tag),
        ["snapshots"] | ["list"] => cmd_list(),
        ["delete", tag] => cmd_delete(tag),
        ["wait-ready", rest @ ..] => {
            let port = rest.iter().position(|s| *s == "--port")
                .and_then(|i| rest.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(2222);
            let timeout = rest.iter().position(|s| *s == "--timeout")
                .and_then(|i| rest.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(120);
            cmd_wait_ready(port, timeout, rest.contains(&"--quiet"));
        }
        ["procs"] => cmd_procs(),
        ["kill"] => cmd_kill(),
        ["unlock"] => cmd_unlock(),
        ["prune", rest @ ..] => {
            let all = rest.contains(&"--all");
            let dry = rest.contains(&"--dry-run");
            let days = rest.iter().position(|s| *s == "--days")
                .and_then(|i| rest.get(i + 1)).and_then(|s| s.parse().ok())
                .unwrap_or(PRUNE_DAYS);
            cmd_prune(days, all, dry);
        }
        _ => err(&format!("unknown: {} -- see: vm help", args.join(" "))),
    }
}
