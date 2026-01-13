use crate::state::GitState;
use crate::views::CommitGraph;
use gpui::prelude::*;
use gpui::*;

pub struct RightPanel {
    git_state: Entity<GitState>,
}

impl RightPanel {
    pub fn new(git_state: Entity<GitState>) -> Self {
        Self { git_state }
    }
}

impl IntoElement for RightPanel {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for RightPanel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            // Search bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x181825))
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .flex_1()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(0x313244))
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child("Search commits..."),
                    ),
            )
            // Commit Graph
            .child(
                div()
                    .id("commit-graph-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .child(CommitGraph::new(self.git_state.clone())),
            )
    }
}
