use faelight_core::paths;
use std::fs;
use std::process::Command;
use std::time::Duration;
use sysinfo::{Disks, System};

pub struct SystemState {
    pub version:     String,
    pub profile:     String,
    pub core_state:  String,
    pub core_icon:   String,
    pub health:      String,
    pub health_icon: String,
    pub wm:          String,
    pub term:        String,
    pub shell:       String,
    pub kernel:      String,
    pub uptime:      String,
    pub hostname:    String,
    pub zone:        String,
    pub zone_icon:   String,
    // New fields
    pub cpu_usage:   String,
    pub memory:      String,
    pub disk:        String,
    pub commits:     String,
    pub tools:       String,
    pub rust_ver:    String,
}

impl SystemState {
    pub fn gather() -> Self {
        let (core_state, core_icon) = get_core_state();
        let (health, health_icon)   = get_health();
        let (zone, zone_icon)       = get_zone();
        let (cpu_usage, memory, disk) = get_resources();

        SystemState {
            version:    get_version(),
            profile:    get_profile(),
            core_state,
            core_icon,
            health,
            health_icon,
            wm:         get_wm(),
            term:       get_term(),
            shell:      get_shell(),
            kernel:     get_kernel(),
            uptime:     get_uptime(),
            hostname:   get_hostname(),
            zone,
            zone_icon,
            cpu_usage,
            memory,
            disk,
            commits:    get_commits(),
            tools:      get_tool_count(),
            rust_ver:   get_rust_version(),
        }
    }
}

fn get_version() -> String {
    fs::read_to_string(paths::version_file())
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn get_profile() -> String {
    fs::read_to_string(paths::profile_file())
        .unwrap_or_else(|_| "DEF".to_string())
        .trim()
        .to_string()
}

fn get_core_state() -> (String, String) {
    let output = Command::new("lsattr")
        .arg("-d")
        .arg(paths::core_dir())
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let parts: Vec<&str> = stdout.split_whitespace().collect();
            if let Some(attrs) = parts.first() {
                if attrs.contains('i') {
                    return ("locked".to_string(), crate::icons::LOCKED.to_string());
                }
            }
            ("unlocked".to_string(), crate::icons::UNLOCKED.to_string())
        }
        _ => ("unlocked".to_string(), crate::icons::UNLOCKED.to_string()),
    }
}

fn get_health() -> (String, String) {
    // Read from cache — instant, no doctor invocation
    let home = std::env::var("HOME").unwrap_or_default();
    let cache = std::path::PathBuf::from(&home)
        .join(".cache/faelight/health-status");

    let pct: u8 = fs::read_to_string(&cache)
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(0);

    let health_str = if pct == 0 {
        "?".to_string()
    } else {
        format!("{}%", pct)
    };

    let icon = if pct == 100 {
        crate::icons::HEALTHY
    } else if pct >= 90 {
        crate::icons::HEALTHY
    } else if pct >= 70 {
        crate::icons::WARNING
    } else {
        crate::icons::ERROR
    };

    (health_str, icon.to_string())
}

fn get_wm() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "sway".to_string())
}

fn get_term() -> String {
    // Walk process tree upward to find the terminal emulator
    // shell → faelight-term/foot/etc
    let skip = ["zsh", "bash", "sh", "fish", "nu", "faelight-fetch"];

    fn ppid_of(pid: u32) -> Option<u32> {
        std::fs::read_to_string(format!("/proc/{}/status", pid))
            .unwrap_or_default()
            .lines()
            .find(|l| l.starts_with("PPid:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok())
    }

    fn comm_of(pid: u32) -> String {
        std::fs::read_to_string(format!("/proc/{}/comm", pid))
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    let mut pid = std::process::id();
    for _ in 0..8 {
        if let Some(parent) = ppid_of(pid) {
            let comm = comm_of(parent);
            if !comm.is_empty() && !skip.contains(&comm.as_str()) && parent > 1 {
                return comm;
            }
            pid = parent;
        } else {
            break;
        }
    }

    std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string())
}

fn get_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| "unknown".to_string())
        .split('/')
        .next_back()
        .unwrap_or("unknown")
        .to_string()
}

fn get_kernel() -> String {
    System::kernel_version().unwrap_or_else(|| "unknown".to_string())
}

fn get_uptime() -> String {
    let seconds = System::uptime();
    format_duration(Duration::from_secs(seconds))
}

fn get_hostname() -> String {
    System::host_name().unwrap_or_else(|| "unknown".to_string())
}

fn get_resources() -> (String, String, String) {
    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU — average across all cores
    let cpu = sys.global_cpu_usage();
    let cpu_str = format!("{:.0}%", cpu);

    // Memory
    let used = sys.used_memory();
    let total = sys.total_memory();
    let mem_str = format!("{} / {}", format_bytes(used), format_bytes(total));

    // Disk — root filesystem
    let disks = Disks::new_with_refreshed_list();
    let disk_str = disks.iter()
        .find(|d| d.mount_point().to_str() == Some("/"))
        .map(|d| {
            let used = d.total_space() - d.available_space();
            let total = d.total_space();
            format!("{} / {}", format_bytes(used), format_bytes(total))
        })
        .unwrap_or_else(|| "?".to_string());

    (cpu_str, mem_str, disk_str)
}

fn get_commits() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let core = format!("{}/0-core", home);
    Command::new("git")
        .args(["-C", &core, "rev-list", "--count", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "?".to_string())
}

fn get_tool_count() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let tools_dir = format!("{}/0-core/rust-tools", home);
    std::fs::read_dir(&tools_dir)
        .map(|e| e.flatten().filter(|e| e.path().is_dir()).count().to_string())
        .unwrap_or_else(|_| "?".to_string())
}

fn get_rust_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .split_whitespace()
                .nth(1)
                .unwrap_or("?")
                .to_string()
        })
        .unwrap_or_else(|_| "?".to_string())
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.0}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.0}K", bytes as f64 / 1024.0)
    }
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days  = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let mins  = (total_seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn get_zone() -> (String, String) {
    use std::env;
    use std::path::PathBuf;

    let cwd  = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let home = PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/home".to_string()));

    let (zone_enum, _reason) = faelight_zone::current_zone(&cwd, &home);
    (zone_enum.short_label().to_string(), zone_enum.icon().to_string())
}
