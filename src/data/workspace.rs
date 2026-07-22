//! Workspace data access object

use super::models::Workspace;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum RenameWorkspaceError {
    #[error("workspace not found")]
    NotFound,
    #[error("workspace name already exists in this repository")]
    Duplicate,
    #[error(transparent)]
    Database(#[from] rusqlite::Error),
}

/// Data access object for Workspace operations
#[derive(Clone)]
pub struct WorkspaceStore {
    conn: Arc<Mutex<Connection>>,
}

impl WorkspaceStore {
    /// Create a new WorkspaceDao
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Insert a new workspace
    pub fn create(&self, workspace: &Workspace) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO workspaces (id, repository_id, name, icon, color, branch, path, created_at, last_accessed, is_default)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                workspace.id.to_string(),
                workspace.repository_id.to_string(),
                workspace.name,
                workspace.icon,
                workspace.color,
                workspace.branch,
                workspace.path.to_string_lossy().to_string(),
                workspace.created_at.to_rfc3339(),
                workspace.last_accessed.to_rfc3339(),
                workspace.is_default as i32,
            ],
        )?;
        Ok(())
    }

    /// Get a workspace by ID
    pub fn get_by_id(&self, id: Uuid) -> SqliteResult<Option<Workspace>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, icon, color, branch, path, created_at, last_accessed, is_default, archived_at, archived_commit_sha
             FROM workspaces WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_workspace(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get all active (non-archived) workspaces for a repository
    pub fn get_by_repository(&self, repository_id: Uuid) -> SqliteResult<Vec<Workspace>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, icon, color, branch, path, created_at, last_accessed, is_default, archived_at, archived_commit_sha
             FROM workspaces WHERE repository_id = ?1 AND archived_at IS NULL ORDER BY is_default DESC, name",
        )?;

        let workspaces = stmt
            .query_map(params![repository_id.to_string()], |row| {
                Self::row_to_workspace(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(workspaces)
    }

    /// Count active (non-archived) workspaces for a repository
    pub fn count_active_by_repository(&self, repository_id: Uuid) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM workspaces WHERE repository_id = ?1 AND archived_at IS NULL",
            params![repository_id.to_string()],
            |row| row.get(0),
        )
    }

    /// Count all workspaces (including archived) for a repository
    pub fn count_all_by_repository(&self, repository_id: Uuid) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM workspaces WHERE repository_id = ?1",
            params![repository_id.to_string()],
            |row| row.get(0),
        )
    }

    /// Get ALL workspace names for a repository (including archived)
    ///
    /// Used for uniqueness checks to prevent resurrection of old workspace names.
    /// Unlike `get_by_repository`, this includes archived workspaces.
    pub fn get_all_names_by_repository(&self, repository_id: Uuid) -> SqliteResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT name FROM workspaces WHERE repository_id = ?1")?;

        let names = stmt
            .query_map(params![repository_id.to_string()], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(names)
    }

    /// Get all active (non-archived) workspaces
    pub fn get_all(&self) -> SqliteResult<Vec<Workspace>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, icon, color, branch, path, created_at, last_accessed, is_default, archived_at, archived_commit_sha
             FROM workspaces WHERE archived_at IS NULL ORDER BY repository_id, is_default DESC, name",
        )?;

        let workspaces = stmt
            .query_map([], Self::row_to_workspace)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(workspaces)
    }

    /// Update the last accessed timestamp
    pub fn update_last_accessed(&self, id: Uuid) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE workspaces SET last_accessed = ?2 WHERE id = ?1",
            params![id.to_string(), Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    /// Update a workspace
    pub fn update(&self, workspace: &Workspace) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE workspaces SET name = ?2, icon = ?3, color = ?4, branch = ?5, path = ?6, last_accessed = ?7, is_default = ?8
             WHERE id = ?1",
            params![
                workspace.id.to_string(),
                workspace.name,
                workspace.icon,
                workspace.color,
                workspace.branch,
                workspace.path.to_string_lossy().to_string(),
                workspace.last_accessed.to_rfc3339(),
                workspace.is_default as i32,
            ],
        )?;
        Ok(())
    }

    /// Rename only the display name of a workspace.
    pub fn rename(&self, id: Uuid, name: &str) -> Result<Workspace, RenameWorkspaceError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let repository_id: String = tx
            .query_row(
                "SELECT repository_id FROM workspaces WHERE id = ?1",
                params![id.to_string()],
                |row| row.get(0),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => RenameWorkspaceError::NotFound,
                other => RenameWorkspaceError::Database(other),
            })?;

        let duplicate: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM workspaces WHERE repository_id = ?1 AND name = ?2 AND id != ?3)",
            params![repository_id, name, id.to_string()],
            |row| row.get(0),
        )?;
        if duplicate {
            return Err(RenameWorkspaceError::Duplicate);
        }

        tx.execute(
            "UPDATE workspaces SET name = ?2 WHERE id = ?1",
            params![id.to_string(), name],
        )?;
        let workspace = tx.query_row(
            "SELECT id, repository_id, name, icon, color, branch, path, created_at, last_accessed, is_default, archived_at, archived_commit_sha FROM workspaces WHERE id = ?1",
            params![id.to_string()],
            Self::row_to_workspace,
        )?;
        tx.commit()?;
        Ok(workspace)
    }

    /// Update only a workspace's visual identity fields.
    pub fn update_identity(
        &self,
        id: Uuid,
        name: &str,
        icon: &str,
        color: &str,
    ) -> Result<Workspace, RenameWorkspaceError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let repository_id: String = tx
            .query_row(
                "SELECT repository_id FROM workspaces WHERE id = ?1",
                params![id.to_string()],
                |row| row.get(0),
            )
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => RenameWorkspaceError::NotFound,
                other => RenameWorkspaceError::Database(other),
            })?;
        let duplicate: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM workspaces WHERE repository_id = ?1 AND name = ?2 AND id != ?3)",
            params![repository_id, name, id.to_string()],
            |row| row.get(0),
        )?;
        if duplicate {
            return Err(RenameWorkspaceError::Duplicate);
        }
        tx.execute(
            "UPDATE workspaces SET name = ?2, icon = ?3, color = ?4 WHERE id = ?1",
            params![id.to_string(), name, icon, color],
        )?;
        let workspace = tx.query_row(
            "SELECT id, repository_id, name, icon, color, branch, path, created_at, last_accessed, is_default, archived_at, archived_commit_sha FROM workspaces WHERE id = ?1",
            params![id.to_string()],
            Self::row_to_workspace,
        )?;
        tx.commit()?;
        Ok(workspace)
    }

    /// Delete a workspace
    pub fn delete(&self, id: Uuid) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM workspaces WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    /// Check if a workspace exists by path
    pub fn exists_by_path(&self, path: &Path) -> SqliteResult<bool> {
        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy().to_string();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM workspaces WHERE path = ?1",
            params![path_str],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get a workspace by path
    pub fn get_by_path(&self, path: &Path) -> SqliteResult<Option<Workspace>> {
        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy().to_string();
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, icon, color, branch, path, created_at, last_accessed, is_default, archived_at, archived_commit_sha
             FROM workspaces WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path_str])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_workspace(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get the default workspace for a repository
    pub fn get_default_for_repository(
        &self,
        repository_id: Uuid,
    ) -> SqliteResult<Option<Workspace>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, icon, color, branch, path, created_at, last_accessed, is_default, archived_at, archived_commit_sha
             FROM workspaces WHERE repository_id = ?1 AND is_default = 1 AND archived_at IS NULL",
        )?;

        let mut rows = stmt.query(params![repository_id.to_string()])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_workspace(row)?))
        } else {
            Ok(None)
        }
    }

    /// Archive a workspace (soft delete - marks as archived and stores the branch SHA)
    pub fn archive(&self, id: Uuid, archived_commit_sha: Option<String>) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE workspaces SET archived_at = ?2, archived_commit_sha = ?3 WHERE id = ?1",
            params![id.to_string(), Utc::now().to_rfc3339(), archived_commit_sha],
        )?;
        Ok(())
    }

    /// Convert a database row to a Workspace
    fn row_to_workspace(row: &rusqlite::Row) -> SqliteResult<Workspace> {
        let id_str: String = row.get(0)?;
        let repo_id_str: String = row.get(1)?;
        let path_str: String = row.get(6)?;
        let created_at_str: String = row.get(7)?;
        let last_accessed_str: String = row.get(8)?;
        let is_default: i32 = row.get(9)?;
        let archived_at_str: Option<String> = row.get(10)?;
        let archived_commit_sha: Option<String> = row.get(11)?;

        Ok(Workspace {
            id: Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
            repository_id: Uuid::parse_str(&repo_id_str).unwrap_or_else(|_| Uuid::new_v4()),
            name: row.get(2)?,
            icon: row.get(3)?,
            color: row.get(4)?,
            branch: row.get(5)?,
            path: PathBuf::from(path_str),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_accessed: DateTime::parse_from_rfc3339(&last_accessed_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            is_default: is_default != 0,
            archived_at: archived_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            archived_commit_sha,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Database, Repository, RepositoryStore};
    use tempfile::tempdir;

    fn setup_db() -> (tempfile::TempDir, Database, RepositoryStore, WorkspaceStore) {
        let dir = tempdir().unwrap();
        let db = Database::open(dir.path().join("test.db")).unwrap();
        let repo_dao = RepositoryStore::new(db.connection());
        let ws_dao = WorkspaceStore::new(db.connection());
        (dir, db, repo_dao, ws_dao)
    }

    #[test]
    fn test_create_and_get() {
        let (_dir, _db, repo_dao, ws_dao) = setup_db();

        // Create a repository first
        let repo = Repository::from_local_path("test-repo", PathBuf::from("/tmp/test"));
        repo_dao.create(&repo).unwrap();

        // Create a workspace
        let ws = Workspace::new(
            repo.id,
            "main",
            "main",
            PathBuf::from("/tmp/test/worktrees/main"),
        );
        ws_dao.create(&ws).unwrap();

        let retrieved = ws_dao.get_by_id(ws.id).unwrap().unwrap();
        assert_eq!(retrieved.name, "main");
        assert_eq!(retrieved.branch, "main");
    }

    #[test]
    fn test_get_by_repository() {
        let (_dir, _db, repo_dao, ws_dao) = setup_db();

        let repo = Repository::from_local_path("test-repo", PathBuf::from("/tmp/test"));
        repo_dao.create(&repo).unwrap();

        let ws1 = Workspace::new_default(repo.id, "main", "main", PathBuf::from("/tmp/main"));
        let ws2 = Workspace::new(
            repo.id,
            "feature",
            "feature-branch",
            PathBuf::from("/tmp/feature"),
        );

        ws_dao.create(&ws1).unwrap();
        ws_dao.create(&ws2).unwrap();

        let workspaces = ws_dao.get_by_repository(repo.id).unwrap();
        assert_eq!(workspaces.len(), 2);
        assert!(workspaces[0].is_default); // Default comes first
    }

    #[test]
    fn test_cascade_delete() {
        let (_dir, _db, repo_dao, ws_dao) = setup_db();

        let repo = Repository::from_local_path("test-repo", PathBuf::from("/tmp/test"));
        repo_dao.create(&repo).unwrap();

        let ws = Workspace::new(repo.id, "main", "main", PathBuf::from("/tmp/main"));
        ws_dao.create(&ws).unwrap();

        // Delete repository should cascade to workspaces
        repo_dao.delete(repo.id).unwrap();

        let workspaces = ws_dao.get_by_repository(repo.id).unwrap();
        assert!(workspaces.is_empty());
    }

    #[test]
    fn rename_changes_only_name_and_preserves_sessions() {
        let (_dir, db, repo_dao, ws_dao) = setup_db();
        let repo = Repository::from_local_path("test-repo", PathBuf::from("/tmp/test"));
        repo_dao.create(&repo).unwrap();
        let ws = Workspace::new(
            repo.id,
            "wide-snow",
            "root/wide-snow",
            PathBuf::from("/tmp/wide-snow"),
        );
        ws_dao.create(&ws).unwrap();
        let session_id = Uuid::new_v4();
        db.connection().lock().unwrap().execute(
            "INSERT INTO session_tabs (id, tab_index, workspace_id, agent_type, created_at) VALUES (?1, 0, ?2, 'codex', ?3)",
            params![session_id.to_string(), ws.id.to_string(), Utc::now().to_rfc3339()],
        ).unwrap();

        let renamed = ws_dao.rename(ws.id, "Render Engine").unwrap();
        assert_eq!(renamed.name, "Render Engine");
        assert_eq!(renamed.branch, "root/wide-snow");
        assert_eq!(renamed.path, PathBuf::from("/tmp/wide-snow"));
        let session_workspace: String = db
            .connection()
            .lock()
            .unwrap()
            .query_row(
                "SELECT workspace_id FROM session_tabs WHERE id = ?1",
                params![session_id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(session_workspace, ws.id.to_string());
    }

    #[test]
    fn rename_rejects_duplicate_and_missing_workspace() {
        let (_dir, _db, repo_dao, ws_dao) = setup_db();
        let repo = Repository::from_local_path("test-repo", PathBuf::from("/tmp/test"));
        repo_dao.create(&repo).unwrap();
        let first = Workspace::new(repo.id, "first", "first", PathBuf::from("/tmp/first"));
        let second = Workspace::new(repo.id, "second", "second", PathBuf::from("/tmp/second"));
        ws_dao.create(&first).unwrap();
        ws_dao.create(&second).unwrap();
        assert!(matches!(
            ws_dao.rename(first.id, "second"),
            Err(RenameWorkspaceError::Duplicate)
        ));
        assert!(matches!(
            ws_dao.rename(Uuid::new_v4(), "missing"),
            Err(RenameWorkspaceError::NotFound)
        ));
    }

    #[test]
    fn update_identity_preserves_git_and_session_fields() {
        let (_dir, db, repo_dao, ws_dao) = setup_db();
        let repo = Repository::from_local_path("test-repo", PathBuf::from("/tmp/test"));
        repo_dao.create(&repo).unwrap();
        let ws = Workspace::new(
            repo.id,
            "backend",
            "root/backend",
            PathBuf::from("/tmp/backend"),
        );
        ws_dao.create(&ws).unwrap();
        let session_id = Uuid::new_v4();
        db.connection().lock().unwrap().execute(
            "INSERT INTO session_tabs (id, tab_index, workspace_id, agent_type, created_at) VALUES (?1, 0, ?2, 'codex', ?3)",
            params![session_id.to_string(), ws.id.to_string(), Utc::now().to_rfc3339()],
        ).unwrap();

        let updated = ws_dao
            .update_identity(ws.id, "Backend", "⚙️", "blue")
            .unwrap();
        assert_eq!(
            (
                updated.name.as_str(),
                updated.icon.as_str(),
                updated.color.as_str()
            ),
            ("Backend", "⚙️", "blue")
        );
        assert_eq!(updated.branch, "root/backend");
        assert_eq!(updated.path, PathBuf::from("/tmp/backend"));
        let session_count: i64 = db
            .connection()
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM session_tabs WHERE id = ?1 AND workspace_id = ?2",
                params![session_id.to_string(), ws.id.to_string()],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(session_count, 1);
    }
}
