use sysinfo::System;
use std::fs;
use std::process::Command;
use std::time::Duration;

pub struct SystemState {
    pub version: String,
    pub profile: String,
    pub core_state: String,
    pub core_icon: String,
    pub health: String,
    pub health_icon: String,
    pub wm: String,
    pub term: String,
    pub shell: String,
    pub kernel: String,
    pub uptime: String,
    pub hostname: String,
}

impl SystemState {
    pub fn gather() -> Self {
        SystemState {
            version: get_version(),
            profile: get_profile(),
            core_state: get_core_state().0,
            core_icon: get_core_state().1,
            health: get_health().0,
            health_icon: get_health().1,
            wm: get_wm(),
            term: get_term(),
            shell: get_shell(),
            kernel: get_kernel(),
            uptime: get_uptime(),
            hostname: get_hostname(),
        }
    }
}

fn get_version() -> String {
    fs::read_to_string("/home/christian/0-core/VERSION")
        .unwrap_or_else(|_| "8.0.0".to_string())
        .trim()
        .to_string()
}

fn get_profile() -> String {
    fs::read_to_string("/home/christian/.config/faelight/profile")
        .unwrap_or_else(|_| "DEF".to_string())
        .trim()
        .to_string()
}

fn get_core_state() -> (String, String) {
    // Check if 0-core directory has immutable attribute (chattr +i)
    let output = Command::new("lsattr")
        .arg("-d")
        .arg("/home/christian/0-core")
        .output();
    
    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            // lsattr output: "----i----------------- /home/christian/0-core"
            // Only check the first part (attributes), not the path!
            let parts: Vec<&str> = stdout.split_whitespace().collect();
            if let Some(attrs) = parts.first() {
                // Check if 'i' flag is in the attributes part
                if attrs.contains('i') {
                    return ("locked".to_string(), crate::icons::LOCKED.to_string());
                }
            }
            ("unlocked".to_string(), crate::icons::UNLOCKED.to_string())
        }
        _ => {
            // Fallback: assume unlocked if we can't check
            ("unlocked".to_string(), crate::icons::UNLOCKED.to_string())
        }
    }
}

fn get_health() -> (String, String) {
    let output = Command::new("dot-doctor")
        .output();
    
    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            
            for line in stdout.lines() {
                if line.contains("Health:") && !line.contains("Check") {
                    if let Some(health_part) = line.split(':').nth(1) {
                        let health = health_part.trim();
                        
                        let icon = if health == "100%" {
                            crate::icons::HEALTHY
                        } else if health.ends_with('%') {
                            if let Ok(pct) = health.trim_end_matches('%').parse::<u8>() {
                                if pct >= 90 {
                                    crate::icons::HEALTHY
                                } else if pct >= 70 {
                                    crate::icons::WARNING
                                } else {
                                    crate::icons::ERROR
                                }
                            } else {
                                crate::icons::WARNING
                            }
                        } else {
                            crate::icons::WARNING
                        };
                        
                        return (health.to_string(), icon.to_string());
                    }
                }
            }
            
            ("100%".to_string(), crate::icons::HEALTHY.to_string())
        }
        _ => ("?/?".to_string(), crate::icons::WARNING.to_string()),
    }
}

fn get_wm() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "sway".to_string())
}

fn get_term() -> String {
    std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string())
}

fn get_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| "unknown".to_string())
        .split('/')
        .last()
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

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
