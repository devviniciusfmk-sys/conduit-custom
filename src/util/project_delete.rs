//! Safety checks for permanently deleting a project's root folder.
//!
//! Everything else Conduit removes lives under a directory it manages, so the
//! removal can simply require the target to sit inside that root. A project's
//! own folder has no such anchor: it is wherever the user's code happens to be,
//! so deleting it needs its own set of refusals. This module holds them as a
//! pure function, separate from any code that touches the disk, so the rules
//! can be tested exhaustively without deleting anything.

use std::path::{Component, Path, PathBuf};
use thiserror::Error;

/// Minimum number of named path components a deletable project must have.
///
/// Rejects `/repos` and `/home` while allowing `/tmp/work/my-repo`.
const MIN_PATH_DEPTH: usize = 2;

/// Why a path must not be deleted.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum UnsafeDeleteReason {
    #[error("Path does not exist: {0}")]
    NotFound(PathBuf),
    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),
    #[error("Path is a symlink, and deleting through it would be surprising: {0}")]
    Symlink(PathBuf),
    #[error("Not a git repository (no .git): {0}")]
    NotAGitRepo(PathBuf),
    #[error("Refusing to delete a filesystem root: {0}")]
    FilesystemRoot(PathBuf),
    #[error("Refusing to delete the home directory: {0}")]
    HomeDirectory(PathBuf),
    #[error("Refusing to delete a path this close to the filesystem root: {0}")]
    TooShallow(PathBuf),
    #[error("Refusing to delete a folder that holds Conduit's own data: {0}")]
    ConduitDataDir(PathBuf),
    #[error("Failed to inspect path: {0}")]
    Io(String),
}

/// Check whether `path` may be deleted as a project root.
///
/// Returns the canonical path to delete, or the reason it must not be touched.
/// Runs no side effects.
///
/// # Arguments
/// * `path` - the project's recorded base path
/// * `home` - the user's home directory, if known
/// * `data_dir` - Conduit's data directory (`~/.conduit` by default)
pub fn validate_project_root_for_deletion(
    path: &Path,
    home: Option<&Path>,
    data_dir: &Path,
) -> Result<PathBuf, UnsafeDeleteReason> {
    // Check the link itself before resolving it: deleting "through" a symlink
    // would remove the target the user never named.
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            return Err(UnsafeDeleteReason::Symlink(path.to_path_buf()));
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(UnsafeDeleteReason::NotFound(path.to_path_buf()));
        }
        Err(e) => return Err(UnsafeDeleteReason::Io(e.to_string())),
    }

    if !path.is_dir() {
        return Err(UnsafeDeleteReason::NotADirectory(path.to_path_buf()));
    }

    // Resolve `..`, symlinked parents and relative paths before comparing
    // against the paths that must never be deleted.
    let canonical =
        std::fs::canonicalize(path).map_err(|e| UnsafeDeleteReason::Io(e.to_string()))?;

    let depth = canonical
        .components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .count();

    if depth == 0 {
        return Err(UnsafeDeleteReason::FilesystemRoot(canonical));
    }

    if let Some(home) = home {
        if let Ok(canonical_home) = std::fs::canonicalize(home) {
            // Both the home directory itself and anything containing it
            // (`/home`, `/Users`) are off limits.
            if canonical_home.starts_with(&canonical) {
                return Err(UnsafeDeleteReason::HomeDirectory(canonical));
            }
        }
    }

    if depth < MIN_PATH_DEPTH {
        return Err(UnsafeDeleteReason::TooShallow(canonical));
    }

    // Deleting a folder that contains the database would take Conduit's own
    // state with it; deleting one inside it means the caller picked the wrong
    // removal path entirely.
    if let Ok(canonical_data) = std::fs::canonicalize(data_dir) {
        if canonical_data.starts_with(&canonical) || canonical.starts_with(&canonical_data) {
            return Err(UnsafeDeleteReason::ConduitDataDir(canonical));
        }
    }

    // A project without .git is not the repository we think it is - refuse
    // rather than delete an arbitrary folder someone registered by mistake.
    if !canonical.join(".git").exists() {
        return Err(UnsafeDeleteReason::NotAGitRepo(canonical));
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// A directory that looks like a git repository
    fn make_repo(parent: &Path, name: &str) -> PathBuf {
        let path = parent.join(name);
        std::fs::create_dir_all(path.join(".git")).unwrap();
        path
    }

    fn data_dir(parent: &Path) -> PathBuf {
        let path = parent.join("conduit-data");
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn accepts_a_normal_project() {
        let temp = tempdir().unwrap();
        let repo = make_repo(temp.path(), "my-repo");

        let result = validate_project_root_for_deletion(&repo, None, &data_dir(temp.path()));

        assert_eq!(result.unwrap(), std::fs::canonicalize(&repo).unwrap());
    }

    #[test]
    fn rejects_missing_path() {
        let temp = tempdir().unwrap();
        let missing = temp.path().join("nope");

        assert!(matches!(
            validate_project_root_for_deletion(&missing, None, &data_dir(temp.path())),
            Err(UnsafeDeleteReason::NotFound(_))
        ));
    }

    #[test]
    fn rejects_a_file() {
        let temp = tempdir().unwrap();
        let file = temp.path().join("a-file");
        std::fs::write(&file, "x").unwrap();

        assert!(matches!(
            validate_project_root_for_deletion(&file, None, &data_dir(temp.path())),
            Err(UnsafeDeleteReason::NotADirectory(_))
        ));
    }

    #[test]
    fn rejects_a_directory_without_git() {
        let temp = tempdir().unwrap();
        let plain = temp.path().join("just-a-folder");
        std::fs::create_dir_all(&plain).unwrap();

        assert!(matches!(
            validate_project_root_for_deletion(&plain, None, &data_dir(temp.path())),
            Err(UnsafeDeleteReason::NotAGitRepo(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlink_to_a_repository() {
        let temp = tempdir().unwrap();
        let repo = make_repo(temp.path(), "real-repo");
        let link = temp.path().join("link-to-repo");
        std::os::unix::fs::symlink(&repo, &link).unwrap();

        assert!(matches!(
            validate_project_root_for_deletion(&link, None, &data_dir(temp.path())),
            Err(UnsafeDeleteReason::Symlink(_))
        ));
        // The real repository must be untouched by the check
        assert!(repo.exists());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_filesystem_root() {
        let temp = tempdir().unwrap();

        assert!(matches!(
            validate_project_root_for_deletion(Path::new("/"), None, &data_dir(temp.path())),
            Err(UnsafeDeleteReason::FilesystemRoot(_))
        ));
    }

    #[test]
    fn rejects_the_home_directory() {
        let temp = tempdir().unwrap();
        let home = make_repo(temp.path(), "home-dir");

        assert!(matches!(
            validate_project_root_for_deletion(&home, Some(&home), &data_dir(temp.path())),
            Err(UnsafeDeleteReason::HomeDirectory(_))
        ));
    }

    #[test]
    fn rejects_a_folder_containing_the_home_directory() {
        let temp = tempdir().unwrap();
        let container = make_repo(temp.path(), "users");
        let home = container.join("estudio");
        std::fs::create_dir_all(&home).unwrap();

        assert!(matches!(
            validate_project_root_for_deletion(&container, Some(&home), &data_dir(temp.path())),
            Err(UnsafeDeleteReason::HomeDirectory(_))
        ));
    }

    // Linux only: on macOS /tmp is a symlink, which the check rejects earlier
    // and for a different reason.
    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_shallow_paths() {
        let temp = tempdir().unwrap();

        // /tmp has a single named component; a project must be deeper
        assert!(matches!(
            validate_project_root_for_deletion(Path::new("/tmp"), None, &data_dir(temp.path())),
            Err(UnsafeDeleteReason::TooShallow(_))
        ));
    }

    #[test]
    fn rejects_a_folder_holding_conduit_data() {
        let temp = tempdir().unwrap();
        let container = make_repo(temp.path(), "everything");
        let data = container.join("conduit-data");
        std::fs::create_dir_all(&data).unwrap();

        assert!(matches!(
            validate_project_root_for_deletion(&container, None, &data),
            Err(UnsafeDeleteReason::ConduitDataDir(_))
        ));
    }

    #[test]
    fn rejects_a_project_inside_conduit_data() {
        let temp = tempdir().unwrap();
        let data = data_dir(temp.path());
        let repo = make_repo(&data, "managed-repo");

        assert!(matches!(
            validate_project_root_for_deletion(&repo, None, &data),
            Err(UnsafeDeleteReason::ConduitDataDir(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn resolves_dot_dot_before_deciding() {
        let temp = tempdir().unwrap();
        make_repo(temp.path(), "my-repo");
        // A path that walks out and back in must be judged by where it lands
        let sneaky = temp.path().join("my-repo").join("..").join("my-repo");

        let result = validate_project_root_for_deletion(&sneaky, None, &data_dir(temp.path()));

        assert_eq!(
            result.unwrap(),
            std::fs::canonicalize(temp.path().join("my-repo")).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn dot_dot_cannot_reach_a_forbidden_path() {
        let temp = tempdir().unwrap();
        let repo = make_repo(temp.path(), "my-repo");
        // Resolves to "/", which must still be refused
        let escape = repo.join("..").join("..").join("..").join("..");

        let result = validate_project_root_for_deletion(&escape, None, &data_dir(temp.path()));

        assert!(
            matches!(
                result,
                Err(UnsafeDeleteReason::FilesystemRoot(_))
                    | Err(UnsafeDeleteReason::TooShallow(_))
                    | Err(UnsafeDeleteReason::NotAGitRepo(_))
            ),
            "unexpected: {result:?}"
        );
    }
}
