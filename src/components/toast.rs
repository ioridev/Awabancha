#![allow(dead_code)]

use crate::state::{ToastMessage, ToastState, ToastType};
use gpui::prelude::*;
use gpui::*;

/// A single toast notification component
pub struct Toast {
    message: ToastMessage,
    toast_state: Entity<ToastState>,
}

impl Toast {
    pub fn new(message: ToastMessage, toast_state: Entity<ToastState>) -> Self {
        Self {
            message,
            toast_state,
        }
    }
}

impl IntoElement for Toast {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for Toast {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (bg, border, icon) = match self.message.toast_type {
            ToastType::Success => (rgb(0x1a3d2e), rgb(0xa6e3a1), "✓"),
            ToastType::Error => (rgb(0x3d1a1a), rgb(0xf38ba8), "✕"),
            ToastType::Warning => (rgb(0x3d3d1a), rgb(0xf9e2af), "⚠"),
            ToastType::Info => (rgb(0x1a2a3d), rgb(0x89b4fa), "ℹ"),
        };

        let id = self.message.id;
        let toast_state = self.toast_state.clone();

        div()
            .id(ElementId::Name(format!("toast-{}", id).into()))
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py_3()
            .rounded_lg()
            .bg(bg)
            .border_l_4()
            .border_color(border)
            .shadow_lg()
            // Icon
            .child(div().text_sm().text_color(border).child(icon))
            // Message
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(rgb(0xcdd6f4))
                    .child(self.message.message.clone()),
            )
            // Dismiss button
            .child(
                div()
                    .id(ElementId::Name(format!("toast-dismiss-{}", id).into()))
                    .px_1()
                    .text_sm()
                    .text_color(rgb(0x6c7086))
                    .cursor_pointer()
                    .hover(|s| s.text_color(rgb(0xcdd6f4)))
                    .child("×")
                    .on_click(move |_event, _window, cx| {
                        toast_state.update(cx, |state, cx| {
                            state.dismiss(id, cx);
                        });
                    }),
            )
    }
}

/// Container for all toast notifications
pub struct ToastContainer {
    toast_state: Entity<ToastState>,
}

impl ToastContainer {
    pub fn new(toast_state: Entity<ToastState>) -> Self {
        Self { toast_state }
    }
}

impl IntoElement for ToastContainer {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for ToastContainer {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let toasts: Vec<_> = self
            .toast_state
            .read(cx)
            .toasts()
            .iter()
            .cloned()
            .collect();

        div()
            .absolute()
            .bottom_4()
            .right_4()
            .flex()
            .flex_col()
            .gap_2()
            .w_80()
            .children(
                toasts
                    .into_iter()
                    .map(|msg| Toast::new(msg, self.toast_state.clone())),
            )
    }
}
