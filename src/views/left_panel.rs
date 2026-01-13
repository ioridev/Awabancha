use crate::state::GitState;
use crate::views::{CommitForm, FileList};
use gpui::prelude::*;
use gpui::*;

pub struct LeftPanel {
    git_state: Entity<GitState>,
}

impl LeftPanel {
    pub fn new(git_state: Entity<GitState>) -> Self {
        Self { git_state }
    }
}

impl IntoElement for LeftPanel {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for LeftPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let git_state = self.git_state.clone();
        let git_state_read = git_state.read(cx);

        let staged_count = git_state_read.staged_files().len();
        let unstaged_count = git_state_read.unstaged_files().len();

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
                    .child(CommitForm::new(self.git_state.clone())),
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
                    .child(FileList::new(self.git_state.clone())),
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
                            .child("Push"),
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
                            .child("Pull"),
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
                            .child("Fetch"),
                    ),
            )
    }
}
