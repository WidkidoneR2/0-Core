//! Git Risk Score engine

use crate::git::GitRepo;
use crate::is_core_locked;
use anyhow::Result;
use colored::*;

#[derive(Debug, Clone)]
pub struct RiskScore {
    pub total: u8,
    pub breakdown: Vec<RiskFactor>,
}

#[derive(Debug, Clone)]
pub struct RiskFactor {
    pub name: String,
    pub delta: i16,
    pub reason: String,
}

impl RiskScore {
    pub fn calculate(repo: &GitRepo) -> Result<Self> {
        let mut breakdown = Vec::new();
        let mut total: i16 = 0;

        // Factor 1: Dirty working tree
        let status = repo.status()?;
        if status.modified > 0 {
            let delta = 10;
            total += delta;
            breakdown.push(RiskFactor {
                name: "Dirty tree".into(),
                delta,
                reason: format!("{} modified files", status.modified),
            });
        }

        if status.untracked > 0 {
            let delta = 5;
            total += delta;
            breakdown.push(RiskFactor {
                name: "Untracked files".into(),
                delta,
                reason: format!("{} untracked files", status.untracked),
            });
        }

        // Factor 2: Core locked
        if is_core_locked() {
            let delta = 10;
            total += delta;
            breakdown.push(RiskFactor {
                name: "Core locked".into(),
                delta,
                reason: "0-core is locked".into(),
            });
        }

        // Factor 3: No upstream
        if repo.upstream()?.is_none() {
            let delta = 5;
            total += delta;
            breakdown.push(RiskFactor {
                name: "No upstream".into(),
                delta,
                reason: "No remote tracking branch".into(),
            });
        }

        // Factor 4: Commits ahead (unpushed)
        let (ahead, behind) = repo.ahead_behind()?;
        if ahead > 0 {
            let delta = (ahead as i16) * 2;
            total += delta;
            breakdown.push(RiskFactor {
                name: "Unpushed commits".into(),
                delta,
                reason: format!("{} commits ahead", ahead),
            });
        }

        if behind > 0 {
            let delta = 5;
            total += delta;
            breakdown.push(RiskFactor {
                name: "Behind upstream".into(),
                delta,
                reason: format!("{} commits behind", behind),
            });
        }

        // TODO: Add more factors:
        // - No snapshot since last commit (+20)
        // - No intent attached (+15)
        // - core-diff risk multiplier

        let total = total.clamp(0, 100) as u8;
        
        Ok(Self { total, breakdown })
    }

    pub fn band(&self) -> RiskBand {
        match self.total {
            0..=20 => RiskBand::Safe,
            21..=50 => RiskBand::Caution,
            _ => RiskBand::Dangerous,
        }
    }

    pub fn emoji(&self) -> &str {
        match self.band() {
            RiskBand::Safe => "🟢",
            RiskBand::Caution => "🟡",
            RiskBand::Dangerous => "🔴",
        }
    }

    pub fn color(&self) -> colored::Color {
        match self.band() {
            RiskBand::Safe => colored::Color::Green,
            RiskBand::Caution => colored::Color::Yellow,
            RiskBand::Dangerous => colored::Color::Red,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RiskBand {
    Safe,
    Caution,
    Dangerous,
}
