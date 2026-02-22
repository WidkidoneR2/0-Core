//! capabilities domain — query domain permission requirements

use crate::app::context::AppContext;
use crate::errors::CoreError;
use colored::Colorize;

struct DomainCaps {
    name: &'static str,
    caps: &'static [&'static str],
    description: &'static str,
}

const DOMAIN_MAP: &[DomainCaps] = &[
    DomainCaps {
        name: "doctor",
        caps: &["orchestrator.access", "filesystem.read.home"],
        description: "System health monitoring",
    },
    DomainCaps {
        name: "security",
        caps: &[
            "orchestrator.access",
            "filesystem.read.home",
            "network.query",
        ],
        description: "Security scanning and reporting",
    },
    DomainCaps {
        name: "git",
        caps: &["filesystem.read.home", "process.spawn"],
        description: "Git governance and risk scoring",
    },
    DomainCaps {
        name: "link",
        caps: &["filesystem.read.home"],
        description: "Symlink management",
    },
    DomainCaps {
        name: "zone",
        caps: &["filesystem.read.home"],
        description: "Zone detection",
    },
    DomainCaps {
        name: "intent",
        caps: &["filesystem.read.home"],
        description: "Intent ledger management",
    },
    DomainCaps {
        name: "profile",
        caps: &["filesystem.read.home", "filesystem.write.home"],
        description: "Profile switching",
    },
    DomainCaps {
        name: "sandbox",
        caps: &[
            "filesystem.read.home",
            "filesystem.write.home",
            "process.spawn",
        ],
        description: "Sandbox environment management",
    },
    DomainCaps {
        name: "fetch",
        caps: &["filesystem.read.home", "network.query"],
        description: "Network health checks",
    },
    DomainCaps {
        name: "workspace",
        caps: &["filesystem.read.home"],
        description: "Workspace and recent files",
    },
    DomainCaps {
        name: "release",
        caps: &[
            "filesystem.read.home",
            "filesystem.write.home",
            "process.spawn",
        ],
        description: "Release automation",
    },
    DomainCaps {
        name: "notify",
        caps: &["process.spawn"],
        description: "Desktop notifications",
    },
    DomainCaps {
        name: "lock",
        caps: &["sway.control"],
        description: "Screen locking",
    },
    DomainCaps {
        name: "launcher",
        caps: &["sway.control", "process.spawn"],
        description: "Application launcher",
    },
    DomainCaps {
        name: "update",
        caps: &["process.spawn", "privilege.elevated"],
        description: "System updates",
    },
    DomainCaps {
        name: "capabilities",
        caps: &[],
        description: "Capability introspection (this command)",
    },
];

pub fn list(_ctx: &AppContext, json: bool, domain: Option<&str>) -> Result<(), CoreError> {
    let domains: Vec<&DomainCaps> = if let Some(d) = domain {
        DOMAIN_MAP.iter().filter(|dm| dm.name == d).collect()
    } else {
        DOMAIN_MAP.iter().collect()
    };

    if domains.is_empty() {
        return Err(CoreError::Runtime(format!(
            "Unknown domain: {}",
            domain.unwrap_or("")
        )));
    }

    if json {
        let entries: Vec<serde_json::Value> = domains
            .iter()
            .map(|d| {
                serde_json::json!({
                    "domain": d.name,
                    "description": d.description,
                    "capabilities": d.caps,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&entries).unwrap_or_default()
        );
        return Ok(());
    }

    println!("{}", "🔒 Capability Requirements".cyan().bold());
    println!("{}", "━".repeat(41).dimmed());

    for d in &domains {
        println!();
        println!("  {} — {}", d.name.green().bold(), d.description.dimmed());
        if d.caps.is_empty() {
            println!("    {}", "no capabilities required".dimmed());
        } else {
            for cap in d.caps.iter() {
                println!("    {} {}", "·".dimmed(), cap.yellow());
            }
        }
    }
    println!();
    Ok(())
}
