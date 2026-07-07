// faelight-fm v3.1 -- shared types

#[derive(Debug, Clone, PartialEq)]
pub enum GitStatus {
    Clean,
    Modified,
    Untracked,
    Staged,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub git_status: GitStatus,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
    pub unlisted: usize, // count of items not shown
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct FlatNode {
    pub node_path: std::path::PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub git_status: GitStatus,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub depth: usize,
    pub unlisted: usize,
    pub is_unlisted_marker: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Mode {
    Normal,
    Filter(String),
    Command(String), // :verb mode
    ConfirmDelete(String),
}

#[derive(Debug, PartialEq)]
pub enum Panel {
    Left,
    Right,
}
