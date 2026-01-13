use crate::state::RecentProjects;
use gpui::prelude::*;
use gpui::*;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(IntoElement)]
pub struct WelcomeView {
    recent_projects: Entity<RecentProjects>,
    on_open_repository: Option<Arc<dyn Fn(&PathBuf, &mut Window, &mut App) + Send + Sync + 'static>>,
    on_open_dialog: Option<Arc<dyn Fn(&(), &mut Window, &mut App) + Send + Sync + 'static>>,
}

impl WelcomeView {
    pub fn new(recent_projects: Entity<RecentProjects>) -> Self {
        Self {
            recent_projects,
            on_open_repository: None,
            on_open_dialog: None,
        }
    }

    pub fn on_open_repository(
        mut self,
        handler: impl Fn(&PathBuf, &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_open_repository = Some(Arc::new(handler));
        self
    }

    pub fn on_open_dialog(
        mut self,
        handler: impl Fn(&(), &mut Window, &mut App) + Send + Sync + 'static,
    ) -> Self {
        self.on_open_dialog = Some(Arc::new(handler));
        self
    }

    /// Check if a path is a valid git repository
    fn is_git_repository(path: &PathBuf) -> bool {
        // Check for .git directory
        let git_dir = path.join(".git");
        if git_dir.exists() {
            return true;
        }

        // Check for bare repository (has HEAD and config files)
        let head = path.join("HEAD");
        let config = path.join("config");
        if head.exists() && config.exists() {
            // Additional check: HEAD file should contain valid git reference
            if let Ok(content) = std::fs::read_to_string(&head) {
                return content.starts_with("ref:") || content.len() == 41; // SHA length + newline
            }
        }

        false
    }
}

impl RenderOnce for WelcomeView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let recent = self.recent_projects.read(cx);
        let projects: Vec<_> = recent.projects().to_vec();
        let on_open = self.on_open_repository.clone();
        let on_open_for_drop = on_open.clone();
        let on_open_dialog = self.on_open_dialog.clone();

        div()
            .id("welcome-drop-target")
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap_8()
            // Drop handler for external file drops
            .on_drop(move |paths: &ExternalPaths, window, cx| {
                // Find the first valid git repository in dropped paths
                for path in paths.paths() {
                    // If it's a file, use its parent directory
                    let check_path = if path.is_file() {
                        path.parent().map(|p| p.to_path_buf())
                    } else {
                        Some(path.clone())
                    };

                    if let Some(dir_path) = check_path {
                        if Self::is_git_repository(&dir_path) {
                            if let Some(ref handler) = on_open_for_drop {
                                handler(&dir_path, window, cx);
                            }
                            return;
                        }
                    }
                }
                // No valid git repository found - could show an error toast here
                log::warn!("Dropped path is not a valid git repository");
            })
            // Logo / Title
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_3xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("Awabancha"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x9399b2))
                            .child("A fast Git GUI client"),
                    ),
            )
            // Drop hint
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child("Drop a git repository folder here"),
            )
            // Open Repository Button
            .child(
                div()
                    .id("open-repo-button")
                    .px_6()
                    .py_3()
                    .rounded_lg()
                    .bg(rgb(0x89b4fa))
                    .text_color(rgb(0x1e1e2e))
                    .font_weight(FontWeight::SEMIBOLD)
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0xb4befe)))
                    .active(|s| s.bg(rgb(0x7287fd)))
                    .child("Open Repository")
                    .on_click(move |_event, window, cx| {
                        if let Some(ref handler) = on_open_dialog {
                            handler(&(), window, cx);
                        }
                    }),
            )
            // Recent Projects
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w_80()
                    .when(!projects.is_empty(), |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x9399b2))
                                .mb_2()
                                .child("Recent Projects"),
                        )
                    })
                    .children(projects.into_iter().map(|project| {
                        let path = project.path.clone();
                        let on_open_clone = on_open.clone();
                        div()
                            .id(ElementId::Name(
                                format!("recent-{}", project.path.display()).into(),
                            ))
                            .flex()
                            .items_center()
                            .gap_3()
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .bg(rgb(0x313244))
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .on_click(move |_event, window, cx| {
                                if let Some(ref handler) = on_open_clone {
                                    handler(&path, window, cx);
                                }
                            })
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xcdd6f4))
                                            .text_ellipsis()
                                            .child(project.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x6c7086))
                                            .text_ellipsis()
                                            .child(project.path.display().to_string()),
                                    ),
                            )
                    })),
            )
    }
}
