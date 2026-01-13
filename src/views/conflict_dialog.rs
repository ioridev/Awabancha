use crate::git::{ConflictInfo, ConflictStrategy, ConflictedFile};
use crate::state::GitState;
use gpui::prelude::*;
use gpui::*;

pub struct ConflictDialog {
    git_state: Entity<GitState>,
    conflict_info: Option<ConflictInfo>,
    mode: ConflictResolutionMode,
    per_file_selections: Vec<(String, Option<ConflictStrategy>)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolutionMode {
    Bulk,
    PerFile,
}

impl ConflictDialog {
    pub fn new(git_state: Entity<GitState>, cx: &mut Context<Self>) -> Self {
        let git_state_read = git_state.read(cx);
        let conflict_info = git_state_read.conflict_info.clone();

        let per_file_selections = conflict_info
            .as_ref()
            .map(|info| {
                info.conflicted_files
                    .iter()
                    .map(|f| (f.path.clone(), None))
                    .collect()
            })
            .unwrap_or_default();

        // Observe git state changes
        cx.observe(&git_state, |this, git_state, cx| {
            let git_state_read = git_state.read(cx);
            this.conflict_info = git_state_read.conflict_info.clone();
            this.per_file_selections = this
                .conflict_info
                .as_ref()
                .map(|info| {
                    info.conflicted_files
                        .iter()
                        .map(|f| (f.path.clone(), None))
                        .collect()
                })
                .unwrap_or_default();
            cx.notify();
        })
        .detach();

        Self {
            git_state,
            conflict_info,
            mode: ConflictResolutionMode::Bulk,
            per_file_selections,
        }
    }

    fn set_mode(&mut self, mode: ConflictResolutionMode, cx: &mut Context<Self>) {
        self.mode = mode;
        cx.notify();
    }

    fn resolve_all(&mut self, strategy: ConflictStrategy, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.resolve_all_conflicts(strategy, cx) {
                log::error!("Failed to resolve all conflicts: {}", e);
            }
        });
    }

    fn set_file_strategy(
        &mut self,
        path: String,
        strategy: ConflictStrategy,
        cx: &mut Context<Self>,
    ) {
        if let Some(selection) = self
            .per_file_selections
            .iter_mut()
            .find(|(p, _)| *p == path)
        {
            selection.1 = Some(strategy);
        }
        cx.notify();
    }

    fn resolve_per_file(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let resolutions: Vec<_> = self
            .per_file_selections
            .iter()
            .filter_map(|(path, strategy)| strategy.map(|s| (path.clone(), s)))
            .collect();

        if resolutions.is_empty() {
            log::warn!("No resolutions selected");
            return;
        }

        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.resolve_conflicts_per_file(resolutions, cx) {
                log::error!("Failed to resolve conflicts: {}", e);
            }
        });
    }

    fn complete_merge(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.complete_merge(None, cx) {
                log::error!("Failed to complete merge: {}", e);
            }
        });
    }

    fn abort_merge(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.abort_merge(cx) {
                log::error!("Failed to abort merge: {}", e);
            }
        });
    }
}

impl Render for ConflictDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(conflict_info) = &self.conflict_info else {
            return div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .child(
                    div()
                        .text_color(rgb(0x9399b2))
                        .child("No merge conflicts"),
                );
        };

        let source = conflict_info
            .source_branch
            .clone()
            .unwrap_or_else(|| "source".to_string());
        let target = conflict_info
            .target_branch
            .clone()
            .unwrap_or_else(|| "target".to_string());
        let file_count = conflict_info.conflicted_files.len();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .p_4()
            .gap_4()
            // Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xf38ba8))
                                    .child("Merge Conflicts"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x9399b2))
                                    .child(format!(
                                        "Merging {} into {} - {} file{} conflicted",
                                        source,
                                        target,
                                        file_count,
                                        if file_count == 1 { "" } else { "s" }
                                    )),
                            ),
                    )
                    // Abort button
                    .child(
                        div()
                            .id("abort-merge-btn")
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x313244))
                            .text_sm()
                            .text_color(rgb(0xf38ba8))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .child("Abort Merge")
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.abort_merge(window, cx);
                            })),
                    ),
            )
            // Mode selector
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .id("bulk-mode-btn")
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(if self.mode == ConflictResolutionMode::Bulk {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_sm()
                            .text_color(if self.mode == ConflictResolutionMode::Bulk {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .cursor_pointer()
                            .hover(|s| {
                                if self.mode != ConflictResolutionMode::Bulk {
                                    s.bg(rgb(0x45475a))
                                } else {
                                    s
                                }
                            })
                            .child("Bulk Resolve")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.set_mode(ConflictResolutionMode::Bulk, cx);
                            })),
                    )
                    .child(
                        div()
                            .id("per-file-mode-btn")
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(if self.mode == ConflictResolutionMode::PerFile {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_sm()
                            .text_color(if self.mode == ConflictResolutionMode::PerFile {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .cursor_pointer()
                            .hover(|s| {
                                if self.mode != ConflictResolutionMode::PerFile {
                                    s.bg(rgb(0x45475a))
                                } else {
                                    s
                                }
                            })
                            .child("Per File")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.set_mode(ConflictResolutionMode::PerFile, cx);
                            })),
                    ),
            )
            // Conflict list
            .child(
                div()
                    .id("conflict-list-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .rounded_md()
                    .bg(rgb(0x181825))
                    .p_2()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .children(conflict_info.conflicted_files.iter().map(|file| {
                                self.render_conflict_file(file.clone(), cx)
                            })),
                    ),
            )
            // Actions
            .child(self.render_actions(cx))
    }
}

impl ConflictDialog {
    fn render_conflict_file(
        &self,
        file: ConflictedFile,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let path = file.path.clone();
        let selected_strategy = self
            .per_file_selections
            .iter()
            .find(|(p, _)| *p == path)
            .and_then(|(_, s)| *s);

        let filename = path.rsplit('/').next().unwrap_or(&path).to_string();
        let dir_path = if path.contains('/') {
            path.rsplit_once('/').map(|(dir, _)| dir.to_string())
        } else {
            None
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .rounded_md()
            .hover(|s| s.bg(rgb(0x313244)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xf38ba8))
                                    .child("C"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xcdd6f4))
                                    .text_ellipsis()
                                    .child(filename),
                            ),
                    )
                    .when_some(dir_path, |this, dir| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x6c7086))
                                .pl_5()
                                .text_ellipsis()
                                .child(dir),
                        )
                    })
                    .when(file.is_deleted_by_us, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xf9e2af))
                                .pl_5()
                                .child("Deleted by us"),
                        )
                    })
                    .when(file.is_deleted_by_them, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xf9e2af))
                                .pl_5()
                                .child("Deleted by them"),
                        )
                    }),
            )
            .when(self.mode == ConflictResolutionMode::PerFile, |this| {
                let path_ours = path.clone();
                let path_theirs = path.clone();
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .id(ElementId::Name(format!("ours-{}", path_ours).into()))
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .bg(if selected_strategy == Some(ConflictStrategy::Ours) {
                                    rgb(0xa6e3a1)
                                } else {
                                    rgb(0x313244)
                                })
                                .text_xs()
                                .text_color(if selected_strategy == Some(ConflictStrategy::Ours) {
                                    rgb(0x1e1e2e)
                                } else {
                                    rgb(0xcdd6f4)
                                })
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x45475a)))
                                .child("Ours")
                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                    this.set_file_strategy(
                                        path_ours.clone(),
                                        ConflictStrategy::Ours,
                                        cx,
                                    );
                                })),
                        )
                        .child(
                            div()
                                .id(ElementId::Name(format!("theirs-{}", path_theirs).into()))
                                .px_2()
                                .py_1()
                                .rounded_sm()
                                .bg(if selected_strategy == Some(ConflictStrategy::Theirs) {
                                    rgb(0x89b4fa)
                                } else {
                                    rgb(0x313244)
                                })
                                .text_xs()
                                .text_color(
                                    if selected_strategy == Some(ConflictStrategy::Theirs) {
                                        rgb(0x1e1e2e)
                                    } else {
                                        rgb(0xcdd6f4)
                                    },
                                )
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x45475a)))
                                .child("Theirs")
                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                    this.set_file_strategy(
                                        path_theirs.clone(),
                                        ConflictStrategy::Theirs,
                                        cx,
                                    );
                                })),
                        ),
                )
            })
    }

    fn render_actions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let git_state_read = self.git_state.read(cx);
        let has_remaining_conflicts = git_state_read
            .conflict_info
            .as_ref()
            .map(|info| !info.conflicted_files.is_empty())
            .unwrap_or(false);

        let all_selected = self.mode == ConflictResolutionMode::PerFile
            && self
                .per_file_selections
                .iter()
                .all(|(_, s)| s.is_some());

        div()
            .flex()
            .items_center()
            .justify_between()
            .pt_2()
            .border_t_1()
            .border_color(rgb(0x313244))
            .when(self.mode == ConflictResolutionMode::Bulk, |this| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .id("resolve-ours-btn")
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(rgb(0xa6e3a1))
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0x1e1e2e))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x94e2d5)))
                                .child("Accept Ours (All)")
                                .on_click(cx.listener(|this, _event, window, cx| {
                                    this.resolve_all(ConflictStrategy::Ours, window, cx);
                                })),
                        )
                        .child(
                            div()
                                .id("resolve-theirs-btn")
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(rgb(0x89b4fa))
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0x1e1e2e))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0xb4befe)))
                                .child("Accept Theirs (All)")
                                .on_click(cx.listener(|this, _event, window, cx| {
                                    this.resolve_all(ConflictStrategy::Theirs, window, cx);
                                })),
                        ),
                )
            })
            .when(self.mode == ConflictResolutionMode::PerFile, |this| {
                this.child(
                    div()
                        .id("apply-selections-btn")
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(if all_selected {
                            rgb(0xa6e3a1)
                        } else {
                            rgb(0x45475a)
                        })
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if all_selected {
                            rgb(0x1e1e2e)
                        } else {
                            rgb(0x6c7086)
                        })
                        .when(all_selected, |this| {
                            this.cursor_pointer()
                                .hover(|s| s.bg(rgb(0x94e2d5)))
                                .on_click(cx.listener(|this, _event, window, cx| {
                                    this.resolve_per_file(window, cx);
                                }))
                        })
                        .child("Apply Selections"),
                )
            })
            // Complete merge button (shown when no conflicts remain)
            .when(!has_remaining_conflicts, |this| {
                this.child(
                    div()
                        .id("complete-merge-btn")
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(rgb(0xa6e3a1))
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x1e1e2e))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x94e2d5)))
                        .child("Complete Merge")
                        .on_click(cx.listener(|this, _event, window, cx| {
                            this.complete_merge(window, cx);
                        })),
                )
            })
    }
}
