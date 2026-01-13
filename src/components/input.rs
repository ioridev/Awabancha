use gpui::prelude::*;
use gpui::*;

pub struct TextInput {
    value: String,
    placeholder: Option<SharedString>,
    password: bool,
    multiline: bool,
}

impl TextInput {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            placeholder: None,
            password: false,
            multiline: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }
}

impl IntoElement for TextInput {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for TextInput {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let display_value = if self.password && !self.value.is_empty() {
            "•".repeat(self.value.len())
        } else if self.value.is_empty() {
            self.placeholder
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or_default()
        } else {
            self.value.clone()
        };

        let is_placeholder = self.value.is_empty() && self.placeholder.is_some();

        let base = div()
            .w_full()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(rgb(0x313244))
            .border_1()
            .border_color(rgb(0x45475a))
            .text_sm()
            .text_color(if is_placeholder {
                rgb(0x6c7086)
            } else {
                rgb(0xcdd6f4)
            })
            .hover(|s| s.border_color(rgb(0x6c7086)))
            .focus(|s| s.border_color(rgb(0x89b4fa)));

        if self.multiline {
            base.h_20().child(display_value)
        } else {
            base.child(display_value)
        }
    }
}
