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
