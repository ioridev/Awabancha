#![allow(dead_code)]

use anyhow::Result;
use git2::{Repository, StatusOptions};

/// Status of a file in the working directory
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileStatusType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

/// File status entry
#[derive(Clone, Debug)]
pub struct FileStatus {
    pub path: String,
    pub status: FileStatusType,
    pub staged: bool,
    /// Old path for renamed files
    pub old_path: Option<String>,
}

impl FileStatus {
    pub fn get_all(repo: &Repository) -> Result<Vec<Self>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false)
            .include_unmodified(false)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let statuses = repo.statuses(Some(&mut opts))?;
        let mut files = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry
                .path()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Check index (staged) status
            if status.is_index_new() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Added,
                    staged: true,
                    old_path: None,
                });
            } else if status.is_index_modified() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Modified,
                    staged: true,
                    old_path: None,
                });
            } else if status.is_index_deleted() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Deleted,
                    staged: true,
                    old_path: None,
                });
            } else if status.is_index_renamed() {
                let old_path = entry
                    .head_to_index()
                    .and_then(|d| d.old_file().path())
                    .map(|p| p.to_string_lossy().to_string());
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Renamed,
                    staged: true,
                    old_path,
                });
            }

            // Check working directory (unstaged) status
            if status.is_wt_new() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Untracked,
                    staged: false,
                    old_path: None,
                });
            } else if status.is_wt_modified() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Modified,
                    staged: false,
                    old_path: None,
                });
            } else if status.is_wt_deleted() {
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Deleted,
                    staged: false,
                    old_path: None,
                });
            } else if status.is_wt_renamed() {
                let old_path = entry
                    .index_to_workdir()
                    .and_then(|d| d.old_file().path())
                    .map(|p| p.to_string_lossy().to_string());
                files.push(FileStatus {
                    path: path.clone(),
                    status: FileStatusType::Renamed,
                    staged: false,
                    old_path,
                });
            }

            // Check for conflicts
            if status.is_conflicted() {
                files.push(FileStatus {
                    path,
                    status: FileStatusType::Conflicted,
                    staged: false,
                    old_path: None,
                });
            }
        }

        // Sort: staged first, then by path
        files.sort_by(|a, b| {
            match (a.staged, b.staged) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.path.cmp(&b.path),
            }
        });

        Ok(files)
    }

    pub fn status_color(&self) -> u32 {
        match self.status {
            FileStatusType::Added => 0xa6e3a1,      // Green
            FileStatusType::Modified => 0xfab387,   // Orange
            FileStatusType::Deleted => 0xf38ba8,    // Red
            FileStatusType::Renamed => 0x89b4fa,    // Blue
            FileStatusType::Untracked => 0x9399b2,  // Gray
            FileStatusType::Conflicted => 0xf9e2af, // Yellow
        }
    }

    pub fn status_char(&self) -> char {
        match self.status {
            FileStatusType::Added => 'A',
            FileStatusType::Modified => 'M',
            FileStatusType::Deleted => 'D',
            FileStatusType::Renamed => 'R',
            FileStatusType::Untracked => '?',
            FileStatusType::Conflicted => '!',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use git2::Repository;

    fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        (temp_dir, repo)
    }

    fn create_initial_commit(temp_dir: &TempDir, repo: &Repository) {
        let file_path = temp_dir.path().join("initial.txt");
        fs::write(&file_path, "initial content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("initial.txt")).unwrap();
        index.write().unwrap();

        let sig = repo.signature().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
    }

    #[test]
    fn test_file_status_creation() {
        let status = FileStatus {
            path: "src/main.rs".to_string(),
            status: FileStatusType::Modified,
            staged: true,
            old_path: None,
        };

        assert_eq!(status.path, "src/main.rs");
        assert_eq!(status.status, FileStatusType::Modified);
        assert!(status.staged);
        assert!(status.old_path.is_none());
    }

    #[test]
    fn test_file_status_renamed() {
        let status = FileStatus {
            path: "new_name.rs".to_string(),
            status: FileStatusType::Renamed,
            staged: true,
            old_path: Some("old_name.rs".to_string()),
        };

        assert_eq!(status.path, "new_name.rs");
        assert_eq!(status.old_path, Some("old_name.rs".to_string()));
    }

    #[test]
    fn test_file_status_clone() {
        let status = FileStatus {
            path: "test.rs".to_string(),
            status: FileStatusType::Added,
            staged: false,
            old_path: Some("old.rs".to_string()),
        };

        let cloned = status.clone();
        assert_eq!(status.path, cloned.path);
        assert_eq!(status.status, cloned.status);
        assert_eq!(status.staged, cloned.staged);
        assert_eq!(status.old_path, cloned.old_path);
    }

    #[test]
    fn test_get_all_empty_repo() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        let statuses = FileStatus::get_all(&repo).unwrap();
        assert!(statuses.is_empty());
    }

    #[test]
    fn test_get_all_untracked_file() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Create an untracked file
        let file_path = temp_dir.path().join("untracked.txt");
        fs::write(&file_path, "untracked content").unwrap();

        let statuses = FileStatus::get_all(&repo).unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].path, "untracked.txt");
        assert_eq!(statuses[0].status, FileStatusType::Untracked);
        assert!(!statuses[0].staged);
    }

    #[test]
    fn test_get_all_modified_file() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Modify the existing file
        let file_path = temp_dir.path().join("initial.txt");
        fs::write(&file_path, "modified content").unwrap();

        let statuses = FileStatus::get_all(&repo).unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].path, "initial.txt");
        assert_eq!(statuses[0].status, FileStatusType::Modified);
        assert!(!statuses[0].staged);
    }

    #[test]
    fn test_get_all_staged_file() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Create and stage a new file
        let file_path = temp_dir.path().join("new_file.txt");
        fs::write(&file_path, "new content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("new_file.txt")).unwrap();
        index.write().unwrap();

        let statuses = FileStatus::get_all(&repo).unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].path, "new_file.txt");
        assert_eq!(statuses[0].status, FileStatusType::Added);
        assert!(statuses[0].staged);
    }

    #[test]
    fn test_get_all_deleted_file() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Delete the file
        let file_path = temp_dir.path().join("initial.txt");
        fs::remove_file(&file_path).unwrap();

        let statuses = FileStatus::get_all(&repo).unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].path, "initial.txt");
        assert_eq!(statuses[0].status, FileStatusType::Deleted);
        assert!(!statuses[0].staged);
    }

    #[test]
    fn test_get_all_sorting() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);

        // Create and stage a file
        let staged_file = temp_dir.path().join("staged.txt");
        fs::write(&staged_file, "staged content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("staged.txt")).unwrap();
        index.write().unwrap();

        // Create an untracked file (alphabetically first)
        let untracked_file = temp_dir.path().join("aaa_untracked.txt");
        fs::write(&untracked_file, "untracked content").unwrap();

        let statuses = FileStatus::get_all(&repo).unwrap();
        assert_eq!(statuses.len(), 2);
        // Staged should come first regardless of alphabetical order
        assert!(statuses[0].staged);
        assert!(!statuses[1].staged);
    }

    #[test]
    fn test_status_color() {
        let added = FileStatus {
            path: "test.rs".to_string(),
            status: FileStatusType::Added,
            staged: true,
            old_path: None,
        };
        assert_eq!(added.status_color(), 0xa6e3a1); // Green

        let modified = FileStatus {
            path: "test.rs".to_string(),
            status: FileStatusType::Modified,
            staged: false,
            old_path: None,
        };
        assert_eq!(modified.status_color(), 0xfab387); // Orange

        let deleted = FileStatus {
            path: "test.rs".to_string(),
            status: FileStatusType::Deleted,
            staged: true,
            old_path: None,
        };
        assert_eq!(deleted.status_color(), 0xf38ba8); // Red

        let renamed = FileStatus {
            path: "new.rs".to_string(),
            status: FileStatusType::Renamed,
            staged: true,
            old_path: Some("old.rs".to_string()),
        };
        assert_eq!(renamed.status_color(), 0x89b4fa); // Blue

        let untracked = FileStatus {
            path: "test.rs".to_string(),
            status: FileStatusType::Untracked,
            staged: false,
            old_path: None,
        };
        assert_eq!(untracked.status_color(), 0x9399b2); // Gray

        let conflicted = FileStatus {
            path: "test.rs".to_string(),
            status: FileStatusType::Conflicted,
            staged: false,
            old_path: None,
        };
        assert_eq!(conflicted.status_color(), 0xf9e2af); // Yellow
    }

    #[test]
    fn test_status_char() {
        let test_cases = [
            (FileStatusType::Added, 'A'),
            (FileStatusType::Modified, 'M'),
            (FileStatusType::Deleted, 'D'),
            (FileStatusType::Renamed, 'R'),
            (FileStatusType::Untracked, '?'),
            (FileStatusType::Conflicted, '!'),
        ];

        for (status_type, expected_char) in test_cases {
            let file_status = FileStatus {
                path: "test.rs".to_string(),
                status: status_type,
                staged: false,
                old_path: None,
            };
            assert_eq!(file_status.status_char(), expected_char);
        }
    }

    #[test]
    fn test_file_status_type_equality() {
        assert_eq!(FileStatusType::Added, FileStatusType::Added);
        assert_ne!(FileStatusType::Added, FileStatusType::Modified);
    }
}
