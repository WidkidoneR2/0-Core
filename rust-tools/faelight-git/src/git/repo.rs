//! Git repository operations using git2-rs

use git2::Repository;
use anyhow::{Context, Result};
use std::path::Path;

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    /// Open repository at current directory
    pub fn open() -> Result<Self> {
        let repo = Repository::discover(".")
            .context("Failed to find git repository")?;
        Ok(Self { repo })
    }

    /// Open repository at specific path
    pub fn open_at(path: impl AsRef<Path>) -> Result<Self> {
        let repo = Repository::open(path.as_ref())
            .context("Failed to open git repository")?;
        Ok(Self { repo })
    }

    /// Get current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        Ok(head.shorthand()
            .unwrap_or("detached")
            .to_string())
    }

    /// Check if working tree is clean
    pub fn is_clean(&self) -> Result<bool> {
        let statuses = self.repo.statuses(None)?;
        Ok(statuses.is_empty())
    }

    /// Get working tree status
    pub fn status(&self) -> Result<WorkingTreeStatus> {
        let statuses = self.repo.statuses(None)?;
        
        let mut modified = 0;
        let mut untracked = 0;
        
        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_wt_modified() || status.is_wt_deleted() {
                modified += 1;
            }
            if status.is_wt_new() {
                untracked += 1;
            }
        }
        
        Ok(WorkingTreeStatus { modified, untracked })
    }

    /// Get upstream branch
    pub fn upstream(&self) -> Result<Option<String>> {
        let head = self.repo.head()?;
        let branch = git2::Branch::wrap(head);
        
        match branch.upstream() {
            Ok(upstream) => {
                Ok(upstream.name()?.map(String::from))
            }
            Err(_) => Ok(None),
        }
    }

    /// Count commits ahead/behind upstream
    pub fn ahead_behind(&self) -> Result<(usize, usize)> {
        let head = self.repo.head()?;
        let local_oid = head.target().context("No HEAD target")?;
        
        // Get upstream - if none exists, return (0, 0)
        let upstream_name = match self.upstream()? {
            Some(name) => name,
            None => return Ok((0, 0)),
        };
        
        // Try to find the upstream reference
        let upstream_ref = match self.repo.find_reference(&upstream_name) {
            Ok(r) => r,
            Err(_) => return Ok((0, 0)), // Upstream configured but not found
        };
        
        let upstream_oid = match upstream_ref.target() {
            Some(oid) => oid,
            None => return Ok((0, 0)),
        };
        
        let (ahead, behind) = self.repo.graph_ahead_behind(local_oid, upstream_oid)?;
        Ok((ahead, behind))
    }

    /// Get last commit hash
    pub fn last_commit_hash(&self) -> Result<String> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }
}

#[derive(Debug, Clone)]
pub struct WorkingTreeStatus {
    pub modified: usize,
    pub untracked: usize,
}
