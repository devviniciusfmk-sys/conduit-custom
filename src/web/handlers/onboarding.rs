//! Onboarding handlers for the Conduit web API.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::data::{Repository, RepositoryStore};
use crate::web::error::WebError;
use crate::web::handlers::repositories::RepositoryResponse;
use crate::web::state::WebAppState;

const PROJECTS_BASE_DIR_KEY: &str = "projects_base_dir";

#[derive(Debug, Serialize)]
pub struct BaseDirResponse {
    pub base_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetBaseDirRequest {
    pub base_dir: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectEntryResponse {
    pub name: String,
    pub path: String,
    pub modified_at: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectsResponse {
    pub projects: Vec<ProjectEntryResponse>,
}

#[derive(Debug, Deserialize)]
pub struct AddProjectRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct AddProjectResponse {
    pub repository: RepositoryResponse,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    /// Project name, which becomes the folder name
    pub name: String,
    /// Folder the project directory is created in
    pub parent: String,
}

fn expand_path(raw: &str) -> PathBuf {
    if let Some(stripped) = raw.strip_prefix('~') {
        let stripped = stripped.trim_start_matches('/');
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }

    PathBuf::from(raw)
}

fn validate_dir(path: &Path) -> Result<(), WebError> {
    if !path.exists() {
        return Err(WebError::BadRequest("Directory does not exist".to_string()));
    }

    if !path.is_dir() {
        return Err(WebError::BadRequest("Path is not a directory".to_string()));
    }

    Ok(())
}

fn ensure_git_dir(path: &Path) -> Result<(), WebError> {
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        return Err(WebError::BadRequest(
            "Not a git repository (no .git directory)".to_string(),
        ));
    }
    Ok(())
}

pub async fn get_base_dir(
    State(state): State<WebAppState>,
) -> Result<Json<BaseDirResponse>, WebError> {
    let core = state.core().await;
    let store = core
        .app_state_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let base_dir = store.get(PROJECTS_BASE_DIR_KEY)?;
    Ok(Json(BaseDirResponse { base_dir }))
}

pub async fn set_base_dir(
    State(state): State<WebAppState>,
    Json(req): Json<SetBaseDirRequest>,
) -> Result<Json<BaseDirResponse>, WebError> {
    if req.base_dir.trim().is_empty() {
        return Err(WebError::BadRequest(
            "Base directory cannot be empty".to_string(),
        ));
    }

    let expanded = expand_path(req.base_dir.trim());
    validate_dir(&expanded)?;

    let core = state.core().await;
    let store = core
        .app_state_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    store.set(PROJECTS_BASE_DIR_KEY, req.base_dir.trim())?;

    Ok(Json(BaseDirResponse {
        base_dir: Some(req.base_dir.trim().to_string()),
    }))
}

pub async fn list_projects(
    State(state): State<WebAppState>,
) -> Result<Json<ProjectsResponse>, WebError> {
    let core = state.core().await;
    let store = core
        .app_state_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;
    let base_dir = store.get(PROJECTS_BASE_DIR_KEY)?;
    let base_dir = base_dir
        .ok_or_else(|| WebError::BadRequest("Projects directory is not set".to_string()))?;
    let base_path = expand_path(&base_dir);
    validate_dir(&base_path)?;

    let entries = std::fs::read_dir(&base_path)
        .map_err(|e| WebError::Internal(format!("Failed to read projects directory: {e}")))?;

    let mut projects = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if !path.join(".git").exists() {
            continue;
        }

        let modified_at = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let modified_at = chrono::DateTime::<chrono::Utc>::from(modified_at).to_rfc3339();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        projects.push(ProjectEntryResponse {
            name,
            path: path.to_string_lossy().to_string(),
            modified_at,
        });
    }

    projects.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(Json(ProjectsResponse { projects }))
}

/// Register a project path in the database
///
/// Returns the existing repository when the path is already known, so adding
/// the same project twice is a no-op instead of an error.
fn register_repository(
    repo_store: &RepositoryStore,
    config: &crate::config::Config,
    path: PathBuf,
    name: &str,
) -> Result<RepositoryResponse, WebError> {
    if let Some(existing) = repo_store
        .get_by_path(&path)
        .map_err(|e| WebError::Internal(format!("Failed to check repositories: {e}")))?
    {
        return Ok(RepositoryResponse::from_repo(existing, config));
    }

    let repo = Repository::from_local_path(name, path);
    repo_store
        .create(&repo)
        .map_err(|e| WebError::Internal(format!("Failed to create repository: {e}")))?;

    Ok(RepositoryResponse::from_repo(repo, config))
}

pub async fn add_project(
    State(state): State<WebAppState>,
    Json(req): Json<AddProjectRequest>,
) -> Result<Json<AddProjectResponse>, WebError> {
    if req.path.trim().is_empty() {
        return Err(WebError::BadRequest("Path cannot be empty".to_string()));
    }

    let expanded = expand_path(req.path.trim());
    validate_dir(&expanded)?;
    ensure_git_dir(&expanded)?;

    let core = state.core().await;
    let repo_store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;

    let name = expanded
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let repository = register_repository(repo_store, core.config(), expanded, &name)?;

    Ok(Json(AddProjectResponse { repository }))
}

/// Create a brand-new project on disk, then register it
///
/// Creates `parent/name`, runs `git init`, writes a README and records the
/// first commit — the same code path the TUI uses.
pub async fn create_project(
    State(state): State<WebAppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<AddProjectResponse>, WebError> {
    let name = req.name.trim().to_string();
    let parent_input = req.parent.trim().to_string();

    if name.is_empty() {
        return Err(WebError::BadRequest(
            "Project name cannot be empty".to_string(),
        ));
    }
    if parent_input.is_empty() {
        return Err(WebError::BadRequest(
            "Parent folder cannot be empty".to_string(),
        ));
    }

    let parent = expand_path(&parent_input);

    // Creating the project shells out to git, so keep it off the async threads.
    let created = {
        let name = name.clone();
        tokio::task::spawn_blocking(move || crate::git::create_new_project(&parent, &name))
            .await
            .map_err(|e| WebError::Internal(format!("Failed to create project: {e}")))?
    };

    // Every failure here is about the requested name/folder or the machine's git
    // setup, so surface the real message instead of a bare 500.
    let path = created.map_err(|e| WebError::BadRequest(e.to_string()))?;

    let core = state.core().await;
    let repo_store = core
        .repo_store()
        .ok_or_else(|| WebError::Internal("Database not available".to_string()))?;

    let repository = register_repository(repo_store, core.config(), path, &name)?;

    Ok(Json(AddProjectResponse { repository }))
}
