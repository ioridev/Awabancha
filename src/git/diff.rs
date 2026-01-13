#![allow(dead_code)]

use anyhow::Result;
use git2::{DiffOptions, Repository};

/// Line in a diff
#[derive(Clone, Debug)]
pub struct DiffLine {
    pub content: String,
    pub line_type: DiffLineType,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
    Header,
}

/// Diff for a single file
#[derive(Clone, Debug)]
pub struct FileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub lines: Vec<DiffLine>,
    pub additions: usize,
    pub deletions: usize,
}

impl FileDiff {
    /// Get diff for a file in the working directory
    pub fn get_file_diff(repo: &Repository, path: &str) -> Result<Self> {
        let mut opts = DiffOptions::new();
        opts.pathspec(path);

        // Compare HEAD to working directory
        let head = repo.head()?.peel_to_tree()?;
        let diff = repo.diff_tree_to_workdir_with_index(Some(&head), Some(&mut opts))?;

        Self::from_diff(&diff, path)
    }

    /// Get diff for a specific commit
    pub fn get_commit_diff(repo: &Repository, sha: &str) -> Result<Vec<Self>> {
        let oid = git2::Oid::from_str(sha)?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;

        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        let mut diffs = Vec::new();
        let deltas: Vec<_> = diff.deltas().collect();

        for delta in &deltas {
            let path = delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            if let Ok(file_diff) = Self::from_diff(&diff, &path) {
                diffs.push(file_diff);
            }
        }

        Ok(diffs)
    }

    fn from_diff(diff: &git2::Diff, target_path: &str) -> Result<Self> {
        let mut lines = Vec::new();
        let mut additions = 0;
        let mut deletions = 0;
        let mut old_path = None;
        let mut found = false;

        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            let path = delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            if path != target_path {
                return true;
            }

            found = true;

            if old_path.is_none() {
                old_path = delta
                    .old_file()
                    .path()
                    .map(|p| p.to_string_lossy().to_string());
            }

            let content = String::from_utf8_lossy(line.content()).to_string();
            let line_type = match line.origin() {
                '+' => {
                    additions += 1;
                    DiffLineType::Addition
                }
                '-' => {
                    deletions += 1;
                    DiffLineType::Deletion
                }
                ' ' => DiffLineType::Context,
                _ => DiffLineType::Header,
            };

            lines.push(DiffLine {
                content,
                line_type,
                old_lineno: line.old_lineno(),
                new_lineno: line.new_lineno(),
            });

            true
        })?;

        if !found {
            anyhow::bail!("File not found in diff: {}", target_path);
        }

        Ok(Self {
            path: target_path.to_string(),
            old_path,
            lines,
            additions,
            deletions,
        })
    }
}
