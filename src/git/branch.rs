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
