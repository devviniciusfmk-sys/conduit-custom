//! Axum web server implementation for Conduit.

use std::net::SocketAddr;

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    http::{header, Method},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use super::routes::api::api_routes;
use super::routes::static_files::{serve_index, serve_static_file};
use super::state::WebAppState;
use super::ws::handle_websocket;

/// Server configuration options.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host address to bind to.
    pub host: String,
    /// Port to listen on.
    pub port: u16,
    /// Enable CORS for development (allows any origin).
    pub cors_permissive: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors_permissive: true,
        }
    }
}

/// Health check response.
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

/// Health check endpoint handler.
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Agent types response.
#[derive(Serialize)]
struct AgentsResponse {
    agents: Vec<AgentInfo>,
}

#[derive(Serialize)]
struct AgentInfo {
    id: &'static str,
    name: &'static str,
    available: bool,
}

/// List available agents.
async fn list_agents(State(state): State<WebAppState>) -> Json<AgentsResponse> {
    use crate::util::Tool;

    let core = state.core().await;
    let tools = core.tools();

    Json(AgentsResponse {
        agents: vec![
            AgentInfo {
                id: "codex",
                name: "Codex CLI",
                available: tools.is_available(Tool::Codex),
            },
            AgentInfo {
                id: "claude",
                name: "Claude Code",
                available: tools.is_available(Tool::Claude),
            },
            AgentInfo {
                id: "gemini",
                name: "Gemini CLI",
                available: tools.is_available(Tool::Gemini),
            },
            AgentInfo {
                id: "opencode",
                name: "OpenCode",
                available: tools.is_available(Tool::Opencode),
            },
        ],
    })
}

/// WebSocket upgrade handler.
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<WebAppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        handle_websocket(socket, state.session_manager().clone()).await
    })
}

/// Build the Axum router with all routes.
fn build_router(state: WebAppState, cors_permissive: bool) -> Router {
    // Build CORS layer
    let cors = if cors_permissive {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
    } else {
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
    };

    // Core API routes (health, agents)
    let core_routes = Router::new()
        .route("/health", get(health))
        .route("/agents", get(list_agents));

    // Build main router combining core routes, REST API routes, and static files
    Router::new()
        .nest("/api", core_routes.merge(api_routes()))
        .route("/ws", get(ws_handler))
        // Static file routes for frontend assets
        .route("/assets/{*path}", get(serve_static_file))
        .route("/", get(serve_index))
        // Fallback to index.html for SPA routing
        .fallback(serve_index)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Run the web server.
///
/// This starts the Axum server and blocks until shutdown.
pub async fn run_server(state: WebAppState, config: ServerConfig) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    state.start_status_manager().await;
    let app = build_router(state, config.cors_permissive);

    tracing::info!("Starting web server at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::core::ConduitCore;
    use crate::util::ToolAvailability;
    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use tower::ServiceExt;

    fn init_test_data_dir() -> PathBuf {
        static TEST_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
        TEST_DATA_DIR
            .get_or_init(|| {
                let dir = tempfile::Builder::new()
                    .prefix("conduit-test-data-")
                    .tempdir()
                    .expect("Failed to create test data dir");
                let path = dir.path().to_path_buf();
                // Keep temp dir alive for test process lifetime.
                std::mem::forget(dir);
                crate::util::init_data_dir(Some(path.clone()));
                path
            })
            .clone()
    }

    fn test_state() -> WebAppState {
        init_test_data_dir();
        let config = Config::default();
        let tools = ToolAvailability::default();
        let core = ConduitCore::new(config, tools);
        WebAppState::new(core)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = test_state();
        let app = build_router(state, true);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_agents_endpoint() {
        let state = test_state();
        let app = build_router(state, true);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/agents")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let ids: Vec<&str> = json["agents"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item["id"].as_str())
            .collect();
        assert_eq!(ids, vec!["codex", "claude", "gemini", "opencode"]);
    }

    #[tokio::test]
    async fn test_list_repositories_endpoint() {
        let state = test_state();
        let app = build_router(state, true);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/repositories")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify response body structure
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("repositories").is_some());
    }

    #[tokio::test]
    async fn test_list_workspaces_endpoint() {
        let state = test_state();
        let app = build_router(state, true);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/workspaces")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify response body structure
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("workspaces").is_some());
    }

    #[tokio::test]
    async fn test_list_sessions_endpoint() {
        let state = test_state();
        let app = build_router(state, true);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/sessions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify response body structure
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("sessions").is_some());
    }

    #[tokio::test]
    async fn test_create_repository_endpoint() {
        let state = test_state();
        let app = build_router(state, true);

        let body = serde_json::json!({
            "name": "test-repo",
            "base_path": "/tmp/test-repo"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/repositories")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // Verify response body structure
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.get("name").and_then(|v| v.as_str()), Some("test-repo"));
        assert!(json.get("id").is_some());
    }

    #[tokio::test]
    async fn test_create_repository_validation() {
        let state = test_state();
        let app = build_router(state, true);

        // Missing both base_path and repository_url
        let body = serde_json::json!({
            "name": "test-repo"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/repositories")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_repository_not_found() {
        let state = test_state();
        let app = build_router(state, true);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/repositories/00000000-0000-0000-0000-000000000000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_session_endpoint() {
        let state = test_state();
        let app = build_router(state, true);

        let body = serde_json::json!({
            "agent_type": "claude",
            "model": "sonnet"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/sessions")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // Verify response body structure
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json.get("agent_type").and_then(|v| v.as_str()),
            Some("claude")
        );
        assert!(json.get("id").is_some());
    }

    #[tokio::test]
    async fn test_create_session_invalid_agent_type() {
        let state = test_state();
        let app = build_router(state, true);

        let body = serde_json::json!({
            "agent_type": "invalid"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/sessions")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// Create a git repository with one commit, using a repo-local identity so
    /// the test does not depend on the machine's git configuration.
    fn init_repo_with_commit(path: &std::path::Path) {
        let run = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(path)
                .output()
                .expect("git failed to run");
            assert!(
                out.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            );
        };

        run(&["init"]);
        run(&["config", "user.email", "test@test.com"]);
        run(&["config", "user.name", "Test"]);
        std::fs::write(path.join("README.md"), "# test\n").unwrap();
        run(&["add", "--", "README.md"]);
        run(&["commit", "-m", "Initial commit"]);
    }

    /// Register a repository plus a workspace backed by a real git worktree
    ///
    /// The worktree goes under `managed_dir`, which must live inside the test's
    /// own temp directory: the default location is a `worktrees/` folder beside
    /// the repository, which for temp repos means a path shared by every test.
    async fn seed_workspace_with_worktree(
        state: &WebAppState,
        repo_path: &std::path::Path,
        managed_dir: PathBuf,
    ) -> (uuid::Uuid, PathBuf) {
        use crate::data::{Repository, Workspace};
        use crate::git::{WorkspaceMode, WorkspaceRepoManager};

        let repo = Repository::from_local_path("test-repo", repo_path.to_path_buf());
        let worktree_path = WorkspaceRepoManager::with_managed_dir(managed_dir)
            .create_workspace(WorkspaceMode::Worktree, repo_path, "test/ws", "ws")
            .expect("failed to create worktree");
        let workspace = Workspace::new(repo.id, "ws", "test/ws", worktree_path.clone());
        let workspace_id = workspace.id;

        let core = state.core().await;
        core.repo_store().unwrap().create(&repo).unwrap();
        core.workspace_store().unwrap().create(&workspace).unwrap();

        (workspace_id, worktree_path)
    }

    #[tokio::test]
    async fn test_delete_workspace_removes_worktree_and_branch() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();
        init_repo_with_commit(repo_path);

        let state = test_state();
        let (workspace_id, worktree_path) =
            seed_workspace_with_worktree(&state, repo_path, dir.path().join("worktrees")).await;
        assert!(worktree_path.exists(), "worktree was not created");

        let app = build_router(state.clone(), true);
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/workspaces/{workspace_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // The directory must be gone, not just the database row
        assert!(
            !worktree_path.exists(),
            "worktree directory survived the delete: {}",
            worktree_path.display()
        );

        // The local branch must be gone too
        let branches = std::process::Command::new("git")
            .args(["branch", "--list", "test/ws"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        let branches = String::from_utf8_lossy(&branches.stdout);
        assert!(branches.trim().is_empty(), "branch survived: {branches}");

        // And the record must be gone from the database
        let core = state.core().await;
        let found = core
            .workspace_store()
            .unwrap()
            .get_by_id(workspace_id)
            .unwrap();
        assert!(found.is_none(), "workspace record survived the delete");
    }

    #[tokio::test]
    async fn test_delete_preflight_reports_uncommitted_work() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();
        init_repo_with_commit(repo_path);

        let state = test_state();
        let (workspace_id, worktree_path) =
            seed_workspace_with_worktree(&state, repo_path, dir.path().join("worktrees")).await;

        // Leave work behind that a forced delete would destroy
        std::fs::write(worktree_path.join("notes.txt"), "work in progress").unwrap();

        let app = build_router(state, true);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/workspaces/{workspace_id}/delete/preflight"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["is_dirty"].as_bool(), Some(true), "{json}");
        assert!(
            !json["warnings"].as_array().unwrap().is_empty(),
            "confirmation would have nothing to warn about: {json}"
        );
    }

    #[tokio::test]
    async fn test_delete_workspace_not_found() {
        let state = test_state();
        let app = build_router(state, true);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/workspaces/{}", uuid::Uuid::new_v4()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// POST /api/onboarding/create-project with the given body
    async fn create_project_request(body: serde_json::Value) -> (StatusCode, serde_json::Value) {
        let state = test_state();
        let app = build_router(state, true);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/onboarding/create-project")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    #[tokio::test]
    async fn test_create_project_endpoint() {
        let dir = tempfile::tempdir().unwrap();

        let (status, json) = create_project_request(serde_json::json!({
            "name": "my-app",
            "parent": dir.path().to_str().unwrap(),
        }))
        .await;

        let project_path = dir.path().join("my-app");

        // Without a git identity nothing can be committed; the handler reports
        // that as a bad request with an actionable message and creates nothing.
        if status == StatusCode::BAD_REQUEST {
            assert!(
                json["details"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("git config --global user.name"),
                "{json}"
            );
            assert!(!project_path.exists());
            return;
        }

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            json["repository"]["name"].as_str(),
            Some("my-app"),
            "{json}"
        );
        assert!(project_path.join(".git").exists());
        assert_eq!(
            std::fs::read_to_string(project_path.join("README.md")).unwrap(),
            "# my-app\n"
        );
    }

    #[tokio::test]
    async fn test_create_project_rejects_empty_name() {
        let dir = tempfile::tempdir().unwrap();

        let (status, _) = create_project_request(serde_json::json!({
            "name": "   ",
            "parent": dir.path().to_str().unwrap(),
        }))
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_create_project_rejects_missing_parent() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope");

        let (status, json) = create_project_request(serde_json::json!({
            "name": "my-app",
            "parent": missing.to_str().unwrap(),
        }))
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        // The real reason must reach the browser, not a generic message
        assert!(
            json["details"]
                .as_str()
                .unwrap_or_default()
                .contains("does not exist"),
            "{json}"
        );
    }

    #[tokio::test]
    async fn test_create_project_rejects_existing_target() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("my-app")).unwrap();

        let (status, json) = create_project_request(serde_json::json!({
            "name": "my-app",
            "parent": dir.path().to_str().unwrap(),
        }))
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(
            json["details"]
                .as_str()
                .unwrap_or_default()
                .contains("already exists"),
            "{json}"
        );
    }
}
