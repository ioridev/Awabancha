#![allow(dead_code)]

use chrono::{DateTime, Utc};
use gpui::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const MAX_RECENT_PROJECTS: usize = 10;

#[derive(Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: PathBuf,
    pub name: String,
    pub last_opened: DateTime<Utc>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RecentProjectsData {
    pub projects: Vec<RecentProject>,
}

pub struct RecentProjects {
    pub data: RecentProjectsData,
}

impl RecentProjects {
    fn storage_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("awabancha").join("recent_projects.json"))
    }

    pub fn load(_cx: &mut Context<Self>) -> Self {
        let data = Self::storage_path()
            .and_then(|path| fs::read_to_string(&path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default();

        Self { data }
    }

    pub fn save(&self, _cx: &mut Context<Self>) {
        if let Some(path) = Self::storage_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(content) = serde_json::to_string_pretty(&self.data) {
                let _ = fs::write(&path, content);
            }
        }
    }

    pub fn add_project(&mut self, path: PathBuf, name: String, cx: &mut Context<Self>) {
        // Remove if already exists
        self.data.projects.retain(|p| p.path != path);

        // Add to front
        self.data.projects.insert(
            0,
            RecentProject {
                path,
                name,
                last_opened: Utc::now(),
            },
        );

        // Trim to max size
        self.data.projects.truncate(MAX_RECENT_PROJECTS);

        self.save(cx);
        cx.notify();
    }

    pub fn remove_project(&mut self, path: &PathBuf, cx: &mut Context<Self>) {
        self.data.projects.retain(|p| &p.path != path);
        self.save(cx);
        cx.notify();
    }

    pub fn clear_all(&mut self, cx: &mut Context<Self>) {
        self.data.projects.clear();
        self.save(cx);
        cx.notify();
    }

    pub fn projects(&self) -> &[RecentProject] {
        &self.data.projects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_project_creation() {
        let project = RecentProject {
            path: PathBuf::from("/test/path"),
            name: "test-project".to_string(),
            last_opened: Utc::now(),
        };

        assert_eq!(project.path, PathBuf::from("/test/path"));
        assert_eq!(project.name, "test-project");
    }

    #[test]
    fn test_recent_projects_data_default() {
        let data = RecentProjectsData::default();
        assert!(data.projects.is_empty());
    }

    #[test]
    fn test_recent_project_serialization() {
        let project = RecentProject {
            path: PathBuf::from("/test/path"),
            name: "test-project".to_string(),
            last_opened: Utc::now(),
        };

        let json = serde_json::to_string(&project).unwrap();
        let deserialized: RecentProject = serde_json::from_str(&json).unwrap();

        assert_eq!(project.path, deserialized.path);
        assert_eq!(project.name, deserialized.name);
    }

    #[test]
    fn test_recent_projects_data_serialization() {
        let mut data = RecentProjectsData::default();
        data.projects.push(RecentProject {
            path: PathBuf::from("/test/path1"),
            name: "project1".to_string(),
            last_opened: Utc::now(),
        });
        data.projects.push(RecentProject {
            path: PathBuf::from("/test/path2"),
            name: "project2".to_string(),
            last_opened: Utc::now(),
        });

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: RecentProjectsData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.projects.len(), 2);
        assert_eq!(deserialized.projects[0].name, "project1");
        assert_eq!(deserialized.projects[1].name, "project2");
    }

    #[test]
    fn test_storage_path_is_some() {
        let path = RecentProjects::storage_path();
        assert!(path.is_some(), "Storage path should be available in test environment");
        let p = path.unwrap();
        assert!(p.to_string_lossy().contains("awabancha"));
        assert!(p.ends_with("recent_projects.json"));
    }

    #[test]
    fn test_max_recent_projects_constant() {
        assert_eq!(MAX_RECENT_PROJECTS, 10);
    }
}
