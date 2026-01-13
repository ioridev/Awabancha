use crate::state::{AuthMode, Language, MergeMode, SettingsState, Theme};
use gpui::prelude::*;
use gpui::*;

pub struct SettingsView {
    settings: Entity<SettingsState>,
}

impl SettingsView {
    pub fn new(settings: Entity<SettingsState>) -> Self {
        Self { settings }
    }
}

impl IntoElement for SettingsView {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for SettingsView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let settings = self.settings.read(cx);

        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x00000088))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_96()
                    .h(px(500.0))
                    .rounded_lg()
                    .bg(rgb(0x1e1e2e))
                    .border_1()
                    .border_color(rgb(0x313244))
                    .overflow_hidden()
                    // Header
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_4()
                            .py_3()
                            .bg(rgb(0x181825))
                            .border_b_1()
                            .border_color(rgb(0x313244))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xcdd6f4))
                                    .child("Settings"),
                            )
                            .child(
                                div()
                                    .id("close-settings")
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .text_sm()
                                    .text_color(rgb(0x9399b2))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x313244)).text_color(rgb(0xcdd6f4)))
                                    .child("×"),
                            ),
                    )
                    // Content
                    .child(
                        div()
                            .id("settings-scroll")
                            .flex_1()
                            .overflow_y_scroll()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_6()
                            // Git Authentication section
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0x89b4fa))
                                            .child("Git Authentication"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x9399b2))
                                                    .child("Auth Mode"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_2()
                                                    .child(SettingsButton::new(
                                                        "HTTPS",
                                                        settings.data.git_auth_mode == AuthMode::Https,
                                                    ))
                                                    .child(SettingsButton::new(
                                                        "SSH",
                                                        settings.data.git_auth_mode == AuthMode::Ssh,
                                                    )),
                                            ),
                                    ),
                            )
                            // About section
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0x89b4fa))
                                            .child("About"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x9399b2))
                                            .child("Awabancha v0.1.0"),
                                    ),
                            ),
                    ),
            )
    }
}

struct SettingsButton {
    label: &'static str,
    selected: bool,
}

impl SettingsButton {
    fn new(label: &'static str, selected: bool) -> Self {
        Self { label, selected }
    }
}

impl IntoElement for SettingsButton {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for SettingsButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .text_sm()
            .cursor_pointer()
            .bg(if self.selected {
                rgb(0x89b4fa)
            } else {
                rgb(0x313244)
            })
            .text_color(if self.selected {
                rgb(0x1e1e2e)
            } else {
                rgb(0xcdd6f4)
            })
            .when(!self.selected, |this| this.hover(|s| s.bg(rgb(0x45475a))))
            .child(self.label)
    }
}
