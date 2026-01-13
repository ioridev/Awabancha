use crate::state::GitState;
use gpui::prelude::*;
use gpui::*;

pub struct CommitForm {
    git_state: Entity<GitState>,
}

impl CommitForm {
    pub fn new(git_state: Entity<GitState>) -> Self {
        Self { git_state }
    }
}

impl IntoElement for CommitForm {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for CommitForm {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let git_state = self.git_state.clone();
        let git_state_read = git_state.read(cx);

        let staged_count = git_state_read.staged_files().len();
        let can_commit = staged_count > 0;

        div()
            .flex()
            .flex_col()
            .gap_3()
            // Commit message textarea
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
                    .child(
                        div()
                            .w_full()
                            .h_20()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x313244))
                            .text_sm()
                            .text_color(rgb(0xcdd6f4))
                            .border_1()
                            .border_color(rgb(0x45475a))
                            .child("Type your commit message here..."),
                    ),
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
                            .child(
                                div()
                                    .size_4()
                                    .rounded_sm()
                                    .border_1()
                                    .border_color(rgb(0x6c7086)),
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
                    })
                    .child(format!(
                        "Commit ({} file{})",
                        staged_count,
                        if staged_count == 1 { "" } else { "s" }
                    )),
            )
    }
}
