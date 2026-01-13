use gpui::prelude::*;
use gpui::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

pub struct Toast {
    message: String,
    toast_type: ToastType,
}

impl Toast {
    pub fn new(message: impl Into<String>, toast_type: ToastType) -> Self {
        Self {
            message: message.into(),
            toast_type,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Success)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Error)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Warning)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Info)
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
        let (bg, border, icon) = match self.toast_type {
            ToastType::Success => (rgb(0x1a3d2e), rgb(0xa6e3a1), "✓"),
            ToastType::Error => (rgb(0x3d1a1a), rgb(0xf38ba8), "✕"),
            ToastType::Warning => (rgb(0x3d3d1a), rgb(0xf9e2af), "⚠"),
            ToastType::Info => (rgb(0x1a2a3d), rgb(0x89b4fa), "ℹ"),
        };

        div()
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
                    .child(self.message),
            )
            // Dismiss button
            .child(
                div()
                    .id("toast-dismiss")
                    .px_1()
                    .text_sm()
                    .text_color(rgb(0x6c7086))
                    .cursor_pointer()
                    .hover(|s| s.text_color(rgb(0xcdd6f4)))
                    .child("×"),
            )
    }
}

pub struct ToastContainer {
    toasts: Vec<Toast>,
}

impl ToastContainer {
    pub fn new(toasts: Vec<Toast>) -> Self {
        Self { toasts }
    }
}

impl IntoElement for ToastContainer {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for ToastContainer {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .absolute()
            .bottom_4()
            .right_4()
            .flex()
            .flex_col()
            .gap_2()
            .w_80()
            .children(self.toasts)
    }
}
