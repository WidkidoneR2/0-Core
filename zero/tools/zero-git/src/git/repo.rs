//! Git repository operations using git2-rs

use anyhow::{Context, Result};
use git2::{Repository, Status};
use std::path::Path;

pub struct GitRepo {
    repo: Repository,
}

/// A single file's state in the working tree
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub state: FileState,
}

/// Every state a file can be in — staged, unstaged, or untracked
#[derive(Debug, Clone, PartialEq)]
pub enum FileState {
    /// Staged and ready to commit
    Staged,
    /// Modified but not staged
    Modified,
    /// Deleted but not staged
    Deleted,
    /// Staged as added (new file, staged)
    StagedNew,
    /// Staged as deleted
    StagedDeleted,
    /// Staged as renamed
    StagedRenamed(String), // "old -> new"
    /// Not tracked by git
    Untracked,
    /// Both staged and unstaged changes (partially staged)
    PartiallyStaged,
}

impl FileState {
    /// Single-character symbol shown in status column
    pub fn symbol(&self) -> &str {
        match self {
            FileState::Staged => "●",
            FileState::StagedNew => "●",
            FileState::StagedDeleted => "●",
            FileState::StagedRenamed(_) => "●",
            FileState::Modified => "○",
            FileState::Deleted => "○",
            FileState::PartiallyStaged => "◐",
            FileState::Untracked => "?",
        }
    }

    /// Short label shown next to the symbol
    pub fn label(&self) -> &str {
        match self {
            FileState::Staged => "staged",
            FileState::StagedNew => "new",
            FileState::StagedDeleted => "deleted",
            FileState::StagedRenamed(_) => "renamed",
            FileState::Modified => "modified",
            FileState::Deleted => "deleted",
            FileState::PartiallyStaged => "partial",
            FileState::Untracked => "untracked",
        }
    }

    pub fn is_staged(&self) -> bool {
        matches!(
            self,
            FileState::Staged
                | FileState::StagedNew
                | FileState::StagedDeleted
                | FileState::StagedRenamed(_)
                | FileState::PartiallyStaged
        )
    }
}

/// Full working tree status with actual file lists
#[derive(Debug, Clone)]
pub struct WorkingTreeStatus {
    pub files: Vec<FileEntry>,
    // Convenience counts — derived from files
    pub modified: usize,
    pub untracked: usize,
    pub staged: usize,
}

impl WorkingTreeStatus {
    pub fn staged_files(&self) -> Vec<&FileEntry> {
        self.files.iter().filter(|f| f.state.is_staged()).collect()
    }

    pub fn unstaged_files(&self) -> Vec<&FileEntry> {
        self.files
            .iter()
            .filter(|f| {
                matches!(
                    f.state,
                    FileState::Modified | FileState::Deleted | FileState::PartiallyStaged
                )
            })
            .collect()
    }

    pub fn untracked_files(&self) -> Vec<&FileEntry> {
        self.files
            .iter()
            .filter(|f| f.state == FileState::Untracked)
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// A single commit entry for the log view
#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub hash: String,      // short 7-char hash
    pub hash_full: String, // full hash
    pub author: String,
    pub time_ago: String,
    pub message: String,
}

impl GitRepo {
    /// Open repository at current directory or any parent
    pub fn open() -> Result<Self> {
        let repo = Repository::discover(".").context("Not inside a git repository")?;
        Ok(Self { repo })
    }

    /// Open repository at a specific path
    pub fn open_at(path: impl AsRef<Path>) -> Result<Self> {
        let repo = Repository::open(path.as_ref()).context("Failed to open git repository")?;
        Ok(Self { repo })
    }

    /// Current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        Ok(head.shorthand().unwrap_or("detached").to_string())
    }

    /// True if nothing staged or modified
    pub fn is_clean(&self) -> Result<bool> {
        let statuses = self.repo.statuses(None)?;
        Ok(statuses.is_empty())
    }

    /// Full working tree status — files with their exact states
    pub fn status(&self) -> Result<WorkingTreeStatus> {
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = self.repo.statuses(Some(&mut opts))?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("unknown").to_string();
            let s = entry.status();

            // Determine rename target if applicable
            let head_delta = entry.head_to_index().map(|d| {
                d.new_file()
                    .path()
                    .map(|p| p.to_string_lossy().into_owned())
            });

            let state = if s.contains(Status::INDEX_NEW) && s.contains(Status::WT_MODIFIED) {
                FileState::PartiallyStaged
            } else if s.contains(Status::INDEX_NEW) {
                FileState::StagedNew
            } else if s.contains(Status::INDEX_DELETED) {
                FileState::StagedDeleted
            } else if s.contains(Status::INDEX_RENAMED) {
                let new_name = head_delta.flatten().unwrap_or_else(|| path.clone());
                FileState::StagedRenamed(new_name)
            } else if s.contains(Status::INDEX_MODIFIED) && s.contains(Status::WT_MODIFIED) {
                FileState::PartiallyStaged
            } else if s.contains(Status::INDEX_MODIFIED) {
                FileState::Staged
            } else if s.contains(Status::WT_MODIFIED) {
                FileState::Modified
            } else if s.contains(Status::WT_DELETED) {
                FileState::Deleted
            } else if s.contains(Status::WT_NEW) {
                FileState::Untracked
            } else {
                continue; // skip anything we don't care about
            };

            files.push(FileEntry { path, state });
        }

        // Derive counts
        let staged = files.iter().filter(|f| f.state.is_staged()).count();
        let modified = files
            .iter()
            .filter(|f| {
                matches!(
                    f.state,
                    FileState::Modified | FileState::Deleted | FileState::PartiallyStaged
                )
            })
            .count();
        let untracked = files
            .iter()
            .filter(|f| f.state == FileState::Untracked)
            .count();

        Ok(WorkingTreeStatus {
            files,
            modified,
            untracked,
            staged,
        })
    }

    /// Upstream branch name
    pub fn upstream(&self) -> Result<Option<String>> {
        let head = self.repo.head()?;
        let branch = git2::Branch::wrap(head);
        match branch.upstream() {
            Ok(up) => Ok(up.name()?.map(String::from)),
            Err(_) => Ok(None),
        }
    }

    /// Commits ahead/behind upstream
    pub fn ahead_behind(&self) -> Result<(usize, usize)> {
        let head = self.repo.head()?;
        let local_oid = head.target().context("No HEAD target")?;

        let upstream_name = match self.upstream()? {
            Some(name) => name,
            None => return Ok((0, 0)),
        };

        let upstream_ref = match self.repo.find_reference(&upstream_name) {
            Ok(r) => r,
            Err(_) => return Ok((0, 0)),
        };

        let upstream_oid = match upstream_ref.target() {
            Some(oid) => oid,
            None => return Ok((0, 0)),
        };

        let (ahead, behind) = self.repo.graph_ahead_behind(local_oid, upstream_oid)?;
        Ok((ahead, behind))
    }

    /// Last commit short hash
    pub fn last_commit_hash(&self) -> Result<String> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string()[..7].to_string())
    }

    /// Commit log — returns up to `count` entries
    pub fn log(&self, count: usize) -> Result<Vec<CommitEntry>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut entries = Vec::new();

        for (i, oid) in revwalk.enumerate() {
            if i >= count {
                break;
            }
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;

            let hash_full = oid.to_string();
            let hash = hash_full[..7].to_string();
            let author = commit.author().name().unwrap_or("unknown").to_string();
            let message = commit.summary().unwrap_or("(no message)").to_string();

            // Human-readable time
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let commit_time = commit.time().seconds();
            let diff = now - commit_time;
            let time_ago = if diff < 60 {
                "just now".to_string()
            } else if diff < 3600 {
                format!("{} min ago", diff / 60)
            } else if diff < 86400 {
                format!("{} hr ago", diff / 3600)
            } else if diff < 604800 {
                format!("{} days ago", diff / 86400)
            } else {
                format!("{} wk ago", diff / 604800)
            };

            entries.push(CommitEntry {
                hash,
                hash_full,
                author,
                time_ago,
                message,
            });
        }

        Ok(entries)
    }

    /// Stage a single file by path
    pub fn stage_file(&self, path: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_path(Path::new(path))?;
        index.write()?;
        Ok(())
    }

    /// Stage all changes (equivalent to git add -A)
    pub fn stage_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    /// Unstage a single file
    pub fn unstage_file(&self, path: &str) -> Result<()> {
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo
            .reset_default(Some(head.as_object()), [path].iter())?;
        Ok(())
    }

    /// Create a commit with the given message
    pub fn commit(&self, message: &str) -> Result<String> {
        let sig = self.repo.signature()?;
        let mut index = self.repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;

        let parent_commit = self.repo.head()?.peel_to_commit();

        let oid = match parent_commit {
            Ok(parent) => self
                .repo
                .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?,
            Err(_) => {
                // Initial commit — no parent
                self.repo
                    .commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?
            }
        };

        Ok(oid.to_string()[..7].to_string())
    }

    /// Get diff stat for a specific commit hash
    pub fn diff_stat(&self, hash: &str) -> Result<String> {
        let oid = self.repo.revparse_single(hash)?.id();
        let commit = self.repo.find_commit(oid)?;
        let tree = commit.tree()?;

        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

        let diff = self
            .repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        let stats = diff.stats()?;
        Ok(format!(
            "{} files changed, {} insertions(+), {} deletions(-)",
            stats.files_changed(),
            stats.insertions(),
            stats.deletions()
        ))
    }
}
