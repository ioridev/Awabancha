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
