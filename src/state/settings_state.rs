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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_mode_default() {
        let default: AuthMode = Default::default();
        assert_eq!(default, AuthMode::Https);
    }

    #[test]
    fn test_auth_mode_equality() {
        assert_eq!(AuthMode::Https, AuthMode::Https);
        assert_eq!(AuthMode::Ssh, AuthMode::Ssh);
        assert_ne!(AuthMode::Https, AuthMode::Ssh);
    }

    #[test]
    fn test_merge_mode_default() {
        let default: MergeMode = Default::default();
        assert_eq!(default, MergeMode::Auto);
    }

    #[test]
    fn test_merge_mode_equality() {
        assert_eq!(MergeMode::Auto, MergeMode::Auto);
        assert_eq!(MergeMode::FfOnly, MergeMode::FfOnly);
        assert_eq!(MergeMode::NoFf, MergeMode::NoFf);
        assert_eq!(MergeMode::Squash, MergeMode::Squash);
        assert_ne!(MergeMode::Auto, MergeMode::Squash);
    }

    #[test]
    fn test_theme_default() {
        let default: Theme = Default::default();
        assert_eq!(default, Theme::Dark);
    }

    #[test]
    fn test_theme_equality() {
        assert_eq!(Theme::Dark, Theme::Dark);
        assert_eq!(Theme::Light, Theme::Light);
        assert_ne!(Theme::Dark, Theme::Light);
    }

    #[test]
    fn test_settings_data_default() {
        let settings = SettingsData::default();
        assert_eq!(settings.git_auth_mode, AuthMode::Https);
        assert!(settings.git_username.is_none());
        assert!(settings.git_token.is_none());
        assert_eq!(settings.merge_mode, MergeMode::Auto);
        assert_eq!(settings.theme, Theme::Dark);
        assert_eq!(settings.locale, Locale::En);
    }

    #[test]
    fn test_settings_data_serialization() {
        let settings = SettingsData::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: SettingsData = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.git_auth_mode, deserialized.git_auth_mode);
        assert_eq!(settings.merge_mode, deserialized.merge_mode);
        assert_eq!(settings.theme, deserialized.theme);
        assert_eq!(settings.locale, deserialized.locale);
    }

    #[test]
    fn test_settings_data_with_values() {
        let settings = SettingsData {
            git_auth_mode: AuthMode::Ssh,
            git_username: Some("testuser".to_string()),
            git_token: Some("testtoken".to_string()),
            merge_mode: MergeMode::FfOnly,
            theme: Theme::Light,
            locale: Locale::Ja,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: SettingsData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.git_auth_mode, AuthMode::Ssh);
        assert_eq!(deserialized.git_username, Some("testuser".to_string()));
        assert_eq!(deserialized.git_token, Some("testtoken".to_string()));
        assert_eq!(deserialized.merge_mode, MergeMode::FfOnly);
        assert_eq!(deserialized.theme, Theme::Light);
        assert_eq!(deserialized.locale, Locale::Ja);
    }

    #[test]
    fn test_settings_path_is_some() {
        // settings_path should return Some on systems with a config directory
        let path = SettingsState::settings_path();
        assert!(path.is_some(), "Settings path should be available in test environment");
        let p = path.unwrap();
        assert!(p.to_string_lossy().contains("awabancha"));
        assert!(p.ends_with("settings.json"));
    }
}
