use crate::components::TextInputView;
use crate::state::GitState;
use gpui::prelude::*;
use gpui::*;

pub struct CommitForm {
    git_state: Entity<GitState>,
    commit_message: Entity<TextInputView>,
    amend: bool,
    /// Saved message when switching between amend/non-amend modes
    saved_message: String,
}

impl CommitForm {
    pub fn new(git_state: Entity<GitState>, cx: &mut Context<Self>) -> Self {
        let commit_message = cx.new(|cx| {
            TextInputView::new(cx)
                .with_placeholder("Enter commit message...")
                .multiline(true)
        });

        // Observe git state changes
        cx.observe(&git_state, |_this, _git_state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            git_state,
            commit_message,
            amend: false,
            saved_message: String::new(),
        }
    }

    fn toggle_amend(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let current_message = self.commit_message.read(cx).content().to_string();

        if !self.amend {
            // Switching to amend mode
            // Save current message and load previous commit message
            self.saved_message = current_message;
            if let Some(last_message) = self.git_state.read(cx).get_last_commit_message() {
                let trimmed = last_message.trim().to_string();
                self.commit_message.update(cx, |input, cx| {
                    input.set_content(trimmed, cx);
                });
            }
        } else {
            // Switching back to normal mode
            // Restore saved message
            let saved = self.saved_message.clone();
            self.commit_message.update(cx, |input, cx| {
                input.set_content(saved, cx);
            });
        }

        self.amend = !self.amend;
        cx.notify();
    }

    fn do_commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let message = self.commit_message.read(cx).content().to_string();
        if message.trim().is_empty() {
            return;
        }

        let amend = self.amend;
        self.git_state.update(cx, |state, cx| {
            let result = if amend {
                state.amend_commit(&message, cx)
            } else {
                state.create_commit(&message, cx)
            };

            if let Err(e) = result {
                log::error!("Failed to commit: {}", e);
            }
        });

        // Clear the commit message after successful commit
        self.commit_message.update(cx, |input, cx| {
            input.set_content("", cx);
        });
        self.amend = false;
        cx.notify();

        // Focus back to the input
        let focus_handle = self.commit_message.read(cx).focus_handle(cx);
        window.focus(&focus_handle, cx);
    }
}

impl Render for CommitForm {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let git_state = self.git_state.read(cx);
        let staged_count = git_state.staged_files().len();
        let can_commit = staged_count > 0;
        let amend = self.amend;

        div()
            .flex()
            .flex_col()
            .gap_3()
            // Commit message section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x9399b2))
                            .child("Commit message"),
                    )
                    .child(self.commit_message.clone()),
            )
            // Options row
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .id("amend-checkbox")
                            .flex()
                            .items_center()
                            .gap_1()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.toggle_amend(window, cx);
                            }))
                            .child(
                                div()
                                    .size_4()
                                    .rounded_sm()
                                    .border_1()
                                    .border_color(if amend {
                                        rgb(0x89b4fa)
                                    } else {
                                        rgb(0x6c7086)
                                    })
                                    .bg(if amend {
                                        rgb(0x89b4fa)
                                    } else {
                                        rgb(0x313244)
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when(amend, |this| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x1e1e2e))
                                                .child("✓"),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9399b2))
                                    .child("Amend"),
                            ),
                    ),
            )
            // Commit button
            .child(
                div()
                    .id("commit-button")
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .py_2()
                    .rounded_md()
                    .bg(if can_commit {
                        rgb(0xa6e3a1)
                    } else {
                        rgb(0x45475a)
                    })
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if can_commit {
                        rgb(0x1e1e2e)
                    } else {
                        rgb(0x6c7086)
                    })
                    .when(can_commit, |this| {
                        this.cursor_pointer()
                            .hover(|s| s.bg(rgb(0x94e2d5)))
                            .active(|s| s.bg(rgb(0x89b4fa)))
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.do_commit(window, cx);
                            }))
                    })
                    .child(format!(
                        "Commit ({} file{})",
                        staged_count,
                        if staged_count == 1 { "" } else { "s" }
                    )),
            )
    }
}
