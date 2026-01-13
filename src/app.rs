use crate::actions::*;
use crate::components::ToastContainer;
use crate::state::{GitState, RecentProjects, RepositoryWatcher, SettingsState, ToastState};
use crate::views::{ConflictDialog, DiffViewer, MainLayout, SettingsView, WelcomeView};
use gpui::prelude::*;
use gpui::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
    /// Toast notifications
    pub toast_state: Entity<ToastState>,
    /// Current view mode
    pub view_mode: ViewMode,
    /// Show settings modal
    pub show_settings: bool,
    /// Show diff viewer modal
    pub show_diff: bool,
    /// Show conflict dialog modal
    pub show_conflict_dialog: bool,
    /// Conflict dialog entity
    conflict_dialog: Option<Entity<ConflictDialog>>,
    /// Main layout entity (created when repository is opened)
    main_layout: Option<Entity<MainLayout>>,
    /// File system watcher for auto-refresh
    watcher: Arc<Mutex<RepositoryWatcher>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Welcome,
    Repository,
}

impl Awabancha {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let git_state = cx.new(|_| GitState::new());
        let settings = cx.new(|cx| SettingsState::load(cx));
        let recent_projects = cx.new(|cx| RecentProjects::load(cx));
        let toast_state = cx.new(|_| ToastState::new());

        // Set up window activation observer for auto-refresh
        let git_state_for_activation = git_state.clone();
        cx.observe_window_activation(window, move |app, _window, cx| {
            // Only refresh when window becomes active and repository is open
            if app.view_mode == ViewMode::Repository {
                git_state_for_activation.update(cx, |state, cx| {
                    state.refresh(cx);
                });
            }
        })
        .detach();

        // Observe toast state for re-renders
        cx.observe(&toast_state, |_this, _toast_state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            repository_path: None,
            git_state,
            settings,
            recent_projects,
            toast_state,
            view_mode: ViewMode::Welcome,
            show_settings: false,
            show_diff: false,
            show_conflict_dialog: false,
            conflict_dialog: None,
            main_layout: None,
            watcher: Arc::new(Mutex::new(RepositoryWatcher::new())),
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

        // Start file watcher
        self.start_watching(path.clone(), cx);

        self.repository_path = Some(path);
        self.view_mode = ViewMode::Repository;
        cx.notify();
    }

    fn start_watching(&self, path: PathBuf, cx: &mut Context<Self>) {
        // Start the watcher
        if let Ok(mut watcher) = self.watcher.lock() {
            if let Err(e) = watcher.watch(path) {
                log::warn!("Failed to start file watcher: {}", e);
            }
        }

        // Spawn a background task to poll for changes
        let watcher = self.watcher.clone();
        let git_state = self.git_state.clone();

        cx.spawn(async move |this, cx| {
            loop {
                // Sleep for a bit
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(500))
                    .await;

                // Check if watcher detected changes
                let should_refresh = watcher
                    .lock()
                    .map(|w| w.poll())
                    .unwrap_or(false);

                if should_refresh {
                    let _ = this.update(cx, |_app, cx| {
                        git_state.update(cx, |state, cx| {
                            state.refresh(cx);
                        });
                    });
                }

                // Check if we should stop (repository closed)
                let should_stop = this
                    .update(cx, |app, _cx| {
                        app.view_mode == ViewMode::Welcome
                    })
                    .unwrap_or(true);

                if should_stop {
                    break;
                }
            }
        })
        .detach();
    }

    pub fn close_repository(&mut self, cx: &mut Context<Self>) {
        // Stop the watcher
        if let Ok(mut watcher) = self.watcher.lock() {
            watcher.stop();
        }

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
        if self.show_conflict_dialog {
            self.show_conflict_dialog = false;
            cx.notify();
        } else if self.show_diff {
            self.show_diff = false;
            self.git_state.update(cx, |state, cx| {
                state.clear_diff(cx);
            });
            cx.notify();
        } else if self.show_settings {
            self.show_settings = false;
            cx.notify();
        }
    }

    fn handle_show_diff(&mut self, _: &ShowDiff, _window: &mut Window, cx: &mut Context<Self>) {
        self.show_diff = true;
        cx.notify();
    }

    fn handle_close_diff(&mut self, _: &CloseDiff, _window: &mut Window, cx: &mut Context<Self>) {
        self.show_diff = false;
        self.git_state.update(cx, |state, cx| {
            state.clear_diff(cx);
        });
        cx.notify();
    }

    fn handle_show_conflict_dialog(
        &mut self,
        _: &ShowConflictDialog,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Create conflict dialog entity if needed
        if self.conflict_dialog.is_none() {
            let git_state = self.git_state.clone();
            self.conflict_dialog = Some(cx.new(|cx| ConflictDialog::new(git_state, cx)));
        }
        self.show_conflict_dialog = true;
        cx.notify();
    }

    fn handle_close_conflict_dialog(
        &mut self,
        _: &CloseConflictDialog,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_conflict_dialog = false;
        cx.notify();
    }

    fn handle_refresh(&mut self, _: &Refresh, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            state.refresh(cx);
        });
    }

    fn handle_stage_all(&mut self, _: &StageAll, _window: &mut Window, cx: &mut Context<Self>) {
        let result = self.git_state.update(cx, |state, cx| state.stage_all(cx));
        match result {
            Ok(_) => {
                self.toast_state.update(cx, |toast, cx| {
                    toast.success("All files staged", cx);
                });
            }
            Err(e) => {
                self.toast_state.update(cx, |toast, cx| {
                    toast.error(format!("Failed to stage: {}", e), cx);
                });
            }
        }
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
        let _ = settings;

        let result = self.git_state.update(cx, |state, cx| state.push(auth.as_ref(), cx));
        match result {
            Ok(_) => {
                self.toast_state.update(cx, |toast, cx| {
                    toast.success("Pushed to remote", cx);
                });
            }
            Err(e) => {
                self.toast_state.update(cx, |toast, cx| {
                    toast.error(format!("Push failed: {}", e), cx);
                });
            }
        }
    }

    fn handle_pull(&mut self, _: &Pull, _window: &mut Window, cx: &mut Context<Self>) {
        let settings = self.settings.read(cx);
        let auth = settings.get_auth_credentials();
        let _ = settings;

        let result = self.git_state.update(cx, |state, cx| state.pull(auth.as_ref(), cx));
        match result {
            Ok(_) => {
                self.toast_state.update(cx, |toast, cx| {
                    toast.success("Pulled from remote", cx);
                });
            }
            Err(e) => {
                self.toast_state.update(cx, |toast, cx| {
                    toast.error(format!("Pull failed: {}", e), cx);
                });
            }
        }
    }
}

impl Render for Awabancha {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let recent_projects = self.recent_projects.clone();
        let settings = self.settings.clone();
        let show_settings = self.show_settings;
        let show_diff = self.show_diff;
        let show_conflict_dialog = self.show_conflict_dialog;
        let conflict_dialog = self.conflict_dialog.clone();
        let current_diff = self.git_state.read(cx).current_diff.clone();
        let has_conflicts = self.git_state.read(cx).conflict_info.is_some();

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
            .on_action(cx.listener(Self::handle_show_diff))
            .on_action(cx.listener(Self::handle_close_diff))
            .on_action(cx.listener(Self::handle_show_conflict_dialog))
            .on_action(cx.listener(Self::handle_close_conflict_dialog))
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            .relative()
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
            // Conflict indicator and button when conflicts exist
            .when(has_conflicts && !show_conflict_dialog, |this| {
                this.child(
                    div()
                        .absolute()
                        .bottom_4()
                        .right_4()
                        .child(
                            div()
                                .id("conflict-indicator")
                                .px_4()
                                .py_2()
                                .rounded_lg()
                                .bg(rgb(0xf38ba8))
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0x1e1e2e))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0xeba0ac)))
                                .child("⚠ Merge Conflicts - Click to Resolve")
                                .on_click(|_event, window, cx| {
                                    window.dispatch_action(Box::new(ShowConflictDialog), cx);
                                }),
                        ),
                )
            })
            // Conflict dialog modal overlay
            .when(show_conflict_dialog && conflict_dialog.is_some(), |this| {
                let dialog = conflict_dialog.unwrap();
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba(0x00000088))
                        .child(
                            div()
                                .w(px(700.0))
                                .h(px(500.0))
                                .rounded_lg()
                                .overflow_hidden()
                                .border_1()
                                .border_color(rgb(0x313244))
                                .child(dialog),
                        ),
                )
            })
            // Diff viewer modal overlay
            .when(show_diff && current_diff.is_some(), |this| {
                let diff = current_diff.unwrap();
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .child(
                            div()
                                .id("diff-backdrop")
                                .absolute()
                                .inset_0()
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    this.show_diff = false;
                                    this.git_state.update(cx, |state, cx| {
                                        state.clear_diff(cx);
                                    });
                                    cx.notify();
                                })),
                        )
                        .child(DiffViewer::new(diff)),
                )
            })
            // Settings modal overlay
            .when(show_settings, |this| {
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .child(
                            div()
                                .id("settings-backdrop")
                                .absolute()
                                .inset_0()
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    this.show_settings = false;
                                    cx.notify();
                                })),
                        )
                        .child(SettingsView::new(settings)),
                )
            })
            // Toast notifications (always on top)
            .child(ToastContainer::new(self.toast_state.clone()))
    }
}
