#![allow(dead_code)]

use anyhow::Result;
use git2::Repository;
use std::path::Path;

/// Conflicted file info
#[derive(Clone, Debug)]
pub struct ConflictedFile {
    pub path: String,
    pub is_deleted_by_us: bool,
    pub is_deleted_by_them: bool,
}

/// Merge conflict information
#[derive(Clone, Debug)]
pub struct ConflictInfo {
    pub conflicted_files: Vec<ConflictedFile>,
    pub source_branch: Option<String>,
    pub target_branch: Option<String>,
    pub is_merging: bool,
}

impl ConflictInfo {
    pub fn get(repo: &Repository) -> Result<Option<Self>> {
        let state = repo.state();

        if state != git2::RepositoryState::Merge
            && state != git2::RepositoryState::RebaseMerge
            && state != git2::RepositoryState::CherryPick
        {
            return Ok(None);
        }

        let index = repo.index()?;
        if !index.has_conflicts() {
            return Ok(None);
        }

        let mut conflicted_files = Vec::new();

        let conflicts = index.conflicts()?;
        for conflict in conflicts {
            let conflict = conflict?;

            let path = conflict
                .our
                .as_ref()
                .or(conflict.their.as_ref())
                .or(conflict.ancestor.as_ref())
                .and_then(|entry| {
                    std::str::from_utf8(&entry.path)
                        .ok()
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "unknown".to_string());

            let is_deleted_by_us = conflict.our.is_none();
            let is_deleted_by_them = conflict.their.is_none();

            conflicted_files.push(ConflictedFile {
                path,
                is_deleted_by_us,
                is_deleted_by_them,
            });
        }

        // Try to get branch names from MERGE_HEAD and HEAD
        let source_branch = Self::get_merge_head_branch(repo);
        let target_branch = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(|s| s.to_string()));

        Ok(Some(ConflictInfo {
            conflicted_files,
            source_branch,
            target_branch,
            is_merging: state == git2::RepositoryState::Merge,
        }))
    }

    fn get_merge_head_branch(repo: &Repository) -> Option<String> {
        let merge_head_path = repo.path().join("MERGE_HEAD");
        if merge_head_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&merge_head_path) {
                let oid_str = content.trim();
                if let Ok(oid) = git2::Oid::from_str(oid_str) {
                    // Try to find a branch pointing to this commit
                    if let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) {
                        for branch in branches.flatten() {
                            if branch.0.get().target() == Some(oid) {
                                if let Ok(Some(name)) = branch.0.name() {
                                    return Some(name.to_string());
                                }
                            }
                        }
                    }
                    return Some(oid_str[..7].to_string());
                }
            }
        }
        None
    }

    pub fn resolve_all(repo: &Repository, strategy: ConflictStrategy) -> Result<()> {
        let mut index = repo.index()?;
        let conflicts: Vec<_> = index.conflicts()?.collect();

        for conflict in conflicts {
            let conflict = conflict?;

            let path = conflict
                .our
                .as_ref()
                .or(conflict.their.as_ref())
                .and_then(|entry| std::str::from_utf8(&entry.path).ok())
                .ok_or_else(|| anyhow::anyhow!("Invalid path in conflict"))?;

            Self::resolve_file(repo, &mut index, path, &conflict, strategy)?;
        }

        index.write()?;
        Ok(())
    }

    pub fn resolve_file(
        repo: &Repository,
        index: &mut git2::Index,
        path: &str,
        conflict: &git2::IndexConflict,
        strategy: ConflictStrategy,
    ) -> Result<()> {
        let entry = match strategy {
            ConflictStrategy::Ours => conflict.our.as_ref(),
            ConflictStrategy::Theirs => conflict.their.as_ref(),
        };

        if let Some(entry) = entry {
            // Get the blob content
            let blob = repo.find_blob(entry.id)?;
            let content = blob.content();

            // Write the resolved content to working directory
            let workdir = repo.workdir().ok_or_else(|| anyhow::anyhow!("No workdir"))?;
            let file_path = workdir.join(path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, content)?;

            // Stage the resolved file
            index.add_path(Path::new(path))?;
        } else {
            // File was deleted in chosen strategy
            let workdir = repo.workdir().ok_or_else(|| anyhow::anyhow!("No workdir"))?;
            let file_path = workdir.join(path);
            if file_path.exists() {
                std::fs::remove_file(&file_path)?;
            }
            index.remove_path(Path::new(path))?;
        }

        Ok(())
    }

    pub fn complete_merge(repo: &Repository, message: Option<&str>) -> Result<()> {
        let sig = repo.signature()?;
        let mut index = repo.index()?;

        if index.has_conflicts() {
            anyhow::bail!("Cannot complete merge with unresolved conflicts");
        }

        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;

        let head = repo.head()?.peel_to_commit()?;

        // Get MERGE_HEAD
        let merge_head_path = repo.path().join("MERGE_HEAD");
        let merge_head_content = std::fs::read_to_string(&merge_head_path)?;
        let merge_oid = git2::Oid::from_str(merge_head_content.trim())?;
        let merge_commit = repo.find_commit(merge_oid)?;

        let msg = message.unwrap_or("Merge commit");

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            msg,
            &tree,
            &[&head, &merge_commit],
        )?;

        repo.cleanup_state()?;
        Ok(())
    }

    pub fn abort_merge(repo: &Repository) -> Result<()> {
        let head = repo.head()?.peel_to_commit()?;
        repo.reset(&head.into_object(), git2::ResetType::Hard, None)?;
        repo.cleanup_state()?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConflictStrategy {
    Ours,
    Theirs,
}
