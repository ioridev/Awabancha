#![allow(dead_code)]

use gpui::prelude::*;
use gpui::*;

#[derive(IntoElement)]
pub struct Modal {
    title: SharedString,
    content: AnyElement,
}

impl Modal {
    pub fn new(title: impl Into<SharedString>, content: impl IntoElement) -> Self {
        Self {
            title: title.into(),
            content: content.into_any_element(),
        }
    }
}

impl RenderOnce for Modal {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
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
                    .min_w_80()
                    .w(px(500.0))
                    .h(px(400.0))
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
                                    .child(self.title.to_string()),
                            )
                            .child(
                                div()
                                    .id("modal-close")
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
                            .id("modal-content")
                            .flex_1()
                            .overflow_y_scroll()
                            .p_4()
                            .child(self.content),
                    ),
            )
    }
}
