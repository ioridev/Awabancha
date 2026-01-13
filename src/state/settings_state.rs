#![allow(dead_code)]

use crate::i18n::Locale;
use crate::state::GitCredentials;
use gpui::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMode {
    Https,
    Ssh,
}

impl Default for AuthMode {
    fn default() -> Self {
        Self::Https
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeMode {
    Auto,
    FfOnly,
    NoFf,
    Squash,
}

impl Default for MergeMode {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SettingsData {
    pub git_auth_mode: AuthMode,
    pub git_username: Option<String>,
    pub git_token: Option<String>,
    pub merge_mode: MergeMode,
    pub theme: Theme,
    pub locale: Locale,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            git_auth_mode: AuthMode::default(),
            git_username: None,
            git_token: None,
            merge_mode: MergeMode::default(),
            theme: Theme::default(),
            locale: Locale::default(),
        }
    }
}

pub struct SettingsState {
    pub data: SettingsData,
}

impl SettingsState {
    fn settings_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("awabancha").join("settings.json"))
    }

    pub fn load(_cx: &mut Context<Self>) -> Self {
        let data = Self::settings_path()
            .and_then(|path| fs::read_to_string(&path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default();

        Self { data }
    }

    pub fn save(&self, _cx: &mut Context<Self>) {
        if let Some(path) = Self::settings_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(content) = serde_json::to_string_pretty(&self.data) {
                let _ = fs::write(&path, content);
            }
        }
    }

    pub fn get_auth_credentials(&self) -> Option<GitCredentials> {
        match self.data.git_auth_mode {
            AuthMode::Https => {
                let username = self.data.git_username.clone()?;
                let password = self.data.git_token.clone()?;
                Some(GitCredentials { username, password })
            }
            AuthMode::Ssh => None, // SSH uses agent
        }
    }

    // Setters
    pub fn set_auth_mode(&mut self, mode: AuthMode, cx: &mut Context<Self>) {
        self.data.git_auth_mode = mode;
        self.save(cx);
        cx.notify();
    }

    pub fn set_username(&mut self, username: Option<String>, cx: &mut Context<Self>) {
        self.data.git_username = username;
        self.save(cx);
        cx.notify();
    }

    pub fn set_token(&mut self, token: Option<String>, cx: &mut Context<Self>) {
        self.data.git_token = token;
        self.save(cx);
        cx.notify();
    }

    pub fn set_merge_mode(&mut self, mode: MergeMode, cx: &mut Context<Self>) {
        self.data.merge_mode = mode;
        self.save(cx);
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.data.theme = theme;
        self.save(cx);
        cx.notify();
    }

    pub fn set_locale(&mut self, locale: Locale, cx: &mut Context<Self>) {
        self.data.locale = locale;
        self.save(cx);
        cx.notify();
    }
}
