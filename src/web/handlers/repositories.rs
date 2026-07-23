//! Repository handlers for the Conduit web API.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::core::resolve_repo_workspace_settings;
use crate::data::Repository;
use crate::git::WorkspaceMode;
use crate::web::error::WebError;
use crate::web::state::WebAppState;

/// Response for a single repository.
#[derive(Debug, Serialize)]
pub struct RepositoryResponse {
    pub id: Uuid,
    pub name: String,
    pub base_path: Option<String>,
    pub repository_url: Option<String>,
    pub workspace_mode: Option<WorkspaceMode>,
    pub workspace_mode_effective: WorkspaceMode,
    pub archive_delete_branch: Option<bool>,
    pub archive_delete_branch_effective: bool,
    pub archive_remote_prompt: Option<bool>,
    pub archive_remote_prompt_effective: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl RepositoryResponse {
    pub(crate) fn from_repo(repo: Repository, config: &crate::config::Config) -> Self {
        let settings = resolve_repo_workspace_settings(config, &repo);
        Self {
            id: repo.id,
            name: repo.name,
            base_path: repo.base_path.map(|p| p.to_string_lossy().to_string()),
            repository_url: repo.repository_url,
            workspace_mode: repo.workspace_mode,
            workspace_mode_effective: settings.mode,
            archive_delete_branch: repo.archive_delete_branch,
            archive_delete_branch_effective: settings.archive_delete_branch,
            archive_remote_prompt: repo.archive_remote_prompt,
            archive_remote_prompt_effective: settings.archive_remote_prompt,
            created_at: repo.created_at.to_rfc3339(),
            updated_at: repo.updated_at.to_rfc3339(),
        }
    }
}

/// Response for listing repositories.
#[derive(Debug, Serialize)]
pub struct ListRepositoriesResponse {
    pub repositories: Vec<RepositoryResponse>,
}

/// Request to create a new repository.
#[derive(Debug, Deserialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub base_path: Option<String>,
    pub repository_url: Option<String>,
}

/// Request to update repository workspace settings.
#[derive(Debug, Deserialize)]
pub struct UpdateRepositorySettingsRequest {
    pub workspace_mode: Option<WorkspaceMode>,
    pub archive_delete_branch: Option<bool>,
    pub archive_remote_prompt: Option<bool>,
}

/// List all repositories.
pub async fn list_repositories(
    State(state): State<WebAppState>,
) -> Result<Json<ListRepositoriesResponse>, WebError> {
    let core = state.core().await;
    let store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let config = core.config();

    let repos = store
        .get_all()
        .map_err(|e| WebError::Internal(format!("Failed to list repositories: {}", e)))?;

    Ok(Json(ListRepositoriesResponse {
        repositories: repos
            .into_iter()
            .map(|repo| RepositoryResponse::from_repo(repo, config))
            .collect(),
    }))
}

/// Get a single repository by ID.
pub async fn get_repository(
    State(state): State<WebAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RepositoryResponse>, WebError> {
    let core = state.core().await;
    let store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let config = core.config();

    let repo = store
        .get_by_id(id)
        .map_err(|e| WebError::Internal(format!("Failed to get repository: {}", e)))?
        .ok_or_else(|| WebError::NotFound(format!("Repository {} not found", id)))?;

    Ok(Json(RepositoryResponse::from_repo(repo, config)))
}

/// Create a new repository.
pub async fn create_repository(
    State(state): State<WebAppState>,
    Json(req): Json<CreateRepositoryRequest>,
) -> Result<(StatusCode, Json<RepositoryResponse>), WebError> {
    // Validate request
    if req.name.is_empty() {
        return Err(WebError::BadRequest(
            "Repository name is required".to_string(),
        ));
    }

    if req.base_path.is_none() && req.repository_url.is_none() {
        return Err(WebError::BadRequest(
            "Either base_path or repository_url is required".to_string(),
        ));
    }

    // Create repository model
    let repo = if let Some(path) = req.base_path {
        Repository::from_local_path(&req.name, PathBuf::from(path))
    } else if let Some(url) = req.repository_url {
        Repository::from_url(&req.name, url)
    } else {
        unreachable!()
    };

    // Save to database
    let core = state.core().await;
    let store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let config = core.config();

    store
        .create(&repo)
        .map_err(|e| WebError::Internal(format!("Failed to create repository: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(RepositoryResponse::from_repo(repo, config)),
    ))
}

/// Update repository workspace settings.
pub async fn update_repository_settings(
    State(state): State<WebAppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRepositorySettingsRequest>,
) -> Result<Json<RepositoryResponse>, WebError> {
    let core = state.core().await;
    let repo_store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let workspace_store = core
        .workspace_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;

    let repo = repo_store
        .get_by_id(id)
        .map_err(|e| WebError::Internal(format!("Failed to get repository: {}", e)))?
        .ok_or_else(|| WebError::NotFound(format!("Repository {} not found", id)))?;

    if let Some(mode) = req.workspace_mode {
        let active_count = workspace_store
            .count_active_by_repository(id)
            .map_err(|e| WebError::Internal(format!("Failed to check workspaces: {}", e)))?;

        if active_count > 0 && repo.workspace_mode != Some(mode) {
            return Err(WebError::Conflict(
                "workspace_mode_locked_active_workspaces".to_string(),
            ));
        }
    }

    let workspace_mode = req.workspace_mode.or(repo.workspace_mode);
    let archive_delete_branch = req.archive_delete_branch.or(repo.archive_delete_branch);
    let archive_remote_prompt = req.archive_remote_prompt.or(repo.archive_remote_prompt);

    repo_store
        .update_settings(
            id,
            workspace_mode,
            archive_delete_branch,
            archive_remote_prompt,
        )
        .map_err(|e| WebError::Internal(format!("Failed to update repository: {}", e)))?;

    let updated = repo_store
        .get_by_id(id)
        .map_err(|e| WebError::Internal(format!("Failed to load repository: {}", e)))?
        .ok_or_else(|| WebError::NotFound(format!("Repository {} not found", id)))?;

    Ok(Json(RepositoryResponse::from_repo(updated, core.config())))
}

/// Delete a repository.
/// Response for remove preflight checks.
#[derive(Debug, Serialize)]
pub struct RepositoryRemovePreflightResponse {
    pub repository_name: String,
    pub workspace_count: usize,
    pub warnings: Vec<String>,
    pub severity: String, // "info" | "warning" | "danger"
}

/// Preflight checks before removing a repository.
///
/// Returns information about workspaces that will be affected,
/// including warnings about uncommitted changes or unmerged branches.
pub async fn get_repository_remove_preflight(
    State(state): State<WebAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RepositoryRemovePreflightResponse>, WebError> {
    let core = state.core().await;
    let repo_store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let workspace_store = core
        .workspace_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;

    let repo = repo_store
        .get_by_id(id)
        .map_err(|e| WebError::Internal(format!("Failed to get repository: {}", e)))?
        .ok_or_else(|| WebError::NotFound(format!("Repository {} not found", id)))?;

    let workspaces = workspace_store
        .get_by_repository(id)
        .map_err(|e| WebError::Internal(format!("Failed to get workspaces: {}", e)))?;

    let worktree_manager = core.worktree_manager();
    let mut warnings = Vec::new();
    let mut has_dirty = false;
    let mut has_unmerged = false;

    // Add workspace count warning
    if !workspaces.is_empty() {
        warnings.push(format!(
            "{} workspace(s) will be archived",
            workspaces.len()
        ));
    }

    // Check git status for each workspace
    for ws in &workspaces {
        match worktree_manager.get_branch_status(&ws.path) {
            Ok(status) => {
                if status.is_dirty {
                    has_dirty = true;
                }
                if !status.is_merged {
                    has_unmerged = true;
                }
            }
            Err(e) => {
                tracing::warn!(
                    workspace_id = %ws.id,
                    error = %e,
                    "Failed to get git status for workspace during preflight"
                );
            }
        }
    }

    if has_dirty {
        warnings.push("Some workspaces have uncommitted changes".to_string());
    }

    if has_unmerged {
        warnings.push("Some branches are not merged to main".to_string());
    }

    // Determine severity
    let severity = if has_dirty && has_unmerged {
        "danger"
    } else if has_dirty || has_unmerged {
        "warning"
    } else {
        "info"
    };

    Ok(Json(RepositoryRemovePreflightResponse {
        repository_name: repo.name,
        workspace_count: workspaces.len(),
        warnings,
        severity: severity.to_string(),
    }))
}

/// Response for remove repository operation.
#[derive(Debug, Serialize)]
pub struct RepositoryRemoveResponse {
    pub success: bool,
    pub errors: Vec<String>,
}

/// Remove a repository and archive all its workspaces.
///
/// This mirrors the TUI's RemoveProject logic:
/// 1. For each workspace: get branch SHA, remove worktree, delete branch, archive in DB
/// 2. Delete the repository folder (with path safety checks)
/// 3. Delete the repository from DB
pub async fn remove_repository(
    State(state): State<WebAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RepositoryRemoveResponse>, WebError> {
    let core = state.core().await;
    let repo_store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let workspace_store = core
        .workspace_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let session_store = core
        .session_tab_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;

    let repo = repo_store
        .get_by_id(id)
        .map_err(|e| WebError::Internal(format!("Failed to get repository: {}", e)))?
        .ok_or_else(|| WebError::NotFound(format!("Repository {} not found", id)))?;

    let workspaces = workspace_store
        .get_by_repository(id)
        .map_err(|e| WebError::Internal(format!("Failed to get workspaces: {}", e)))?;

    let worktree_manager = core.worktree_manager();
    let settings = resolve_repo_workspace_settings(core.config(), &repo);
    let mut errors = Vec::new();

    // Process each workspace
    for ws in &workspaces {
        let mut archived_commit_sha = None;

        if let Some(ref base_path) = repo.base_path {
            // Get branch SHA
            match worktree_manager.get_branch_sha(settings.mode, base_path, &ws.path, &ws.branch) {
                Ok(sha) => {
                    archived_commit_sha = Some(sha);
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to read branch SHA for workspace '{}': {}",
                        ws.name, e
                    ));
                }
            }

            // Remove worktree
            if let Err(e) = worktree_manager.remove_workspace(settings.mode, base_path, &ws.path) {
                errors.push(format!("Failed to remove worktree '{}': {}", ws.name, e));
            }

            // Delete branch
            if let Err(e) =
                worktree_manager.delete_branch(settings.mode, base_path, &ws.path, &ws.branch)
            {
                errors.push(format!(
                    "Failed to delete branch '{}' for workspace '{}': {}",
                    ws.branch, ws.name, e
                ));
            }
        }

        // Archive workspace in DB
        if let Err(e) = workspace_store.archive(ws.id, archived_commit_sha) {
            errors.push(format!("Failed to archive workspace '{}': {}", ws.name, e));
        }

        // Close sessions for workspace
        if let Err(e) = session_store.set_open_by_workspace(ws.id, false) {
            tracing::warn!(error = %e, "Failed to close sessions for archived workspace");
        }

        // Remove from status manager
        state.status_manager().remove_workspace(ws.id);
    }

    // Delete repository folder (with path safety checks)
    let workspaces_dir = crate::util::workspaces_dir();
    if let Some(e) = crate::util::remove_project_workspaces_dir(&workspaces_dir, &repo.name) {
        errors.push(e);
    }

    // Delete repository from DB
    if let Err(e) = repo_store.delete(id) {
        errors.push(format!("Failed to delete repository from database: {}", e));
    }

    if !errors.is_empty() {
        tracing::warn!(
            repository_id = %id,
            errors = ?errors,
            "Repository removed with warnings"
        );
    }

    Ok(Json(RepositoryRemoveResponse {
        success: errors.is_empty(),
        errors,
    }))
}

// ===================== Permanent project deletion =====================
//
// This is the only operation in Conduit that deletes a folder it does not
// manage, so it is deliberately harder to reach than removing a project:
// the caller must type the project's name, the path is validated against a
// list of refusals, and the database record is dropped only after the folder
// is confirmed gone.

/// Preflight information for permanently deleting a project.
#[derive(Debug, Serialize)]
pub struct RepositoryDeletePreflightResponse {
    /// The name the user must type to confirm
    pub repository_name: String,
    /// Canonical folder that would be deleted
    pub project_path: Option<String>,
    pub workspace_count: usize,
    /// Whether any remote is configured
    pub has_remote: bool,
    /// Commits that exist on no remote
    pub unpushed_commits: usize,
    /// Registered projects living inside this folder
    pub nested_projects: Vec<String>,
    /// Set when deletion is refused outright; the UI must not offer to proceed
    pub blocked_reason: Option<String>,
    /// Whether the folder can be moved to the system trash instead of erased
    pub trash_available: bool,
    pub warnings: Vec<String>,
    pub severity: String,
}

/// Request to permanently delete a project.
#[derive(Debug, Deserialize)]
pub struct DeleteProjectRequest {
    /// Project name as typed by the user
    pub confirm_name: String,
    /// Erase the folder when the system trash is unavailable
    #[serde(default)]
    pub permanent: bool,
}

/// Result of permanently deleting a project.
#[derive(Debug, Serialize)]
pub struct DeleteProjectResponse {
    /// Whether the folder went to the trash rather than being erased
    pub moved_to_trash: bool,
    pub warnings: Vec<String>,
}

/// Check whether the system trash works for items on this folder's filesystem.
///
/// The only honest answer comes from trying, so this trashes a scratch file.
/// It is created in the parent directory to avoid leaving anything, however
/// briefly, inside the user's repository.
fn probe_trash(project_path: &std::path::Path) -> bool {
    let Some(parent) = project_path.parent() else {
        return false;
    };

    let probe = parent.join(".conduit-trash-probe");
    if std::fs::write(&probe, b"conduit").is_err() {
        return false;
    }

    match trash::delete(&probe) {
        Ok(()) => true,
        Err(_) => {
            // Never leave the scratch file behind
            let _ = std::fs::remove_file(&probe);
            false
        }
    }
}

/// Registered projects whose folder lives inside `root`.
fn nested_project_names(repos: &[Repository], root: &std::path::Path, skip: Uuid) -> Vec<String> {
    repos
        .iter()
        .filter(|candidate| candidate.id != skip)
        .filter(|candidate| {
            candidate
                .base_path
                .as_ref()
                .and_then(|path| std::fs::canonicalize(path).ok())
                .map(|path| path.starts_with(root))
                .unwrap_or(false)
        })
        .map(|candidate| candidate.name.clone())
        .collect()
}

/// Preflight checks before permanently deleting a project.
pub async fn get_repository_delete_preflight(
    State(state): State<WebAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RepositoryDeletePreflightResponse>, WebError> {
    let core = state.core().await;
    let repo_store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let workspace_store = core
        .workspace_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;

    let repo = repo_store
        .get_by_id(id)
        .map_err(|e| WebError::Internal(format!("Failed to get repository: {}", e)))?
        .ok_or_else(|| WebError::NotFound(format!("Repository {} not found", id)))?;

    let workspaces = workspace_store
        .get_by_repository(id)
        .map_err(|e| WebError::Internal(format!("Failed to get workspaces: {}", e)))?;

    let mut warnings = Vec::new();
    let mut blocked_reason = None;
    let mut project_path = None;
    let mut nested_projects = Vec::new();
    let mut exposure = crate::git::RepositoryExposure::default();
    let mut trash_available = false;

    match repo.base_path.as_ref() {
        Some(base_path) => {
            match crate::util::validate_project_root_for_deletion(
                base_path,
                dirs::home_dir().as_deref(),
                &crate::util::data_dir(),
            ) {
                Ok(canonical) => {
                    exposure = crate::git::RepositoryExposure::inspect(&canonical);
                    trash_available = probe_trash(&canonical);

                    let all_repos = repo_store.get_all().unwrap_or_default();
                    nested_projects = nested_project_names(&all_repos, &canonical, id);

                    project_path = Some(canonical.to_string_lossy().to_string());
                }
                Err(reason) => blocked_reason = Some(reason.to_string()),
            }
        }
        None => blocked_reason = Some("Project has no folder on disk".to_string()),
    }

    if !exposure.has_remote {
        warnings.push(
            "No remote is configured: the entire history exists only in this folder".to_string(),
        );
    }
    if exposure.unpushed_commits > 0 {
        warnings.push(format!(
            "{} commit(s) exist on no remote and cannot be recovered",
            exposure.unpushed_commits
        ));
    }
    if !workspaces.is_empty() {
        warnings.push(format!(
            "{} workspace(s) will be deleted with their branches",
            workspaces.len()
        ));
    }
    if !nested_projects.is_empty() {
        warnings.push(format!(
            "Contains other registered projects: {}",
            nested_projects.join(", ")
        ));
    }
    if !trash_available && blocked_reason.is_none() {
        warnings.push(
            "The system trash is unavailable here, so the folder would be erased".to_string(),
        );
    }

    let worktree_manager = core.worktree_manager();
    let dirty = workspaces
        .iter()
        .filter(|ws| {
            worktree_manager
                .get_branch_status(&ws.path)
                .map(|status| status.is_dirty)
                .unwrap_or(false)
        })
        .count();
    if dirty > 0 {
        warnings.push(format!("{dirty} workspace(s) have uncommitted changes"));
    }

    // Everything here destroys a folder, so "warning" is the floor; work that
    // exists nowhere else, or a folder that cannot be deleted at all, raises it.
    let irrecoverable =
        !exposure.has_remote || exposure.unpushed_commits > 0 || !nested_projects.is_empty();
    let severity = if blocked_reason.is_some() || irrecoverable {
        "danger"
    } else {
        "warning"
    };

    Ok(Json(RepositoryDeletePreflightResponse {
        repository_name: repo.name,
        project_path,
        workspace_count: workspaces.len(),
        has_remote: exposure.has_remote,
        unpushed_commits: exposure.unpushed_commits,
        nested_projects,
        blocked_reason,
        trash_available,
        warnings,
        severity: severity.to_string(),
    }))
}

/// Permanently delete a project: workspaces, branches, folder and record.
pub async fn delete_repository_permanently(
    State(state): State<WebAppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeleteProjectRequest>,
) -> Result<Json<DeleteProjectResponse>, WebError> {
    let core = state.core().await;
    let repo_store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let workspace_store = core
        .workspace_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let session_store = core
        .session_tab_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;

    let repo = repo_store
        .get_by_id(id)
        .map_err(|e| WebError::Internal(format!("Failed to get repository: {}", e)))?
        .ok_or_else(|| WebError::NotFound(format!("Repository {} not found", id)))?;

    // Typed confirmation, checked server-side as well: the browser is not the
    // only thing that can call this.
    if req.confirm_name.trim() != repo.name {
        return Err(WebError::BadRequest(
            "Typed name does not match the project name".to_string(),
        ));
    }

    let base_path = repo
        .base_path
        .as_ref()
        .ok_or_else(|| WebError::BadRequest("Project has no folder on disk".to_string()))?;

    let canonical = crate::util::validate_project_root_for_deletion(
        base_path,
        dirs::home_dir().as_deref(),
        &crate::util::data_dir(),
    )
    .map_err(|reason| WebError::Conflict(reason.to_string()))?;

    // Refuse rather than silently destroy a project the user registered
    // separately and would not see in this confirmation.
    let all_repos = repo_store
        .get_all()
        .map_err(|e| WebError::Internal(format!("Failed to list repositories: {}", e)))?;
    let nested = nested_project_names(&all_repos, &canonical, id);
    if !nested.is_empty() {
        return Err(WebError::Conflict(format!(
            "Folder contains other registered projects ({}); remove them first",
            nested.join(", ")
        )));
    }

    // Decide how the folder will go before anything is destroyed, so a missing
    // trash never silently escalates into an unrecoverable delete.
    let use_trash = if req.permanent {
        false
    } else {
        if !probe_trash(&canonical) {
            return Err(WebError::Conflict(
                "The system trash is unavailable for this folder. Confirm again to erase it \
                 permanently."
                    .to_string(),
            ));
        }
        true
    };

    let settings = resolve_repo_workspace_settings(core.config(), &repo);
    let workspaces = workspace_store
        .get_by_repository(id)
        .map_err(|e| WebError::Internal(format!("Failed to get workspaces: {}", e)))?;

    let mut warnings = Vec::new();

    for workspace in &workspaces {
        warnings.extend(super::workspaces::teardown_workspace_on_disk(
            core.worktree_manager(),
            settings.mode,
            base_path,
            workspace,
            super::workspaces::TeardownOptions {
                local_branch: true,
                remote_branch: false,
            },
        ));

        if let Err(e) = session_store.set_open_by_workspace(workspace.id, false) {
            tracing::warn!(error = %e, "Failed to close sessions for deleted workspace");
        }
        state.status_manager().remove_workspace(workspace.id);
    }

    // Conduit's own folder for this project, which lives outside the repository
    if let Some(e) =
        crate::util::remove_project_workspaces_dir(&crate::util::workspaces_dir(), &repo.name)
    {
        warnings.push(e);
    }

    let outcome = if use_trash {
        trash::delete(&canonical).map_err(|e| format!("Failed to move folder to trash: {e}"))
    } else {
        std::fs::remove_dir_all(&canonical).map_err(|e| format!("Failed to delete folder: {e}"))
    };

    if let Err(message) = outcome {
        return Err(WebError::Conflict(message));
    }

    // Keep the record when the folder survived, so the project stays visible
    // and the deletion can be retried instead of leaving an orphan on disk.
    if canonical.exists() {
        return Err(WebError::Conflict(format!(
            "Project folder could not be removed: {}",
            canonical.display()
        )));
    }

    repo_store
        .delete(id)
        .map_err(|e| WebError::Internal(format!("Failed to delete repository: {}", e)))?;

    if !warnings.is_empty() {
        tracing::warn!(
            repository_id = %id,
            warnings = ?warnings,
            "Project deleted with warnings"
        );
    }

    Ok(Json(DeleteProjectResponse {
        moved_to_trash: use_trash,
        warnings,
    }))
}
