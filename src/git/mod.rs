//! Git operations module

mod init;
mod pr;
mod status;
mod workspace_mode;
mod workspace_repo;
mod worktree;

pub use init::{create_new_project, validate_new_project_path, InitProjectError};
pub use pr::{
    CheckState, CheckStatus, MergeReadiness, MergeableStatus, PrManager, PrPreflightResult,
    PrState, PrStatus, ReviewDecision,
};
pub use status::{GitDiffStats, RepositoryExposure};
pub use workspace_mode::WorkspaceMode;
pub use workspace_repo::WorkspaceRepoManager;
pub use worktree::{WorktreeInfo, WorktreeManager};
