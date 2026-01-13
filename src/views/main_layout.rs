use crate::state::{GitState, SettingsState};
use crate::views::{LeftPanel, RightPanel};
use gpui::prelude::*;
use gpui::*;

pub struct MainLayout {
    git_state: Entity<GitState>,
    settings: Entity<SettingsState>,
}

impl MainLayout {
    pub fn new(git_state: Entity<GitState>, settings: Entity<SettingsState>) -> Self {
        Self { git_state, settings }
    }
}

impl IntoElement for MainLayout {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for MainLayout {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let git_state = self.git_state.clone();
        let git_state_read = git_state.read(cx);

        let current_branch = git_state_read.current_branch().map(|s| s.to_string());
        let is_detached = git_state_read.is_detached();
        let ahead = git_state_read
            .repository_info
            .as_ref()
            .map(|r| r.ahead)
            .unwrap_or(0);
        let behind = git_state_read
            .repository_info
            .as_ref()
            .map(|r| r.behind)
            .unwrap_or(0);

        div()
            .flex()
            .flex_col()
            .size_full()
            // Header bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .h_12()
                    .bg(rgb(0x181825))
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    // Left: Branch info
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if is_detached {
                                        rgb(0xf9e2af)
                                    } else {
                                        rgb(0x89b4fa)
                                    })
                                    .child(if is_detached {
                                        "HEAD detached".to_string()
                                    } else {
                                        current_branch.unwrap_or_else(|| "No branch".to_string())
                                    }),
                            )
                            .when(ahead > 0 || behind > 0, |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x9399b2))
                                        .child(format!("↑{} ↓{}", ahead, behind)),
                                )
                            }),
                    )
                    // Right: Settings button
                    .child(
                        div()
                            .id("settings-button")
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .text_sm()
                            .text_color(rgb(0x9399b2))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x313244)).text_color(rgb(0xcdd6f4)))
                            .child("Settings"),
                    ),
            )
            // Main content area (left + right panels)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .overflow_hidden()
                    // Left panel (file changes, commit form)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_80()
                            .min_w_64()
                            .bg(rgb(0x1e1e2e))
                            .border_r_1()
                            .border_color(rgb(0x313244))
                            .child(LeftPanel::new(self.git_state.clone())),
                    )
                    // Right panel (commit graph)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .bg(rgb(0x1e1e2e))
                            .child(RightPanel::new(self.git_state.clone())),
                    ),
            )
    }
}
