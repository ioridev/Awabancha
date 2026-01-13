#![allow(dead_code)]

use anyhow::Result;
use git2::{BranchType, Repository};

/// Branch information
#[derive(Clone, Debug)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub upstream: Option<String>,
    pub branch_type: BranchKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BranchKind {
    Local,
    Remote,
}

impl BranchInfo {
    pub fn get_all(repo: &Repository) -> Result<Vec<Self>> {
        let mut branches = Vec::new();

        // Get HEAD reference to determine current branch
        let head_name = repo.head().ok().and_then(|h| {
            if h.is_branch() {
                h.shorthand().map(|s| s.to_string())
            } else {
                None
            }
        });

        // Local branches
        for branch in repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                let is_head = head_name.as_ref().map(|h| h == name).unwrap_or(false);
                let upstream = branch
                    .upstream()
                    .ok()
                    .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));

                branches.push(BranchInfo {
                    name: name.to_string(),
                    is_head,
                    upstream,
                    branch_type: BranchKind::Local,
                });
            }
        }

        // Remote branches
        for branch in repo.branches(Some(BranchType::Remote))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                branches.push(BranchInfo {
                    name: name.to_string(),
                    is_head: false,
                    upstream: None,
                    branch_type: BranchKind::Remote,
                });
            }
        }

        // Sort: current branch first, then main/master, then alphabetically
        branches.sort_by(|a, b| {
            if a.is_head {
                return std::cmp::Ordering::Less;
            }
            if b.is_head {
                return std::cmp::Ordering::Greater;
            }

            let priority = |name: &str| -> u8 {
                match name {
                    "main" => 0,
                    "master" => 1,
                    "develop" | "dev" => 2,
                    _ => 3,
                }
            };

            let a_priority = priority(&a.name);
            let b_priority = priority(&b.name);

            if a_priority != b_priority {
                a_priority.cmp(&b_priority)
            } else {
                a.name.cmp(&b.name)
            }
        });

        Ok(branches)
    }

    pub fn local_branches(repo: &Repository) -> Result<Vec<Self>> {
        Ok(Self::get_all(repo)?
            .into_iter()
            .filter(|b| b.branch_type == BranchKind::Local)
            .collect())
    }

    pub fn remote_branches(repo: &Repository) -> Result<Vec<Self>> {
        Ok(Self::get_all(repo)?
            .into_iter()
            .filter(|b| b.branch_type == BranchKind::Remote)
            .collect())
    }
}

/// Merge mode
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MergeMode {
    Auto,
    FfOnly,
    NoFf,
    Squash,
}

impl MergeMode {
    pub fn merge_branch(
        repo: &Repository,
        branch_name: &str,
        mode: MergeMode,
    ) -> Result<()> {
        let branch = repo.find_branch(branch_name, BranchType::Local)?;
        let branch_commit = branch.get().peel_to_commit()?;
        let annotated = repo.find_annotated_commit(branch_commit.id())?;

        let (analysis, _) = repo.merge_analysis(&[&annotated])?;

        match mode {
            MergeMode::FfOnly => {
                if !analysis.is_fast_forward() {
                    anyhow::bail!("Cannot fast-forward, merge required");
                }
                Self::fast_forward_merge(repo, &branch_commit)?;
            }
            MergeMode::NoFf => {
                Self::create_merge_commit(repo, branch_name, &branch_commit)?;
            }
            MergeMode::Squash => {
                Self::squash_merge(repo, &annotated)?;
            }
            MergeMode::Auto => {
                if analysis.is_fast_forward() {
                    Self::fast_forward_merge(repo, &branch_commit)?;
                } else if analysis.is_normal() {
                    repo.merge(&[&annotated], None, None)?;
                    // Merge commit will be created when conflicts are resolved or if clean
                } else {
                    anyhow::bail!("Nothing to merge");
                }
            }
        }

        Ok(())
    }

    fn fast_forward_merge(repo: &Repository, commit: &git2::Commit) -> Result<()> {
        let head = repo.head()?;
        let branch_name = head.shorthand().unwrap_or("HEAD");
        let mut reference = repo.find_reference(&format!("refs/heads/{}", branch_name))?;
        reference.set_target(commit.id(), "Fast-forward merge")?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        Ok(())
    }

    fn create_merge_commit(
        repo: &Repository,
        branch_name: &str,
        branch_commit: &git2::Commit,
    ) -> Result<()> {
        let head = repo.head()?.peel_to_commit()?;
        let sig = repo.signature()?;

        // Merge the trees
        let ancestor = repo.find_commit(repo.merge_base(head.id(), branch_commit.id())?)?;
        let mut index = repo.merge_trees(
            &ancestor.tree()?,
            &head.tree()?,
            &branch_commit.tree()?,
            None,
        )?;

        if index.has_conflicts() {
            anyhow::bail!("Merge has conflicts");
        }

        let tree_oid = index.write_tree_to(repo)?;
        let tree = repo.find_tree(tree_oid)?;

        let message = format!("Merge branch '{}'", branch_name);
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &message,
            &tree,
            &[&head, branch_commit],
        )?;

        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
        Ok(())
    }

    fn squash_merge(repo: &Repository, annotated: &git2::AnnotatedCommit) -> Result<()> {
        repo.merge(&[annotated], None, None)?;
        // Don't create commit yet - leave staged for user to commit
        repo.cleanup_state()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        (temp_dir, repo)
    }

    fn create_initial_commit(temp_dir: &TempDir, repo: &Repository) -> git2::Oid {
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "initial content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let sig = repo.signature().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap()
    }

    #[test]
    fn test_branch_kind_equality() {
        assert_eq!(BranchKind::Local, BranchKind::Local);
        assert_eq!(BranchKind::Remote, BranchKind::Remote);
        assert_ne!(BranchKind::Local, BranchKind::Remote);
    }

    #[test]
    fn test_branch_kind_clone() {
        let kind = BranchKind::Local;
        let cloned = kind;
        assert_eq!(kind, cloned);
    }

    #[test]
    fn test_merge_mode_equality() {
        assert_eq!(MergeMode::Auto, MergeMode::Auto);
        assert_eq!(MergeMode::FfOnly, MergeMode::FfOnly);
        assert_eq!(MergeMode::NoFf, MergeMode::NoFf);
        assert_eq!(MergeMode::Squash, MergeMode::Squash);
        assert_ne!(MergeMode::Auto, MergeMode::FfOnly);
    }

    #[test]
    fn test_merge_mode_clone() {
        let mode = MergeMode::Auto;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_branch_info_creation() {
        let branch = BranchInfo {
            name: "main".to_string(),
            is_head: true,
            upstream: Some("origin/main".to_string()),
            branch_type: BranchKind::Local,
        };

        assert_eq!(branch.name, "main");
        assert!(branch.is_head);
        assert_eq!(branch.upstream, Some("origin/main".to_string()));
        assert_eq!(branch.branch_type, BranchKind::Local);
    }

    #[test]
    fn test_remote_branch_info() {
        let branch = BranchInfo {
            name: "origin/main".to_string(),
            is_head: false,
            upstream: None,
            branch_type: BranchKind::Remote,
        };

        assert!(!branch.is_head);
        assert!(branch.upstream.is_none());
        assert_eq!(branch.branch_type, BranchKind::Remote);
    }

    #[test]
    fn test_branch_info_clone() {
        let branch = BranchInfo {
            name: "feature".to_string(),
            is_head: false,
            upstream: Some("origin/feature".to_string()),
            branch_type: BranchKind::Local,
        };

        let cloned = branch.clone();
        assert_eq!(branch.name, cloned.name);
        assert_eq!(branch.is_head, cloned.is_head);
        assert_eq!(branch.upstream, cloned.upstream);
        assert_eq!(branch.branch_type, cloned.branch_type);
    }

    #[test]
    fn test_get_all_branches() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        let branches = BranchInfo::get_all(&repo).unwrap();
        assert!(!branches.is_empty());
        // Should have at least one branch (main or master)
        assert!(branches.iter().any(|b| b.is_head));
    }

    #[test]
    fn test_local_branches() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        let branches = BranchInfo::local_branches(&repo).unwrap();
        assert!(!branches.is_empty());
        // All returned branches should be local
        assert!(branches.iter().all(|b| b.branch_type == BranchKind::Local));
    }

    #[test]
    fn test_remote_branches_empty() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        let branches = BranchInfo::remote_branches(&repo).unwrap();
        // New repo has no remote branches
        assert!(branches.is_empty());
    }

    #[test]
    fn test_branch_sorting_head_first() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Create another branch
        let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("z_feature", &head_commit, false).unwrap();

        let branches = BranchInfo::get_all(&repo).unwrap();
        // HEAD branch should always be first
        assert!(branches[0].is_head);
    }

    #[test]
    fn test_branch_sorting_main_priority() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        let head_commit = repo.head().unwrap().peel_to_commit().unwrap();

        // Create branches with different priorities
        repo.branch("z_branch", &head_commit, false).unwrap();
        repo.branch("develop", &head_commit, false).unwrap();

        // Checkout z_branch so main is not HEAD
        repo.set_head("refs/heads/z_branch").unwrap();

        let branches = BranchInfo::local_branches(&repo).unwrap();

        // z_branch should be first (HEAD), then main/master, then develop, then others
        assert!(branches[0].is_head);
    }
}
