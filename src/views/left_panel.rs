use crate::state::{GitState, SettingsState};
use crate::views::{CommitForm, FileList};
use gpui::prelude::*;
use gpui::*;

pub struct LeftPanel {
    git_state: Entity<GitState>,
    settings: Option<Entity<SettingsState>>,
    commit_form: Entity<CommitForm>,
    file_list: Entity<FileList>,
    /// Whether stash section is expanded
    stash_expanded: bool,
}

impl LeftPanel {
    pub fn new(git_state: Entity<GitState>, cx: &mut Context<Self>) -> Self {
        let commit_form = cx.new(|cx| CommitForm::new(git_state.clone(), cx));
        let file_list = cx.new(|cx| FileList::new(git_state.clone(), cx));

        // Observe git state changes
        cx.observe(&git_state, |_this, _git_state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            git_state,
            settings: None,
            commit_form,
            file_list,
            stash_expanded: false,
        }
    }

    pub fn with_settings(mut self, settings: Entity<SettingsState>) -> Self {
        self.settings = Some(settings);
        self
    }

    fn toggle_stash_expanded(&mut self, cx: &mut Context<Self>) {
        self.stash_expanded = !self.stash_expanded;
        cx.notify();
    }

    fn handle_push(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let auth = self.settings.as_ref().and_then(|s| {
            let settings = s.read(cx);
            settings.get_auth_credentials()
        });

        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.push(auth.as_ref(), cx) {
                log::error!("Failed to push: {}", e);
            }
        });
    }

    fn handle_pull(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let auth = self.settings.as_ref().and_then(|s| {
            let settings = s.read(cx);
            settings.get_auth_credentials()
        });

        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.pull(auth.as_ref(), cx) {
                log::error!("Failed to pull: {}", e);
            }
        });
    }

    fn handle_fetch(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let auth = self.settings.as_ref().and_then(|s| {
            let settings = s.read(cx);
            settings.get_auth_credentials()
        });

        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.fetch(auth.as_ref(), cx) {
                log::error!("Failed to fetch: {}", e);
            }
        });
    }

    fn handle_stage_all(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.stage_all(cx) {
                log::error!("Failed to stage all: {}", e);
            }
        });
    }

    fn handle_unstage_all(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.unstage_all(cx) {
                log::error!("Failed to unstage all: {}", e);
            }
        });
    }

    fn handle_stash_save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.stash_save(None, cx) {
                log::error!("Failed to save stash: {}", e);
            }
        });
    }

    fn handle_stash_pop(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.stash_pop(index, cx) {
                log::error!("Failed to pop stash: {}", e);
            }
        });
    }

    fn handle_stash_apply(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.stash_apply(index, cx) {
                log::error!("Failed to apply stash: {}", e);
            }
        });
    }

    fn handle_stash_drop(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.stash_drop(index, cx) {
                log::error!("Failed to drop stash: {}", e);
            }
        });
    }
}

impl Render for LeftPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let git_state_read = self.git_state.read(cx);
        let staged_count = git_state_read.staged_files().len();
        let unstaged_count = git_state_read.unstaged_files().len();
        let stashes = git_state_read.stashes.clone();
        let stash_expanded = self.stash_expanded;

        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            // Commit Form
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .child(self.commit_form.clone()),
            )
            // File List Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x181825))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("Changes"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            // Stage All button
                            .when(unstaged_count > 0, |this| {
                                this.child(
                                    div()
                                        .id("stage-all-btn")
                                        .px_2()
                                        .py_px()
                                        .rounded_sm()
                                        .text_xs()
                                        .text_color(rgb(0xa6e3a1))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgb(0x313244)))
                                        .child("+All")
                                        .on_click(cx.listener(|this, _event, window, cx| {
                                            this.handle_stage_all(window, cx);
                                        })),
                                )
                            })
                            // Unstage All button
                            .when(staged_count > 0, |this| {
                                this.child(
                                    div()
                                        .id("unstage-all-btn")
                                        .px_2()
                                        .py_px()
                                        .rounded_sm()
                                        .text_xs()
                                        .text_color(rgb(0xfab387))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgb(0x313244)))
                                        .child("-All")
                                        .on_click(cx.listener(|this, _event, window, cx| {
                                            this.handle_unstage_all(window, cx);
                                        })),
                                )
                            })
                            .when(staged_count > 0, |this| {
                                this.child(
                                    div()
                                        .px_2()
                                        .py_px()
                                        .rounded_sm()
                                        .bg(rgb(0xa6e3a1))
                                        .text_xs()
                                        .text_color(rgb(0x1e1e2e))
                                        .child(format!("{} staged", staged_count)),
                                )
                            })
                            .when(unstaged_count > 0, |this| {
                                this.child(
                                    div()
                                        .px_2()
                                        .py_px()
                                        .rounded_sm()
                                        .bg(rgb(0xfab387))
                                        .text_xs()
                                        .text_color(rgb(0x1e1e2e))
                                        .child(format!("{} unstaged", unstaged_count)),
                                )
                            }),
                    ),
            )
            // File List
            .child(
                div()
                    .id("file-list-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .child(self.file_list.clone()),
            )
            // Stash Section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .border_t_1()
                    .border_color(rgb(0x313244))
                    // Stash Header
                    .child(
                        div()
                            .id("stash-header")
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x181825))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x1e1e2e)))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.toggle_stash_expanded(cx);
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x9399b2))
                                            .child(if stash_expanded { "▼" } else { "▶" }),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xcdd6f4))
                                            .child("Stashes"),
                                    )
                                    .when(!stashes.is_empty(), |this| {
                                        this.child(
                                            div()
                                                .px_2()
                                                .py_px()
                                                .rounded_sm()
                                                .bg(rgb(0xcba6f7))
                                                .text_xs()
                                                .text_color(rgb(0x1e1e2e))
                                                .child(format!("{}", stashes.len())),
                                        )
                                    }),
                            )
                            // Stash Save button
                            .child(
                                div()
                                    .id("stash-save-btn")
                                    .px_2()
                                    .py_1()
                                    .rounded_sm()
                                    .text_xs()
                                    .text_color(rgb(0xcba6f7))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x313244)))
                                    .child("+ Stash")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.handle_stash_save(window, cx);
                                    })),
                            ),
                    )
                    // Stash List (when expanded)
                    .when(stash_expanded && !stashes.is_empty(), |this| {
                        this.child(
                            div()
                                .id("stash-list-scroll")
                                .flex()
                                .flex_col()
                                .max_h(px(150.0))
                                .overflow_scroll()
                                .children(stashes.iter().enumerate().map(|(idx, stash)| {
                                    let stash_idx = stash.index;
                                    let stash_idx_pop = stash_idx;
                                    let stash_idx_apply = stash_idx;
                                    let stash_idx_drop = stash_idx;
                                    div()
                                        .id(ElementId::Name(format!("stash-{}", idx).into()))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px_4()
                                        .py_2()
                                        .border_t_1()
                                        .border_color(rgb(0x313244))
                                        .hover(|s| s.bg(rgb(0x313244)))
                                        // Stash info
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .flex_1()
                                                .overflow_hidden()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x9399b2))
                                                        .child(format!("stash@{{{}}}", stash.index)),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(rgb(0xcdd6f4))
                                                        .text_ellipsis()
                                                        .child(stash.message.clone()),
                                                ),
                                        )
                                        // Action buttons
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                // Pop
                                                .child(
                                                    div()
                                                        .id(ElementId::Name(format!("stash-pop-{}", idx).into()))
                                                        .px_2()
                                                        .py_1()
                                                        .rounded_sm()
                                                        .text_xs()
                                                        .text_color(rgb(0xa6e3a1))
                                                        .cursor_pointer()
                                                        .hover(|s| s.bg(rgb(0x45475a)))
                                                        .child("Pop")
                                                        .on_click(cx.listener(move |this, _event, window, cx| {
                                                            this.handle_stash_pop(stash_idx_pop, window, cx);
                                                        })),
                                                )
                                                // Apply
                                                .child(
                                                    div()
                                                        .id(ElementId::Name(format!("stash-apply-{}", idx).into()))
                                                        .px_2()
                                                        .py_1()
                                                        .rounded_sm()
                                                        .text_xs()
                                                        .text_color(rgb(0x89b4fa))
                                                        .cursor_pointer()
                                                        .hover(|s| s.bg(rgb(0x45475a)))
                                                        .child("Apply")
                                                        .on_click(cx.listener(move |this, _event, window, cx| {
                                                            this.handle_stash_apply(stash_idx_apply, window, cx);
                                                        })),
                                                )
                                                // Drop
                                                .child(
                                                    div()
                                                        .id(ElementId::Name(format!("stash-drop-{}", idx).into()))
                                                        .px_2()
                                                        .py_1()
                                                        .rounded_sm()
                                                        .text_xs()
                                                        .text_color(rgb(0xf38ba8))
                                                        .cursor_pointer()
                                                        .hover(|s| s.bg(rgb(0x45475a)))
                                                        .child("Drop")
                                                        .on_click(cx.listener(move |this, _event, window, cx| {
                                                            this.handle_stash_drop(stash_idx_drop, window, cx);
                                                        })),
                                                ),
                                        )
                                })),
                        )
                    }),
            )
            // Remote Operations
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .p_4()
                    .border_t_1()
                    .border_color(rgb(0x313244))
                    // Push button
                    .child(
                        div()
                            .id("push-button")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x313244))
                            .text_sm()
                            .text_color(rgb(0xcdd6f4))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .child("Push")
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.handle_push(window, cx);
                            })),
                    )
                    // Pull button
                    .child(
                        div()
                            .id("pull-button")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x313244))
                            .text_sm()
                            .text_color(rgb(0xcdd6f4))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .child("Pull")
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.handle_pull(window, cx);
                            })),
                    )
                    // Fetch button
                    .child(
                        div()
                            .id("fetch-button")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x313244))
                            .text_sm()
                            .text_color(rgb(0xcdd6f4))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .child("Fetch")
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.handle_fetch(window, cx);
                            })),
                    ),
            )
    }
}
