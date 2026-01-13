#![allow(dead_code)]

use crate::git::FileStatus;
use crate::state::GitState;
use gpui::prelude::*;
use gpui::*;

pub struct FileList {
    git_state: Entity<GitState>,
}

impl FileList {
    pub fn new(git_state: Entity<GitState>, cx: &mut Context<Self>) -> Self {
        // Observe git state changes
        cx.observe(&git_state, |_this, _git_state, cx| {
            cx.notify();
        })
        .detach();

        Self { git_state }
    }

    fn stage_file(&mut self, path: String, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.stage_file(&path, cx) {
                log::error!("Failed to stage file: {}", e);
            }
        });
    }

    fn unstage_file(&mut self, path: String, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.unstage_file(&path, cx) {
                log::error!("Failed to unstage file: {}", e);
            }
        });
    }

    fn discard_file(&mut self, path: String, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.discard_file(&path, cx) {
                log::error!("Failed to discard file: {}", e);
            }
        });
    }
}

impl Render for FileList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let git_state_read = self.git_state.read(cx);

        let staged_files: Vec<_> = git_state_read
            .staged_files()
            .iter()
            .map(|f| (*f).clone())
            .collect();
        let unstaged_files: Vec<_> = git_state_read
            .unstaged_files()
            .iter()
            .map(|f| (*f).clone())
            .collect();
        let is_empty = git_state_read.files.is_empty();

        div()
            .flex()
            .flex_col()
            // Staged section
            .when(!staged_files.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .px_4()
                                .py_1()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xa6e3a1))
                                .bg(rgb(0x181825))
                                .child("Staged"),
                        )
                        .children(staged_files.into_iter().map(|file| {
                            let path = file.path.clone();
                            self.render_file_item(file, true, cx)
                                .on_click(cx.listener(move |this, _event, window, cx| {
                                    this.unstage_file(path.clone(), window, cx);
                                }))
                        })),
                )
            })
            // Unstaged section
            .when(!unstaged_files.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .px_4()
                                .py_1()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xfab387))
                                .bg(rgb(0x181825))
                                .child("Unstaged"),
                        )
                        .children(unstaged_files.into_iter().map(|file| {
                            let path = file.path.clone();
                            self.render_file_item(file, false, cx)
                                .on_click(cx.listener(move |this, _event, window, cx| {
                                    this.stage_file(path.clone(), window, cx);
                                }))
                        })),
                )
            })
            // Empty state
            .when(is_empty, |this| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py_8()
                        .text_sm()
                        .text_color(rgb(0x6c7086))
                        .child("No changes"),
                )
            })
    }
}

impl FileList {
    fn render_file_item(
        &self,
        file: FileStatus,
        is_staged: bool,
        _cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let status_char = file.status_char();
        let status_color = file.status_color();

        // Get just the filename for display
        let filename = file
            .path
            .rsplit('/')
            .next()
            .unwrap_or(&file.path)
            .to_string();

        // Get directory path
        let dir_path = if file.path.contains('/') {
            file.path.rsplit_once('/').map(|(dir, _)| dir.to_string())
        } else {
            None
        };

        let base = div()
            .id(ElementId::Name(format!("file-{}", file.path).into()))
            .flex()
            .items_center()
            .gap_2()
            .px_4()
            .py_1()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(0x313244)))
            // Status indicator
            .child(
                div()
                    .w_5()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(status_color))
                    .child(status_char.to_string()),
            )
            // File info
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcdd6f4))
                            .text_ellipsis()
                            .child(filename),
                    )
                    .when_some(dir_path, |this, dir| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x6c7086))
                                .text_ellipsis()
                                .child(dir),
                        )
                    }),
            )
            // Stage/Unstage indicator
            .child(
                div()
                    .px_2()
                    .py_px()
                    .rounded_sm()
                    .text_xs()
                    .text_color(rgb(0x9399b2))
                    .hover(|s| s.bg(rgb(0x45475a)).text_color(rgb(0xcdd6f4)))
                    .child(if is_staged { "−" } else { "+" }),
            );

        base
    }
}

// Keep FileListItem for backward compatibility if needed elsewhere
pub struct FileListItem {
    file: FileStatus,
    is_staged: bool,
}

impl FileListItem {
    pub fn new(file: FileStatus, is_staged: bool) -> Self {
        Self { file, is_staged }
    }
}

impl IntoElement for FileListItem {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for FileListItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let status_char = self.file.status_char();
        let status_color = self.file.status_color();

        // Get just the filename for display
        let filename = self
            .file
            .path
            .rsplit('/')
            .next()
            .unwrap_or(&self.file.path);

        // Get directory path
        let dir_path = if self.file.path.contains('/') {
            self.file.path.rsplit_once('/').map(|(dir, _)| dir.to_string())
        } else {
            None
        };

        let base = div()
            .id(ElementId::Name(format!("file-{}", self.file.path).into()))
            .flex()
            .items_center()
            .gap_2()
            .px_4()
            .py_1()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(0x313244)))
            // Status indicator
            .child(
                div()
                    .w_5()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(status_color))
                    .child(status_char.to_string()),
            )
            // File info
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcdd6f4))
                            .text_ellipsis()
                            .child(filename.to_string()),
                    ),
            )
            // Stage/Unstage button
            .child(
                div()
                    .px_2()
                    .py_px()
                    .rounded_sm()
                    .text_xs()
                    .text_color(rgb(0x9399b2))
                    .hover(|s| s.bg(rgb(0x45475a)).text_color(rgb(0xcdd6f4)))
                    .child(if self.is_staged { "−" } else { "+" }),
            );

        if let Some(dir) = dir_path {
            base.child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .text_ellipsis()
                    .child(dir),
            )
        } else {
            base
        }
    }
}
