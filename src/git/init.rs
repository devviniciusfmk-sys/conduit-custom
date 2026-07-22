//! Creating brand-new git projects from scratch
//!
//! Used by the "Create new project" tab of the add-project dialog: given a
//! parent folder and a project name, create the folder, `git init` it, write a
//! minimal README and record the first commit.

use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitProjectError {
    #[error("Project name cannot be empty")]
    EmptyName,
    #[error("Project name cannot contain '/' or '\\'")]
    NameHasSeparator,
    #[error("Project name is reserved")]
    ReservedName,
    #[error("Parent folder does not exist: {0}")]
    ParentNotFound(PathBuf),
    #[error("Parent path is not a directory: {0}")]
    ParentNotADirectory(PathBuf),
    #[error("Path already exists: {0}")]
    AlreadyExists(PathBuf),
    #[error(
        "Git has no identity configured, so it cannot record the first commit. Run: \
         git config --global user.name \"Your Name\" and \
         git config --global user.email \"you@example.com\""
    )]
    MissingGitIdentity,
    #[error("Git command failed: {0}")]
    CommandFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Validate a project name for use as a directory name
pub fn validate_project_name(name: &str) -> Result<(), InitProjectError> {
    if name.is_empty() {
        return Err(InitProjectError::EmptyName);
    }
    if name.contains('/') || name.contains('\\') {
        return Err(InitProjectError::NameHasSeparator);
    }
    if name == "." || name == ".." {
        return Err(InitProjectError::ReservedName);
    }
    Ok(())
}

/// Validate that `parent` can host a new project directory named `name`
///
/// Returns the path the project would be created at. Runs no side effects, so
/// dialogs can call it on every keystroke to show validation errors.
pub fn validate_new_project_path(parent: &Path, name: &str) -> Result<PathBuf, InitProjectError> {
    validate_project_name(name)?;

    if !parent.exists() {
        return Err(InitProjectError::ParentNotFound(parent.to_path_buf()));
    }
    if !parent.is_dir() {
        return Err(InitProjectError::ParentNotADirectory(parent.to_path_buf()));
    }

    let project_path = parent.join(name);
    if project_path.exists() {
        return Err(InitProjectError::AlreadyExists(project_path));
    }

    Ok(project_path)
}

/// Create a new git project inside `parent`
///
/// Creates `parent/name`, runs `git init`, writes a `README.md` containing the
/// project title and records an initial commit.
///
/// # Returns
/// Path to the created project
pub fn create_new_project(parent: &Path, name: &str) -> Result<PathBuf, InitProjectError> {
    let project_path = validate_new_project_path(parent, name)?;

    // Check before touching the disk: without an identity the commit would fail
    // at the very last step, and git's own message is easy to miss in a dialog.
    ensure_git_identity(parent)?;

    std::fs::create_dir(&project_path)?;

    // Everything past this point happens inside a directory we just created, so
    // clean it up on failure instead of leaving a half-initialized project that
    // would block a retry.
    match init_and_commit(&project_path, name) {
        Ok(()) => Ok(project_path),
        Err(e) => {
            let _ = std::fs::remove_dir_all(&project_path);
            Err(e)
        }
    }
}

/// Check that git knows who would author the commit
///
/// `git var GIT_COMMITTER_IDENT` resolves the identity from env vars and config
/// exactly like `git commit` does, and exits non-zero when it cannot.
fn ensure_git_identity(dir: &Path) -> Result<(), InitProjectError> {
    let output = Command::new("git")
        .args(["var", "GIT_COMMITTER_IDENT"])
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        return Err(InitProjectError::MissingGitIdentity);
    }

    Ok(())
}

/// Run `git init`, write the README and record the first commit
fn init_and_commit(project_path: &Path, name: &str) -> Result<(), InitProjectError> {
    run_git(project_path, &["init"])?;

    let readme = project_path.join("README.md");
    std::fs::write(&readme, format!("# {}\n", name))?;

    run_git(project_path, &["add", "--", "README.md"])?;
    run_git(project_path, &["commit", "-m", "Initial commit"])?;

    Ok(())
}

/// Run a git command inside `dir`, turning a non-zero exit into an error
fn run_git(dir: &Path, args: &[&str]) -> Result<(), InitProjectError> {
    let output = Command::new("git").args(args).current_dir(dir).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("git {} failed", args.join(" "))
        } else {
            stderr
        };
        return Err(InitProjectError::CommandFailed(message));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn rejects_invalid_names() {
        assert!(matches!(
            validate_project_name(""),
            Err(InitProjectError::EmptyName)
        ));
        assert!(matches!(
            validate_project_name("a/b"),
            Err(InitProjectError::NameHasSeparator)
        ));
        assert!(matches!(
            validate_project_name(".."),
            Err(InitProjectError::ReservedName)
        ));
        assert!(validate_project_name("my-app").is_ok());
    }

    #[test]
    fn rejects_missing_parent() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("nope");
        assert!(matches!(
            validate_new_project_path(&missing, "my-app"),
            Err(InitProjectError::ParentNotFound(_))
        ));
    }

    #[test]
    fn rejects_existing_target() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("my-app")).unwrap();
        assert!(matches!(
            validate_new_project_path(dir.path(), "my-app"),
            Err(InitProjectError::AlreadyExists(_))
        ));
    }

    /// Whether this machine can author a commit at all
    fn has_git_identity() -> bool {
        ensure_git_identity(Path::new(".")).is_ok()
    }

    #[test]
    fn creates_repo_with_readme_and_initial_commit() {
        if !has_git_identity() {
            // Covered by refuses_to_create_without_a_git_identity instead
            return;
        }

        let dir = tempdir().unwrap();
        let project_path = create_new_project(dir.path(), "my-app").unwrap();

        assert!(project_path.join(".git").exists());
        assert_eq!(
            std::fs::read_to_string(project_path.join("README.md")).unwrap(),
            "# my-app\n"
        );

        let log = Command::new("git")
            .args(["log", "--oneline"])
            .current_dir(&project_path)
            .output()
            .unwrap();
        let log = String::from_utf8_lossy(&log.stdout);
        assert!(log.contains("Initial commit"), "unexpected log: {log}");
    }

    #[test]
    fn refuses_to_create_without_a_git_identity() {
        if has_git_identity() {
            // Covered by creates_repo_with_readme_and_initial_commit instead
            return;
        }

        let dir = tempdir().unwrap();
        let result = create_new_project(dir.path(), "my-app");

        assert!(matches!(result, Err(InitProjectError::MissingGitIdentity)));
        // The check runs before anything is written
        assert!(!dir.path().join("my-app").exists());
    }

    #[test]
    fn missing_identity_error_explains_the_fix() {
        let message = InitProjectError::MissingGitIdentity.to_string();
        assert!(
            message.contains("git config --global user.name"),
            "{message}"
        );
        assert!(
            message.contains("git config --global user.email"),
            "{message}"
        );
    }

    #[test]
    fn init_and_commit_fails_when_readme_cannot_be_written() {
        let dir = tempdir().unwrap();
        let project_path = dir.path().join("my-app");

        // A directory where the README must go makes the write fail, without
        // depending on git's behaviour.
        std::fs::create_dir(&project_path).unwrap();
        std::fs::create_dir(project_path.join("README.md")).unwrap();

        assert!(matches!(
            init_and_commit(&project_path, "my-app"),
            Err(InitProjectError::Io(_))
        ));
    }
}
