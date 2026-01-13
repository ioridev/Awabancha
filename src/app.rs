use crate::actions::*;
use crate::state::{GitState, RecentProjects, SettingsState};
use crate::views::{MainLayout, WelcomeView};
use gpui::prelude::*;
use gpui::*;
use std::path::PathBuf;

pub struct Assets;

impl Global for Assets {}

impl Assets {
    pub fn new() -> Self {
        Self
    }
}

pub struct Awabancha {
    /// Current repository path (None = show welcome screen)
    pub repository_path: Option<PathBuf>,
    /// Git state (repository info, commits, files)
    pub git_state: Entity<GitState>,
    /// Application settings
    pub settings: Entity<SettingsState>,
    /// Recent projects list
    pub recent_projects: Entity<RecentProjects>,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Show settings modal
    pub show_settings: bool,
    /// Main layout entity (created when repository is opened)
    main_layout: Option<Entity<MainLayout>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Welcome,
    Repository,
}

impl Awabancha {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let git_state = cx.new(|_| GitState::new());
        let settings = cx.new(|cx| SettingsState::load(cx));
        let recent_projects = cx.new(|cx| RecentProjects::load(cx));

        Self {
            repository_path: None,
            git_state,
            settings,
            recent_projects,
            view_mode: ViewMode::Welcome,
            show_settings: false,
            main_layout: None,
        }
    }

    pub fn load_assets() -> Assets {
        Assets::new()
    }

    pub fn open_repository(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        // Update recent projects
        self.recent_projects.update(cx, |recent, cx| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            recent.add_project(path.clone(), name, cx);
        });

        // Open the repository
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.open_repository(&path, cx) {
                log::error!("Failed to open repository: {}", e);
            }
        });

        // Create main layout
        let git_state = self.git_state.clone();
        let settings = self.settings.clone();
        self.main_layout = Some(cx.new(|cx| MainLayout::new(git_state, settings, cx)));

        self.repository_path = Some(path);
        self.view_mode = ViewMode::Repository;
        cx.notify();
    }

    pub fn close_repository(&mut self, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            state.close_repository(cx);
        });
        self.repository_path = None;
        self.view_mode = ViewMode::Welcome;
        self.main_layout = None;
        cx.notify();
    }

    fn handle_open_repository(
        &mut self,
        _: &OpenRepository,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Open file dialog via App
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Select Repository".into()),
        });

        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = receiver.await {
                if let Some(path) = paths.into_iter().next() {
                    this.update(cx, |app, cx| {
                        app.open_repository(path, cx);
                    })
                    .ok();
                }
            }
        })
        .detach();
    }

    fn handle_close_repository(
        &mut self,
        _: &CloseRepository,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_repository(cx);
    }

    fn handle_open_settings(
        &mut self,
        _: &OpenSettings,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_settings = true;
        cx.notify();
    }

    fn handle_cancel(&mut self, _: &Cancel, _window: &mut Window, cx: &mut Context<Self>) {
        if self.show_settings {
            self.show_settings = false;
            cx.notify();
        }
    }

    fn handle_refresh(&mut self, _: &Refresh, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            state.refresh(cx);
        });
    }

    fn handle_stage_all(&mut self, _: &StageAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.stage_all(cx) {
                log::error!("Failed to stage all: {}", e);
            }
        });
    }

    fn handle_create_commit(
        &mut self,
        _: &CreateCommit,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.notify();
    }

    fn handle_push(&mut self, _: &Push, _window: &mut Window, cx: &mut Context<Self>) {
        let settings = self.settings.read(cx);
        let auth = settings.get_auth_credentials();
        drop(settings);

        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.push(auth.as_ref(), cx) {
                log::error!("Failed to push: {}", e);
            }
        });
    }

    fn handle_pull(&mut self, _: &Pull, _window: &mut Window, cx: &mut Context<Self>) {
        let settings = self.settings.read(cx);
        let auth = settings.get_auth_credentials();
        drop(settings);

        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.pull(auth.as_ref(), cx) {
                log::error!("Failed to pull: {}", e);
            }
        });
    }
}

impl Render for Awabancha {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let recent_projects = self.recent_projects.clone();

        div()
            .id("awabancha-root")
            .key_context("Awabancha")
            .on_action(cx.listener(Self::handle_open_repository))
            .on_action(cx.listener(Self::handle_close_repository))
            .on_action(cx.listener(Self::handle_open_settings))
            .on_action(cx.listener(Self::handle_cancel))
            .on_action(cx.listener(Self::handle_refresh))
            .on_action(cx.listener(Self::handle_stage_all))
            .on_action(cx.listener(Self::handle_create_commit))
            .on_action(cx.listener(Self::handle_push))
            .on_action(cx.listener(Self::handle_pull))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            .when(self.view_mode == ViewMode::Welcome, |this| {
                this.child(
                    WelcomeView::new(recent_projects.clone()).on_open_repository(
                        cx.listener(|this, path: &PathBuf, _window, cx| {
                            this.open_repository(path.clone(), cx);
                        }),
                    ),
                )
            })
            .when_some(self.main_layout.clone(), |this, main_layout| {
                this.child(main_layout)
            })
    }
}
