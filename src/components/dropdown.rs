use gpui::prelude::*;
use gpui::*;

pub struct DropdownOption {
    pub value: String,
    pub label: String,
}

pub struct Dropdown {
    options: Vec<DropdownOption>,
    selected: Option<String>,
    placeholder: SharedString,
}

impl Dropdown {
    pub fn new(options: Vec<DropdownOption>) -> Self {
        Self {
            options,
            selected: None,
            placeholder: "Select...".into(),
        }
    }

    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
}

impl IntoElement for Dropdown {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for Dropdown {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let selected_label = self
            .selected
            .as_ref()
            .and_then(|v| self.options.iter().find(|o| &o.value == v))
            .map(|o| o.label.clone())
            .unwrap_or_else(|| self.placeholder.to_string());

        div()
            .relative()
            .w_full()
            // Trigger
            .child(
                div()
                    .id("dropdown-trigger")
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(rgb(0x313244))
                    .border_1()
                    .border_color(rgb(0x45475a))
                    .cursor_pointer()
                    .hover(|s| s.border_color(rgb(0x6c7086)))
                    .child(
                        div()
                            .text_sm()
                            .text_color(if self.selected.is_some() {
                                rgb(0xcdd6f4)
                            } else {
                                rgb(0x6c7086)
                            })
                            .child(selected_label),
                    )
                    .child(div().text_xs().text_color(rgb(0x6c7086)).child("▼")),
            )
    }
}
