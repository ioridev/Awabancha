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
