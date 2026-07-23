//! Git status utilities for tracking diff statistics

use std::path::Path;
use std::process::Command;

/// Git diff statistics (additions, deletions, files changed)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitDiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub files_changed: usize,
}

impl GitDiffStats {
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.additions > 0 || self.deletions > 0
    }

    /// Get git diff stats for the current working directory
    /// Uses `git diff --shortstat` to get uncommitted changes
    pub fn from_working_dir(working_dir: &Path) -> Self {
        // Get stats for staged and unstaged changes
        let output = Command::new("git")
            .args(["--no-optional-locks", "diff", "--shortstat", "HEAD"])
            .current_dir(working_dir)
            .output();

        let stats = match output {
            Ok(o) if o.status.success() => {
                let output_str = String::from_utf8_lossy(&o.stdout);
                Self::parse_shortstat(&output_str)
            }
            _ => GitDiffStats::default(),
        };

        // Fallback: if HEAD comparison fails (e.g., no commits yet), try unstaged-only diff
        if stats == GitDiffStats::default() {
            let unstaged = Command::new("git")
                .args(["--no-optional-locks", "diff", "--shortstat"])
                .current_dir(working_dir)
                .output();

            if let Ok(o) = unstaged {
                if o.status.success() {
                    let output_str = String::from_utf8_lossy(&o.stdout);
                    return Self::parse_shortstat(&output_str);
                }
            }
        }

        stats
    }

    /// Parse from `git diff --shortstat` output
    /// Format: "1 file changed, 44 insertions(+), 10 deletions(-)"
    fn parse_shortstat(output: &str) -> Self {
        let mut stats = GitDiffStats::default();
        let output = output.trim();

        if output.is_empty() {
            return stats;
        }

        for part in output.split(',') {
            let part = part.trim();
            if part.contains("insertion") {
                stats.additions = part
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0);
            } else if part.contains("deletion") {
                stats.deletions = part
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0);
            } else if part.contains("file") {
                stats.files_changed = part
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0);
            }
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// A repository with one commit and a repo-local identity
    fn init_repo(path: &Path) {
        let run = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(path)
                .output()
                .expect("git failed to run");
            assert!(out.status.success(), "git {args:?} failed");
        };
        run(&["init"]);
        run(&["config", "user.email", "test@test.com"]);
        run(&["config", "user.name", "Test"]);
        std::fs::write(path.join("a.txt"), "x").unwrap();
        run(&["add", "--", "a.txt"]);
        run(&["commit", "-m", "first"]);
    }

    #[test]
    fn exposure_flags_a_repository_with_no_remote() {
        let dir = tempdir().unwrap();
        init_repo(dir.path());

        let exposure = RepositoryExposure::inspect(dir.path());

        assert!(!exposure.has_remote, "a fresh repo has no remote");
        // With no remote, every commit exists only here
        assert!(exposure.unpushed_commits >= 1, "{exposure:?}");
    }

    #[test]
    fn exposure_counts_commits_missing_from_the_remote() {
        let origin = tempdir().unwrap();
        let clone = tempdir().unwrap();
        init_repo(origin.path());

        let clone_path = clone.path().join("work");
        let out = Command::new("git")
            .args(["clone"])
            .arg(origin.path())
            .arg(&clone_path)
            .output()
            .unwrap();
        assert!(out.status.success(), "clone failed");

        let run = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(&clone_path)
                .output()
                .unwrap();
        };
        run(&["config", "user.email", "test@test.com"]);
        run(&["config", "user.name", "Test"]);

        let fresh = RepositoryExposure::inspect(&clone_path);
        assert!(fresh.has_remote);
        assert_eq!(fresh.unpushed_commits, 0, "nothing local yet: {fresh:?}");

        std::fs::write(clone_path.join("b.txt"), "y").unwrap();
        run(&["add", "--", "b.txt"]);
        run(&["commit", "-m", "local only"]);

        let after = RepositoryExposure::inspect(&clone_path);
        assert_eq!(after.unpushed_commits, 1, "{after:?}");
    }

    #[test]
    fn test_parse_shortstat_full() {
        let output = " 3 files changed, 44 insertions(+), 10 deletions(-)";
        let stats = GitDiffStats::parse_shortstat(output);
        assert_eq!(stats.files_changed, 3);
        assert_eq!(stats.additions, 44);
        assert_eq!(stats.deletions, 10);
    }

    #[test]
    fn test_parse_shortstat_insertions_only() {
        let output = " 1 file changed, 25 insertions(+)";
        let stats = GitDiffStats::parse_shortstat(output);
        assert_eq!(stats.files_changed, 1);
        assert_eq!(stats.additions, 25);
        assert_eq!(stats.deletions, 0);
    }

    #[test]
    fn test_parse_shortstat_deletions_only() {
        let output = " 2 files changed, 15 deletions(-)";
        let stats = GitDiffStats::parse_shortstat(output);
        assert_eq!(stats.files_changed, 2);
        assert_eq!(stats.additions, 0);
        assert_eq!(stats.deletions, 15);
    }

    #[test]
    fn test_parse_shortstat_empty() {
        let output = "";
        let stats = GitDiffStats::parse_shortstat(output);
        assert_eq!(stats.files_changed, 0);
        assert_eq!(stats.additions, 0);
        assert_eq!(stats.deletions, 0);
    }

    #[test]
    fn test_has_changes() {
        let empty = GitDiffStats::default();
        assert!(!empty.has_changes());

        let with_additions = GitDiffStats {
            additions: 10,
            deletions: 0,
            files_changed: 1,
        };
        assert!(with_additions.has_changes());

        let with_deletions = GitDiffStats {
            additions: 0,
            deletions: 5,
            files_changed: 1,
        };
        assert!(with_deletions.has_changes());
    }
}

/// What a repository would lose if its folder were deleted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepositoryExposure {
    /// Whether any remote is configured
    pub has_remote: bool,
    /// Commits that exist on no remote, across every local branch
    pub unpushed_commits: usize,
}

impl RepositoryExposure {
    /// Inspect a repository for work that only exists locally.
    ///
    /// A repository with no remote has its entire history at stake; one with a
    /// remote still loses whatever was never pushed.
    pub fn inspect(repo_path: &Path) -> Self {
        let has_remote = Command::new("git")
            .args(["--no-optional-locks", "remote"])
            .current_dir(repo_path)
            .output()
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false);

        // Commits reachable from local branches but from no remote branch
        let unpushed_commits = Command::new("git")
            .args([
                "--no-optional-locks",
                "rev-list",
                "--count",
                "--branches",
                "--not",
                "--remotes",
            ])
            .current_dir(repo_path)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
            .unwrap_or(0);

        Self {
            has_remote,
            unpushed_commits,
        }
    }
}
