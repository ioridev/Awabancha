#![allow(dead_code)]

use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use git2::{Oid, Repository, Sort};
use std::collections::HashMap;

/// Single commit information
#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub sha: String,
    pub short_sha: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub timestamp: DateTime<Utc>,
    pub parents: Vec<String>,
    /// Branch at this commit (if any)
    pub branch: Option<String>,
    /// All branches pointing to this commit
    pub branches: Vec<String>,
    /// Remote references pointing to this commit
    pub remotes: Vec<String>,
    /// Tags pointing to this commit
    pub tags: Vec<String>,
}

impl CommitInfo {
    pub fn from_commit(
        commit: &git2::Commit,
        branches_map: &HashMap<Oid, Vec<String>>,
        remotes_map: &HashMap<Oid, Vec<String>>,
        tags_map: &HashMap<Oid, Vec<String>>,
    ) -> Self {
        let sha = commit.id().to_string();
        let short_sha = sha[..7].to_string();

        let timestamp = Utc
            .timestamp_opt(commit.time().seconds(), 0)
            .single()
            .unwrap_or_else(Utc::now);

        let parents = commit.parents().map(|p| p.id().to_string()).collect();

        let oid = commit.id();
        let branches = branches_map.get(&oid).cloned().unwrap_or_default();
        let remotes = remotes_map.get(&oid).cloned().unwrap_or_default();
        let tags = tags_map.get(&oid).cloned().unwrap_or_default();

        Self {
            sha,
            short_sha,
            message: commit.summary().unwrap_or("").to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            email: commit.author().email().unwrap_or("").to_string(),
            timestamp,
            parents,
            branch: branches.first().cloned(),
            branches,
            remotes,
            tags,
        }
    }

    pub fn relative_time(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.timestamp);

        if duration.num_days() > 365 {
            format!("{} years ago", duration.num_days() / 365)
        } else if duration.num_days() > 30 {
            format!("{} months ago", duration.num_days() / 30)
        } else if duration.num_days() > 7 {
            format!("{} weeks ago", duration.num_days() / 7)
        } else if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            "just now".to_string()
        }
    }
}

/// Graph edge type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EdgeType {
    Linear,
    Branch,
    Merge,
}

/// Edge in the commit graph
#[derive(Clone, Debug)]
pub struct GraphEdge {
    pub from_sha: String,
    pub to_sha: String,
    pub from_column: usize,
    pub to_column: usize,
    pub from_row: usize,
    pub to_row: usize,
    pub color: u32,
    pub edge_type: EdgeType,
}

/// Node in the commit graph
#[derive(Clone, Debug)]
pub struct GraphNode {
    pub commit: CommitInfo,
    pub column: usize,
    pub row: usize,
    pub color: u32,
}

/// Complete commit graph data
#[derive(Clone, Debug)]
pub struct CommitGraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub max_column: usize,
}

impl CommitGraphData {
    /// Build commit graph from repository
    pub fn build(repo: &Repository, limit: usize, offset: usize) -> Result<Self> {
        let mut revwalk = repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
        revwalk.push_head()?;

        // Also include all branches
        for branch in repo.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch?;
            if let Some(oid) = branch.get().target() {
                let _ = revwalk.push(oid);
            }
        }

        // Build reference maps
        let branches_map = Self::build_branches_map(repo)?;
        let remotes_map = Self::build_remotes_map(repo)?;
        let tags_map = Self::build_tags_map(repo)?;

        // Collect commits
        let mut commits: Vec<git2::Commit> = Vec::new();
        for (i, oid_result) in revwalk.enumerate() {
            if i < offset {
                continue;
            }
            if i >= offset + limit {
                break;
            }
            if let Ok(oid) = oid_result {
                if let Ok(commit) = repo.find_commit(oid) {
                    commits.push(commit);
                }
            }
        }

        // Build graph layout
        let (nodes, edges, max_column) =
            Self::layout_graph(&commits, &branches_map, &remotes_map, &tags_map);

        Ok(Self {
            nodes,
            edges,
            max_column,
        })
    }

    fn build_branches_map(repo: &Repository) -> Result<HashMap<Oid, Vec<String>>> {
        let mut map: HashMap<Oid, Vec<String>> = HashMap::new();
        for branch in repo.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch?;
            if let (Some(name), Some(oid)) = (branch.name()?, branch.get().target()) {
                map.entry(oid).or_default().push(name.to_string());
            }
        }
        Ok(map)
    }

    fn build_remotes_map(repo: &Repository) -> Result<HashMap<Oid, Vec<String>>> {
        let mut map: HashMap<Oid, Vec<String>> = HashMap::new();
        for branch in repo.branches(Some(git2::BranchType::Remote))? {
            let (branch, _) = branch?;
            if let (Some(name), Some(oid)) = (branch.name()?, branch.get().target()) {
                map.entry(oid).or_default().push(name.to_string());
            }
        }
        Ok(map)
    }

    fn build_tags_map(repo: &Repository) -> Result<HashMap<Oid, Vec<String>>> {
        let mut map: HashMap<Oid, Vec<String>> = HashMap::new();
        repo.tag_foreach(|oid, name| {
            let name = String::from_utf8_lossy(name)
                .trim_start_matches("refs/tags/")
                .to_string();
            // Resolve annotated tags to their target commit
            if let Ok(obj) = repo.find_object(oid, None) {
                let target_oid = if let Some(tag) = obj.as_tag() {
                    tag.target_id()
                } else {
                    oid
                };
                map.entry(target_oid).or_default().push(name);
            }
            true
        })?;
        Ok(map)
    }

    fn layout_graph(
        commits: &[git2::Commit],
        branches_map: &HashMap<Oid, Vec<String>>,
        remotes_map: &HashMap<Oid, Vec<String>>,
        tags_map: &HashMap<Oid, Vec<String>>,
    ) -> (Vec<GraphNode>, Vec<GraphEdge>, usize) {
        // Color palette for branches
        const COLORS: [u32; 8] = [
            0x89b4fa, // Blue
            0xa6e3a1, // Green
            0xf9e2af, // Yellow
            0xfab387, // Orange
            0xf38ba8, // Red
            0xcba6f7, // Purple
            0x94e2d5, // Teal
            0xf5c2e7, // Pink
        ];

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut max_column = 0;

        // Track active columns (which parent SHAs are in which columns)
        let mut active_columns: Vec<Option<String>> = Vec::new();
        // Map SHA to row index
        let mut sha_to_row: HashMap<String, usize> = HashMap::new();
        // Map SHA to column
        let mut sha_to_column: HashMap<String, usize> = HashMap::new();

        for (row, commit) in commits.iter().enumerate() {
            let sha = commit.id().to_string();
            sha_to_row.insert(sha.clone(), row);

            // Find or assign column for this commit
            let column = if let Some(col) = sha_to_column.get(&sha) {
                *col
            } else {
                // Find first available column or add new one
                let col = active_columns
                    .iter()
                    .position(|c| c.is_none())
                    .unwrap_or_else(|| {
                        active_columns.push(None);
                        active_columns.len() - 1
                    });
                sha_to_column.insert(sha.clone(), col);
                col
            };

            // Mark this column as available (commit is being rendered)
            if column < active_columns.len() {
                active_columns[column] = None;
            }

            max_column = max_column.max(column);

            let color = COLORS[column % COLORS.len()];

            // Create node
            let commit_info = CommitInfo::from_commit(commit, branches_map, remotes_map, tags_map);
            nodes.push(GraphNode {
                commit: commit_info,
                column,
                row,
                color,
            });

            // Process parents and create edges
            let parents: Vec<Oid> = commit.parents().map(|p| p.id()).collect();

            for (parent_idx, parent_oid) in parents.iter().enumerate() {
                let parent_sha = parent_oid.to_string();

                // Determine parent column
                let parent_column = if parent_idx == 0 {
                    // First parent stays in same column
                    sha_to_column.insert(parent_sha.clone(), column);
                    if column < active_columns.len() {
                        active_columns[column] = Some(parent_sha.clone());
                    }
                    column
                } else {
                    // Other parents get new columns
                    let new_col = active_columns
                        .iter()
                        .position(|c| c.is_none())
                        .unwrap_or_else(|| {
                            active_columns.push(None);
                            active_columns.len() - 1
                        });
                    sha_to_column.insert(parent_sha.clone(), new_col);
                    if new_col < active_columns.len() {
                        active_columns[new_col] = Some(parent_sha.clone());
                    } else {
                        active_columns.push(Some(parent_sha.clone()));
                    }
                    max_column = max_column.max(new_col);
                    new_col
                };

                // Find parent row if it exists
                let parent_row = sha_to_row.get(&parent_sha).copied().unwrap_or(row + 1);

                let edge_type = if parents.len() > 1 {
                    EdgeType::Merge
                } else if column != parent_column {
                    EdgeType::Branch
                } else {
                    EdgeType::Linear
                };

                edges.push(GraphEdge {
                    from_sha: sha.clone(),
                    to_sha: parent_sha,
                    from_column: column,
                    to_column: parent_column,
                    from_row: row,
                    to_row: parent_row,
                    color,
                    edge_type,
                });
            }
        }

        (nodes, edges, max_column)
    }
}

/// Reset mode for reset_to_commit
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
}

impl ResetMode {
    pub fn to_git2(self) -> git2::ResetType {
        match self {
            ResetMode::Soft => git2::ResetType::Soft,
            ResetMode::Mixed => git2::ResetType::Mixed,
            ResetMode::Hard => git2::ResetType::Hard,
        }
    }
}

/// Revert a commit (create an undo commit)
pub fn revert_commit(repo: &Repository, sha: &str, mainline: Option<u32>) -> Result<Oid> {
    let oid = git2::Oid::from_str(sha)?;
    let commit = repo.find_commit(oid)?;

    // For merge commits, mainline specifies which parent to consider as the mainline
    let mut revert_opts = git2::RevertOptions::new();
    if let Some(m) = mainline {
        revert_opts.mainline(m);
    }

    repo.revert(&commit, Some(&mut revert_opts))?;

    // Create the revert commit
    let sig = repo.signature()?;
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let head = repo.head()?.peel_to_commit()?;

    let message = format!("Revert \"{}\"\n\nThis reverts commit {}.",
        commit.summary().unwrap_or(""),
        &sha[..7]);

    let new_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &message,
        &tree,
        &[&head],
    )?;

    // Clean up state
    repo.cleanup_state()?;

    Ok(new_commit)
}

/// Cherry-pick a commit
pub fn cherry_pick(repo: &Repository, sha: &str) -> Result<Oid> {
    let oid = git2::Oid::from_str(sha)?;
    let commit = repo.find_commit(oid)?;

    repo.cherrypick(&commit, None)?;

    // Check for conflicts
    let index = repo.index()?;
    if index.has_conflicts() {
        anyhow::bail!("Cherry-pick resulted in conflicts. Resolve them and commit manually.");
    }

    // Create the cherry-pick commit
    let sig = repo.signature()?;
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let head = repo.head()?.peel_to_commit()?;

    let message = format!(
        "{}\n\n(cherry picked from commit {})",
        commit.message().unwrap_or(""),
        &sha[..7]
    );

    let new_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &message,
        &tree,
        &[&head],
    )?;

    // Clean up state
    repo.cleanup_state()?;

    Ok(new_commit)
}

/// Reset HEAD to a specific commit
pub fn reset_to_commit(repo: &Repository, sha: &str, mode: ResetMode) -> Result<()> {
    let oid = git2::Oid::from_str(sha)?;
    let commit = repo.find_commit(oid)?;
    let obj = commit.into_object();

    repo.reset(&obj, mode.to_git2(), None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use chrono::Duration;

    fn create_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        (temp_dir, repo)
    }

    fn create_initial_commit(temp_dir: &TempDir, repo: &Repository) -> Oid {
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

    fn create_second_commit(temp_dir: &TempDir, repo: &Repository) -> Oid {
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "second content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let sig = repo.signature().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let head = repo.head().unwrap().peel_to_commit().unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Second commit", &tree, &[&head])
            .unwrap()
    }

    #[test]
    fn test_reset_mode_to_git2() {
        assert!(matches!(
            ResetMode::Soft.to_git2(),
            git2::ResetType::Soft
        ));
        assert!(matches!(
            ResetMode::Mixed.to_git2(),
            git2::ResetType::Mixed
        ));
        assert!(matches!(
            ResetMode::Hard.to_git2(),
            git2::ResetType::Hard
        ));
    }

    #[test]
    fn test_reset_mode_equality() {
        assert_eq!(ResetMode::Soft, ResetMode::Soft);
        assert_ne!(ResetMode::Soft, ResetMode::Hard);
    }

    #[test]
    fn test_reset_mode_clone() {
        let mode = ResetMode::Mixed;
        let cloned = mode;
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_edge_type_equality() {
        assert_eq!(EdgeType::Linear, EdgeType::Linear);
        assert_ne!(EdgeType::Linear, EdgeType::Branch);
        assert_ne!(EdgeType::Branch, EdgeType::Merge);
    }

    #[test]
    fn test_edge_type_clone() {
        let edge_type = EdgeType::Merge;
        let cloned = edge_type;
        assert_eq!(edge_type, cloned);
    }

    #[test]
    fn test_commit_info_creation() {
        let commit_info = CommitInfo {
            sha: "abcdef1234567890".to_string(),
            short_sha: "abcdef1".to_string(),
            message: "Test commit".to_string(),
            author: "Test Author".to_string(),
            email: "test@example.com".to_string(),
            timestamp: Utc::now(),
            parents: vec!["parent1".to_string()],
            branch: Some("main".to_string()),
            branches: vec!["main".to_string()],
            remotes: vec![],
            tags: vec!["v1.0".to_string()],
        };

        assert_eq!(commit_info.sha, "abcdef1234567890");
        assert_eq!(commit_info.short_sha, "abcdef1");
        assert_eq!(commit_info.message, "Test commit");
        assert_eq!(commit_info.author, "Test Author");
        assert_eq!(commit_info.email, "test@example.com");
        assert_eq!(commit_info.parents.len(), 1);
        assert_eq!(commit_info.branch, Some("main".to_string()));
        assert_eq!(commit_info.tags.len(), 1);
    }

    #[test]
    fn test_commit_info_clone() {
        let commit_info = CommitInfo {
            sha: "abc123".to_string(),
            short_sha: "abc123".to_string(),
            message: "Test".to_string(),
            author: "Author".to_string(),
            email: "email@test.com".to_string(),
            timestamp: Utc::now(),
            parents: vec![],
            branch: None,
            branches: vec![],
            remotes: vec![],
            tags: vec![],
        };

        let cloned = commit_info.clone();
        assert_eq!(commit_info.sha, cloned.sha);
        assert_eq!(commit_info.message, cloned.message);
    }

    #[test]
    fn test_commit_info_short_sha() {
        let sha = "abcdef1234567890abcdef1234567890abcdef12";
        let short = &sha[..7];
        assert_eq!(short, "abcdef1");
    }

    #[test]
    fn test_commit_info_relative_time_just_now() {
        let commit_info = CommitInfo {
            sha: "abc".to_string(),
            short_sha: "abc".to_string(),
            message: "test".to_string(),
            author: "author".to_string(),
            email: "email@test.com".to_string(),
            timestamp: Utc::now(),
            parents: vec![],
            branch: None,
            branches: vec![],
            remotes: vec![],
            tags: vec![],
        };

        let relative = commit_info.relative_time();
        assert!(relative == "just now" || relative.contains("minute"));
    }

    #[test]
    fn test_commit_info_relative_time_hours() {
        let commit_info = CommitInfo {
            sha: "abc".to_string(),
            short_sha: "abc".to_string(),
            message: "test".to_string(),
            author: "author".to_string(),
            email: "email@test.com".to_string(),
            timestamp: Utc::now() - Duration::hours(5),
            parents: vec![],
            branch: None,
            branches: vec![],
            remotes: vec![],
            tags: vec![],
        };

        let relative = commit_info.relative_time();
        assert!(relative.contains("hours ago"));
    }

    #[test]
    fn test_commit_info_relative_time_days() {
        let commit_info = CommitInfo {
            sha: "abc".to_string(),
            short_sha: "abc".to_string(),
            message: "test".to_string(),
            author: "author".to_string(),
            email: "email@test.com".to_string(),
            timestamp: Utc::now() - Duration::days(3),
            parents: vec![],
            branch: None,
            branches: vec![],
            remotes: vec![],
            tags: vec![],
        };

        let relative = commit_info.relative_time();
        assert!(relative.contains("days ago"));
    }

    #[test]
    fn test_commit_info_relative_time_weeks() {
        let commit_info = CommitInfo {
            sha: "abc".to_string(),
            short_sha: "abc".to_string(),
            message: "test".to_string(),
            author: "author".to_string(),
            email: "email@test.com".to_string(),
            timestamp: Utc::now() - Duration::weeks(2),
            parents: vec![],
            branch: None,
            branches: vec![],
            remotes: vec![],
            tags: vec![],
        };

        let relative = commit_info.relative_time();
        assert!(relative.contains("weeks ago"));
    }

    #[test]
    fn test_commit_info_relative_time_months() {
        let commit_info = CommitInfo {
            sha: "abc".to_string(),
            short_sha: "abc".to_string(),
            message: "test".to_string(),
            author: "author".to_string(),
            email: "email@test.com".to_string(),
            timestamp: Utc::now() - Duration::days(60),
            parents: vec![],
            branch: None,
            branches: vec![],
            remotes: vec![],
            tags: vec![],
        };

        let relative = commit_info.relative_time();
        assert!(relative.contains("months ago"));
    }

    #[test]
    fn test_commit_info_relative_time_years() {
        let commit_info = CommitInfo {
            sha: "abc".to_string(),
            short_sha: "abc".to_string(),
            message: "test".to_string(),
            author: "author".to_string(),
            email: "email@test.com".to_string(),
            timestamp: Utc::now() - Duration::days(400),
            parents: vec![],
            branch: None,
            branches: vec![],
            remotes: vec![],
            tags: vec![],
        };

        let relative = commit_info.relative_time();
        assert!(relative.contains("years ago"));
    }

    #[test]
    fn test_graph_node_creation() {
        let node = GraphNode {
            commit: CommitInfo {
                sha: "abc1234".to_string(),
                short_sha: "abc1234".to_string(),
                message: "Test commit".to_string(),
                author: "Test Author".to_string(),
                email: "test@example.com".to_string(),
                timestamp: Utc::now(),
                parents: vec![],
                branch: None,
                branches: vec![],
                remotes: vec![],
                tags: vec![],
            },
            column: 0,
            row: 0,
            color: 0x89b4fa,
        };

        assert_eq!(node.column, 0);
        assert_eq!(node.row, 0);
        assert_eq!(node.commit.message, "Test commit");
    }

    #[test]
    fn test_graph_node_clone() {
        let node = GraphNode {
            commit: CommitInfo {
                sha: "abc".to_string(),
                short_sha: "abc".to_string(),
                message: "Test".to_string(),
                author: "Author".to_string(),
                email: "email".to_string(),
                timestamp: Utc::now(),
                parents: vec![],
                branch: None,
                branches: vec![],
                remotes: vec![],
                tags: vec![],
            },
            column: 1,
            row: 2,
            color: 0xffffff,
        };

        let cloned = node.clone();
        assert_eq!(node.column, cloned.column);
        assert_eq!(node.row, cloned.row);
        assert_eq!(node.color, cloned.color);
    }

    #[test]
    fn test_graph_edge_creation() {
        let edge = GraphEdge {
            from_sha: "abc1234".to_string(),
            to_sha: "def5678".to_string(),
            from_column: 0,
            to_column: 1,
            from_row: 0,
            to_row: 1,
            color: 0xa6e3a1,
            edge_type: EdgeType::Branch,
        };

        assert_eq!(edge.from_column, 0);
        assert_eq!(edge.to_column, 1);
        assert_eq!(edge.edge_type, EdgeType::Branch);
    }

    #[test]
    fn test_graph_edge_clone() {
        let edge = GraphEdge {
            from_sha: "abc".to_string(),
            to_sha: "def".to_string(),
            from_column: 0,
            to_column: 0,
            from_row: 0,
            to_row: 1,
            color: 0x123456,
            edge_type: EdgeType::Linear,
        };

        let cloned = edge.clone();
        assert_eq!(edge.from_sha, cloned.from_sha);
        assert_eq!(edge.to_sha, cloned.to_sha);
        assert_eq!(edge.edge_type, cloned.edge_type);
    }

    #[test]
    fn test_commit_graph_data_build() {
        let (temp_dir, repo) = create_test_repo();
        create_initial_commit(&temp_dir, &repo);
        create_second_commit(&temp_dir, &repo);

        let graph = CommitGraphData::build(&repo, 10, 0).unwrap();
        assert!(!graph.nodes.is_empty());
        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn test_commit_graph_data_clone() {
        let graph = CommitGraphData {
            nodes: vec![],
            edges: vec![],
            max_column: 0,
        };

        let cloned = graph.clone();
        assert_eq!(graph.max_column, cloned.max_column);
    }

    #[test]
    fn test_reset_to_commit() {
        let (temp_dir, repo) = create_test_repo();
        let first_oid = create_initial_commit(&temp_dir, &repo);
        create_second_commit(&temp_dir, &repo);

        // Reset to first commit
        let result = reset_to_commit(&repo, &first_oid.to_string(), ResetMode::Hard);
        assert!(result.is_ok());

        // Verify HEAD is at first commit
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.id(), first_oid);
    }

    #[test]
    fn test_reset_to_commit_soft() {
        let (temp_dir, repo) = create_test_repo();
        let first_oid = create_initial_commit(&temp_dir, &repo);
        create_second_commit(&temp_dir, &repo);

        let result = reset_to_commit(&repo, &first_oid.to_string(), ResetMode::Soft);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reset_to_commit_mixed() {
        let (temp_dir, repo) = create_test_repo();
        let first_oid = create_initial_commit(&temp_dir, &repo);
        create_second_commit(&temp_dir, &repo);

        let result = reset_to_commit(&repo, &first_oid.to_string(), ResetMode::Mixed);
        assert!(result.is_ok());
    }
}
