#![allow(dead_code)]

use crate::git::{
    self, BranchInfo, CommitGraphData, CommitInfo, ConflictInfo, ConflictStrategy, FileDiff,
    FileStatus, ResetMode, RepositoryInfo, StashEntry, TagInfo,
};
use anyhow::Result;
use gpui::*;
use std::path::{Path, PathBuf};

/// Credentials for git operations
#[derive(Clone)]
pub struct GitCredentials {
    pub username: String,
    pub password: String,
}

/// Main git state for the application
pub struct GitState {
    /// Path to the repository
    pub path: Option<PathBuf>,
    /// Repository info (HEAD, branches, etc.)
    pub repository_info: Option<RepositoryInfo>,
    /// File status (staged/unstaged changes)
    pub files: Vec<FileStatus>,
    /// Selected files in the file list
    pub selected_files: Vec<String>,
    /// Commit graph data
    pub commits: Option<CommitGraphData>,
    /// Currently selected commit
    pub selected_commit: Option<CommitInfo>,
    /// Current diff being viewed
    pub current_diff: Option<FileDiff>,
    /// List of branches
    pub branches: Vec<BranchInfo>,
    /// List of tags
    pub tags: Vec<TagInfo>,
    /// List of stashes
    pub stashes: Vec<StashEntry>,
    /// Merge conflict info
    pub conflict_info: Option<ConflictInfo>,
    /// Is loading
    pub is_loading: bool,
    /// Error message
    pub error: Option<String>,
    /// Refresh trigger counter
    refresh_trigger: u32,
}

impl GitState {
    pub fn new() -> Self {
        Self {
            path: None,
            repository_info: None,
            files: Vec::new(),
            selected_files: Vec::new(),
            commits: None,
            selected_commit: None,
            current_diff: None,
            branches: Vec::new(),
            tags: Vec::new(),
            stashes: Vec::new(),
            conflict_info: None,
            is_loading: false,
            error: None,
            refresh_trigger: 0,
        }
    }

    pub fn open_repository(&mut self, path: &Path, cx: &mut Context<Self>) -> Result<()> {
        self.is_loading = true;
        cx.notify();

        // Open the repository using git2
        let mut repo = git2::Repository::open(path)?;

        // Get repository info
        self.path = Some(path.to_path_buf());
        self.repository_info = Some(RepositoryInfo::from_repo(&repo)?);

        // Get file status
        self.files = FileStatus::get_all(&repo)?;

        // Get branches
        self.branches = BranchInfo::get_all(&repo)?;

        // Get tags
        self.tags = TagInfo::get_all(&repo)?;

        // Get stashes
        self.stashes = StashEntry::get_all(&mut repo)?;

        // Get commit graph (first 100 commits)
        self.commits = Some(CommitGraphData::build(&repo, 100, 0)?);

        // Check for conflicts
        self.conflict_info = ConflictInfo::get(&repo)?;

        self.is_loading = false;
        self.error = None;
        cx.notify();
        Ok(())
    }

    pub fn close_repository(&mut self, cx: &mut Context<Self>) {
        self.path = None;
        self.repository_info = None;
        self.files.clear();
        self.selected_files.clear();
        self.commits = None;
        self.selected_commit = None;
        self.current_diff = None;
        self.branches.clear();
        self.tags.clear();
        self.stashes.clear();
        self.conflict_info = None;
        self.is_loading = false;
        self.error = None;
        cx.notify();
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if let Some(path) = self.path.clone() {
            if let Err(e) = self.open_repository(&path, cx) {
                self.error = Some(e.to_string());
                cx.notify();
            }
        }
        self.refresh_trigger += 1;
        cx.notify();
    }

    fn with_repo<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&git2::Repository) -> Result<T>,
    {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No repository open"))?;
        let repo = git2::Repository::open(path)?;
        f(&repo)
    }

    fn with_repo_mut<F, T>(&mut self, f: F, cx: &mut Context<Self>) -> Result<T>
    where
        F: FnOnce(&git2::Repository) -> Result<T>,
    {
        let result = self.with_repo(f)?;
        self.refresh(cx);
        Ok(result)
    }

    // File operations
    pub fn stage_file(&mut self, path: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut index = repo.index()?;
                index.add_path(Path::new(path))?;
                index.write()?;
                Ok(())
            },
            cx,
        )
    }

    pub fn unstage_file(&mut self, path: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let head = repo.head()?.peel_to_commit()?;
                repo.reset_default(Some(&head.into_object()), [Path::new(path)])?;
                Ok(())
            },
            cx,
        )
    }

    pub fn stage_all(&mut self, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut index = repo.index()?;
                index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
                index.write()?;
                Ok(())
            },
            cx,
        )
    }

    pub fn unstage_all(&mut self, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let head = repo.head()?.peel_to_commit()?;
                repo.reset(&head.into_object(), git2::ResetType::Mixed, None)?;
                Ok(())
            },
            cx,
        )
    }

    pub fn discard_file(&mut self, path: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut checkout_opts = git2::build::CheckoutBuilder::new();
                checkout_opts.force();
                checkout_opts.path(path);
                repo.checkout_head(Some(&mut checkout_opts))?;
                Ok(())
            },
            cx,
        )
    }

    pub fn discard_all(&mut self, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut checkout_opts = git2::build::CheckoutBuilder::new();
                checkout_opts.force();
                repo.checkout_head(Some(&mut checkout_opts))?;
                Ok(())
            },
            cx,
        )
    }

    // Commit operations
    pub fn create_commit(&mut self, message: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let sig = repo.signature()?;
                let mut index = repo.index()?;
                let tree_id = index.write_tree()?;
                let tree = repo.find_tree(tree_id)?;
                let parent = repo.head()?.peel_to_commit()?;
                repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;
                Ok(())
            },
            cx,
        )
    }

    pub fn amend_commit(&mut self, message: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut index = repo.index()?;
                let tree_id = index.write_tree()?;
                let tree = repo.find_tree(tree_id)?;
                let head = repo.head()?.peel_to_commit()?;
                head.amend(Some("HEAD"), None, None, None, Some(message), Some(&tree))?;
                Ok(())
            },
            cx,
        )
    }

    // Remote operations
    pub fn push(&mut self, auth: Option<&GitCredentials>, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut remote = repo.find_remote("origin")?;
                let head = repo.head()?;
                let branch_name = head.shorthand().unwrap_or("HEAD");

                let mut callbacks = git2::RemoteCallbacks::new();
                if let Some(creds) = auth {
                    let username = creds.username.clone();
                    let password = creds.password.clone();
                    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
                        git2::Cred::userpass_plaintext(&username, &password)
                    });
                }

                let mut push_opts = git2::PushOptions::new();
                push_opts.remote_callbacks(callbacks);

                let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
                remote.push(&[&refspec], Some(&mut push_opts))?;
                Ok(())
            },
            cx,
        )
    }

    pub fn pull(&mut self, auth: Option<&GitCredentials>, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut remote = repo.find_remote("origin")?;
                let head = repo.head()?;
                let branch_name = head.shorthand().unwrap_or("HEAD");

                let mut callbacks = git2::RemoteCallbacks::new();
                if let Some(creds) = auth {
                    let username = creds.username.clone();
                    let password = creds.password.clone();
                    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
                        git2::Cred::userpass_plaintext(&username, &password)
                    });
                }

                let mut fetch_opts = git2::FetchOptions::new();
                fetch_opts.remote_callbacks(callbacks);

                remote.fetch(&[branch_name], Some(&mut fetch_opts), None)?;

                // Merge
                let fetch_head = repo.find_reference("FETCH_HEAD")?;
                let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
                let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;

                if analysis.is_fast_forward() {
                    let mut reference =
                        repo.find_reference(&format!("refs/heads/{}", branch_name))?;
                    reference.set_target(fetch_commit.id(), "Fast-forward")?;
                    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
                } else if analysis.is_normal() {
                    repo.merge(&[&fetch_commit], None, None)?;
                }

                Ok(())
            },
            cx,
        )
    }

    pub fn fetch(&mut self, auth: Option<&GitCredentials>, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut remote = repo.find_remote("origin")?;

                let mut callbacks = git2::RemoteCallbacks::new();
                if let Some(creds) = auth {
                    let username = creds.username.clone();
                    let password = creds.password.clone();
                    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
                        git2::Cred::userpass_plaintext(&username, &password)
                    });
                }

                let mut fetch_opts = git2::FetchOptions::new();
                fetch_opts.remote_callbacks(callbacks);

                remote.fetch::<&str>(&[], Some(&mut fetch_opts), None)?;
                Ok(())
            },
            cx,
        )
    }

    // Branch operations
    pub fn checkout_branch(&mut self, name: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let obj = repo.revparse_single(&format!("refs/heads/{}", name))?;
                repo.checkout_tree(&obj, None)?;
                repo.set_head(&format!("refs/heads/{}", name))?;
                Ok(())
            },
            cx,
        )
    }

    pub fn checkout_commit(&mut self, sha: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let oid = git2::Oid::from_str(sha)?;
                let commit = repo.find_commit(oid)?;
                repo.checkout_tree(&commit.into_object(), None)?;
                repo.set_head_detached(oid)?;
                Ok(())
            },
            cx,
        )
    }

    pub fn create_branch(&mut self, name: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let head = repo.head()?.peel_to_commit()?;
                repo.branch(name, &head, false)?;
                Ok(())
            },
            cx,
        )
    }

    pub fn delete_branch(&mut self, name: &str, force: bool, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut branch = repo.find_branch(name, git2::BranchType::Local)?;
                if force || !branch.is_head() {
                    branch.delete()?;
                } else {
                    anyhow::bail!("Cannot delete current branch");
                }
                Ok(())
            },
            cx,
        )
    }

    // Tag operations
    pub fn create_tag(
        &mut self,
        name: &str,
        sha: &str,
        message: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                if let Some(msg) = message {
                    TagInfo::create_annotated(repo, name, Some(sha), msg)?;
                } else {
                    TagInfo::create_lightweight(repo, name, Some(sha))?;
                }
                Ok(())
            },
            cx,
        )?;
        // Refresh tag list
        if let Some(path) = &self.path {
            let repo = git2::Repository::open(path)?;
            self.tags = TagInfo::get_all(&repo)?;
            cx.notify();
        }
        Ok(())
    }

    pub fn delete_tag(&mut self, name: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                TagInfo::delete(repo, name)?;
                Ok(())
            },
            cx,
        )?;
        // Refresh tag list
        if let Some(path) = &self.path {
            let repo = git2::Repository::open(path)?;
            self.tags = TagInfo::get_all(&repo)?;
            cx.notify();
        }
        Ok(())
    }

    // Stash operations
    pub fn stash_save(&mut self, message: Option<&str>, cx: &mut Context<Self>) -> Result<()> {
        if let Some(path) = &self.path {
            let mut repo = git2::Repository::open(path)?;
            StashEntry::save(&mut repo, message)?;
            // Refresh stash list
            self.stashes = StashEntry::get_all(&mut repo)?;
            cx.notify();
        }
        Ok(())
    }

    pub fn stash_pop(&mut self, index: usize, cx: &mut Context<Self>) -> Result<()> {
        if let Some(path) = &self.path {
            let mut repo = git2::Repository::open(path)?;
            StashEntry::pop(&mut repo, index)?;
            // Refresh stash list and files
            self.stashes = StashEntry::get_all(&mut repo)?;
            self.files = FileStatus::get_all(&repo)?;
            cx.notify();
        }
        Ok(())
    }

    pub fn stash_apply(&mut self, index: usize, cx: &mut Context<Self>) -> Result<()> {
        if let Some(path) = &self.path {
            let mut repo = git2::Repository::open(path)?;
            StashEntry::apply(&mut repo, index)?;
            // Refresh files (stash list stays the same)
            self.files = FileStatus::get_all(&repo)?;
            cx.notify();
        }
        Ok(())
    }

    pub fn stash_drop(&mut self, index: usize, cx: &mut Context<Self>) -> Result<()> {
        if let Some(path) = &self.path {
            let mut repo = git2::Repository::open(path)?;
            StashEntry::drop(&mut repo, index)?;
            // Refresh stash list
            self.stashes = StashEntry::get_all(&mut repo)?;
            cx.notify();
        }
        Ok(())
    }

    // Selection
    pub fn toggle_file_selection(&mut self, path: &str, cx: &mut Context<Self>) {
        if let Some(pos) = self.selected_files.iter().position(|p| p == path) {
            self.selected_files.remove(pos);
        } else {
            self.selected_files.push(path.to_string());
        }
        cx.notify();
    }

    pub fn select_all_files(&mut self, cx: &mut Context<Self>) {
        self.selected_files = self.files.iter().map(|f| f.path.clone()).collect();
        cx.notify();
    }

    pub fn deselect_all_files(&mut self, cx: &mut Context<Self>) {
        self.selected_files.clear();
        cx.notify();
    }

    pub fn set_selected_commit(&mut self, commit: Option<CommitInfo>, cx: &mut Context<Self>) {
        self.selected_commit = commit;
        cx.notify();
    }

    pub fn set_current_diff(&mut self, diff: Option<FileDiff>, cx: &mut Context<Self>) {
        self.current_diff = diff;
        cx.notify();
    }

    pub fn load_file_diff(&mut self, path: &str, cx: &mut Context<Self>) -> Result<()> {
        let diff = self.with_repo(|repo| FileDiff::get_file_diff(repo, path))?;
        self.current_diff = Some(diff);
        cx.notify();
        Ok(())
    }

    pub fn clear_diff(&mut self, cx: &mut Context<Self>) {
        self.current_diff = None;
        cx.notify();
    }

    // Load more commits
    pub fn load_more_commits(&mut self, cx: &mut Context<Self>) -> Result<()> {
        if let Some(path) = &self.path {
            let repo = git2::Repository::open(path)?;
            let current_count = self.commits.as_ref().map(|c| c.nodes.len()).unwrap_or(0);
            let more_commits = CommitGraphData::build(&repo, 100, current_count)?;

            if let Some(ref mut commits) = self.commits {
                commits.nodes.extend(more_commits.nodes);
                commits.edges.extend(more_commits.edges);
            }
            cx.notify();
        }
        Ok(())
    }

    // Getters
    pub fn staged_files(&self) -> Vec<&FileStatus> {
        self.files.iter().filter(|f| f.staged).collect()
    }

    pub fn unstaged_files(&self) -> Vec<&FileStatus> {
        self.files.iter().filter(|f| !f.staged).collect()
    }

    pub fn is_detached(&self) -> bool {
        self.repository_info
            .as_ref()
            .map(|r| r.is_detached)
            .unwrap_or(false)
    }

    pub fn current_branch(&self) -> Option<&str> {
        self.repository_info
            .as_ref()
            .and_then(|r| r.current_branch.as_deref())
    }

    // Conflict resolution
    pub fn resolve_all_conflicts(
        &mut self,
        strategy: ConflictStrategy,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                ConflictInfo::resolve_all(repo, strategy)?;
                Ok(())
            },
            cx,
        )
    }

    pub fn resolve_conflicts_per_file(
        &mut self,
        resolutions: Vec<(String, ConflictStrategy)>,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                let mut index = repo.index()?;
                let conflicts: Vec<_> = index.conflicts()?.collect::<Result<Vec<_>, _>>()?;

                for (path, strategy) in resolutions {
                    if let Some(conflict) = conflicts.iter().find(|c| {
                        c.our
                            .as_ref()
                            .or(c.their.as_ref())
                            .and_then(|e| std::str::from_utf8(&e.path).ok())
                            == Some(&path)
                    }) {
                        ConflictInfo::resolve_file(repo, &mut index, &path, conflict, strategy)?;
                    }
                }
                index.write()?;
                Ok(())
            },
            cx,
        )
    }

    pub fn complete_merge(
        &mut self,
        message: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                ConflictInfo::complete_merge(repo, message)?;
                Ok(())
            },
            cx,
        )
    }

    pub fn abort_merge(&mut self, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                ConflictInfo::abort_merge(repo)?;
                Ok(())
            },
            cx,
        )
    }

    // Advanced operations
    pub fn revert_commit(
        &mut self,
        sha: &str,
        mainline: Option<u32>,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                git::revert_commit(repo, sha, mainline)?;
                Ok(())
            },
            cx,
        )
    }

    pub fn cherry_pick(&mut self, sha: &str, cx: &mut Context<Self>) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                git::cherry_pick(repo, sha)?;
                Ok(())
            },
            cx,
        )
    }

    pub fn reset_to_commit(
        &mut self,
        sha: &str,
        mode: ResetMode,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        self.with_repo_mut(
            |repo| {
                git::reset_to_commit(repo, sha, mode)?;
                Ok(())
            },
            cx,
        )
    }

    /// Search commits by message, author, or SHA
    pub fn search_commits(&self, query: &str, limit: usize) -> Vec<CommitInfo> {
        let query = query.to_lowercase();

        if query.is_empty() {
            return Vec::new();
        }

        // Search in existing commits (loaded in memory)
        if let Some(commits) = &self.commits {
            commits
                .nodes
                .iter()
                .filter(|node| {
                    let commit = &node.commit;
                    commit.message.to_lowercase().contains(&query)
                        || commit.author.to_lowercase().contains(&query)
                        || commit.sha.to_lowercase().starts_with(&query)
                        || commit.short_sha.to_lowercase().starts_with(&query)
                })
                .take(limit)
                .map(|node| node.commit.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
}
