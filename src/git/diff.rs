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

    /// Get short path for display (basename)
    pub fn short_path(&self) -> &str {
        std::path::Path::new(&self.path).file_name().and_then(|s| s.to_str()).unwrap_or(&self.path)
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

    fn create_initial_commit(temp_dir: &TempDir, repo: &Repository) {
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "initial content\nline 2\nline 3").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let sig = repo.signature().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
    }

    #[test]
    fn test_diff_line_type_equality() {
        assert_eq!(DiffLineType::Context, DiffLineType::Context);
        assert_eq!(DiffLineType::Addition, DiffLineType::Addition);
        assert_eq!(DiffLineType::Deletion, DiffLineType::Deletion);
        assert_eq!(DiffLineType::Header, DiffLineType::Header);
        assert_ne!(DiffLineType::Addition, DiffLineType::Deletion);
    }

    #[test]
    fn test_diff_line_creation() {
        let line = DiffLine {
            content: "+ added line".to_string(),
            line_type: DiffLineType::Addition,
            old_lineno: None,
            new_lineno: Some(10),
        };

        assert_eq!(line.content, "+ added line");
        assert_eq!(line.line_type, DiffLineType::Addition);
        assert!(line.old_lineno.is_none());
        assert_eq!(line.new_lineno, Some(10));
    }

    #[test]
    fn test_file_diff_creation() {
        let diff = FileDiff {
            path: "src/lib.rs".to_string(),
            old_path: Some("src/old_lib.rs".to_string()),
            lines: vec![
                DiffLine {
                    content: "context".to_string(),
                    line_type: DiffLineType::Context,
                    old_lineno: Some(1),
                    new_lineno: Some(1),
                },
                DiffLine {
                    content: "+new".to_string(),
                    line_type: DiffLineType::Addition,
                    old_lineno: None,
                    new_lineno: Some(2),
                },
            ],
            additions: 1,
            deletions: 0,
        };

        assert_eq!(diff.path, "src/lib.rs");
        assert_eq!(diff.old_path, Some("src/old_lib.rs".to_string()));
        assert_eq!(diff.lines.len(), 2);
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 0);
    }

    #[test]
    fn test_file_diff_short_path() {
        let diff = FileDiff {
            path: "src/components/button.rs".to_string(),
            old_path: None,
            lines: vec![],
            additions: 0,
            deletions: 0,
        };

        assert_eq!(diff.short_path(), "button.rs");
    }

    #[test]
    fn test_file_diff_short_path_no_slash() {
        let diff = FileDiff {
            path: "README.md".to_string(),
            old_path: None,
            lines: vec![],
            additions: 0,
            deletions: 0,
        };

        assert_eq!(diff.short_path(), "README.md");
    }

    #[test]
    fn test_diff_line_clone() {
        let line = DiffLine {
            content: "test".to_string(),
            line_type: DiffLineType::Context,
            old_lineno: Some(5),
            new_lineno: Some(5),
        };

        let cloned = line.clone();
        assert_eq!(line.content, cloned.content);
        assert_eq!(line.line_type, cloned.line_type);
        assert_eq!(line.old_lineno, cloned.old_lineno);
        assert_eq!(line.new_lineno, cloned.new_lineno);
    }

    #[test]
    fn test_file_diff_clone() {
        let diff = FileDiff {
            path: "test.rs".to_string(),
            old_path: Some("old_test.rs".to_string()),
            lines: vec![],
            additions: 5,
            deletions: 3,
        };

        let cloned = diff.clone();
        assert_eq!(diff.path, cloned.path);
        assert_eq!(diff.old_path, cloned.old_path);
        assert_eq!(diff.additions, cloned.additions);
        assert_eq!(diff.deletions, cloned.deletions);
    }

    #[test]
    fn test_get_file_diff() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Modify the file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "modified content\nline 2\nline 3\nnew line").unwrap();

        let result = FileDiff::get_file_diff(&repo, "test.txt");
        assert!(result.is_ok());

        let diff = result.unwrap();
        assert_eq!(diff.path, "test.txt");
        assert!(diff.additions > 0 || diff.deletions > 0);
    }

    #[test]
    fn test_get_commit_diff() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Create a second commit with changes
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "changed content\nline 2\nline 3").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let sig = repo.signature().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        let new_commit = repo.commit(Some("HEAD"), &sig, &sig, "Second commit", &tree, &[&head])
            .unwrap();

        let result = FileDiff::get_commit_diff(&repo, &new_commit.to_string());
        assert!(result.is_ok());

        let diffs = result.unwrap();
        assert!(!diffs.is_empty());
    }

    #[test]
    fn test_file_diff_all_line_types() {
        let diff = FileDiff {
            path: "test.rs".to_string(),
            old_path: None,
            lines: vec![
                DiffLine {
                    content: "@@ -1,3 +1,4 @@".to_string(),
                    line_type: DiffLineType::Header,
                    old_lineno: None,
                    new_lineno: None,
                },
                DiffLine {
                    content: " context line".to_string(),
                    line_type: DiffLineType::Context,
                    old_lineno: Some(1),
                    new_lineno: Some(1),
                },
                DiffLine {
                    content: "-removed line".to_string(),
                    line_type: DiffLineType::Deletion,
                    old_lineno: Some(2),
                    new_lineno: None,
                },
                DiffLine {
                    content: "+added line".to_string(),
                    line_type: DiffLineType::Addition,
                    old_lineno: None,
                    new_lineno: Some(2),
                },
            ],
            additions: 1,
            deletions: 1,
        };

        assert_eq!(diff.lines.len(), 4);
        assert_eq!(diff.lines[0].line_type, DiffLineType::Header);
        assert_eq!(diff.lines[1].line_type, DiffLineType::Context);
        assert_eq!(diff.lines[2].line_type, DiffLineType::Deletion);
        assert_eq!(diff.lines[3].line_type, DiffLineType::Addition);
    }
}
