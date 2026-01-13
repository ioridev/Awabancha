#![allow(dead_code)]

use anyhow::Result;
use git2::Repository;

/// Repository information (HEAD, branches, remotes)
#[derive(Clone, Debug)]
pub struct RepositoryInfo {
    /// Current HEAD reference name
    pub head_ref: Option<String>,
    /// Current branch name (None if detached)
    pub current_branch: Option<String>,
    /// Is HEAD detached?
    pub is_detached: bool,
    /// Commits ahead of upstream
    pub ahead: usize,
    /// Commits behind upstream
    pub behind: usize,
    /// Remote name (usually "origin")
    pub remote_name: Option<String>,
    /// Remote URL
    pub remote_url: Option<String>,
}

impl RepositoryInfo {
    pub fn from_repo(repo: &Repository) -> Result<Self> {
        let head = repo.head()?;
        let is_detached = head.is_branch() == false;

        let current_branch = if is_detached {
            None
        } else {
            head.shorthand().map(|s| s.to_string())
        };

        let head_ref = head.name().map(|s| s.to_string());

        // Get ahead/behind counts
        let (ahead, behind) = if let Some(ref branch_name) = current_branch {
            Self::get_ahead_behind(repo, branch_name).unwrap_or((0, 0))
        } else {
            (0, 0)
        };

        // Get remote info
        let (remote_name, remote_url) = Self::get_remote_info(repo);

        Ok(Self {
            head_ref,
            current_branch,
            is_detached,
            ahead,
            behind,
            remote_name,
            remote_url,
        })
    }

    fn get_ahead_behind(repo: &Repository, branch_name: &str) -> Result<(usize, usize)> {
        let local_branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
        let upstream = local_branch.upstream()?;

        let local_oid = local_branch.get().target().ok_or(anyhow::anyhow!("No local target"))?;
        let upstream_oid = upstream.get().target().ok_or(anyhow::anyhow!("No upstream target"))?;

        let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;
        Ok((ahead, behind))
    }

    fn get_remote_info(repo: &Repository) -> (Option<String>, Option<String>) {
        if let Ok(remote) = repo.find_remote("origin") {
            let name = Some("origin".to_string());
            let url = remote.url().map(|s| s.to_string());
            (name, url)
        } else {
            // Try to find any remote
            if let Ok(remotes) = repo.remotes() {
                for remote_name in remotes.iter().flatten() {
                    if let Ok(remote) = repo.find_remote(remote_name) {
                        return (
                            Some(remote_name.to_string()),
                            remote.url().map(|s| s.to_string()),
                        );
                    }
                }
            }
            (None, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Create a temporary git repository for testing
    fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Configure user for commits
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        (temp_dir, repo)
    }

    /// Create initial commit in repo
    fn create_initial_commit(repo: &Repository) -> git2::Oid {
        let sig = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap()
    }

    #[test]
    fn test_repository_info_from_new_repo() {
        let (_temp_dir, repo) = create_test_repo();

        // Create initial commit so HEAD is valid
        create_initial_commit(&repo);

        let info = RepositoryInfo::from_repo(&repo).unwrap();

        assert!(info.current_branch.is_some());
        // New repos typically have "main" or "master" as default branch
        let branch = info.current_branch.unwrap();
        assert!(branch == "main" || branch == "master");
        assert!(!info.is_detached);
        assert_eq!(info.ahead, 0);
        assert_eq!(info.behind, 0);
        assert!(info.remote_name.is_none()); // No remote configured
        assert!(info.remote_url.is_none());
    }

    #[test]
    fn test_repository_info_clone() {
        let (_temp_dir, repo) = create_test_repo();
        create_initial_commit(&repo);

        let info = RepositoryInfo::from_repo(&repo).unwrap();
        let cloned = info.clone();

        assert_eq!(info.current_branch, cloned.current_branch);
        assert_eq!(info.is_detached, cloned.is_detached);
        assert_eq!(info.ahead, cloned.ahead);
        assert_eq!(info.behind, cloned.behind);
    }

    #[test]
    fn test_repository_with_file() {
        let (temp_dir, repo) = create_test_repo();

        // Create a file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();

        // Stage the file
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        // Create commit
        let sig = repo.signature().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Add test file", &tree, &[])
            .unwrap();

        let info = RepositoryInfo::from_repo(&repo).unwrap();
        assert!(info.current_branch.is_some());
    }
}
