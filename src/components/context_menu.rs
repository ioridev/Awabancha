use gpui::prelude::*;
use gpui::*;

pub struct ContextMenuItem {
    pub label: String,
    pub danger: bool,
    pub disabled: bool,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            danger: false,
            disabled: false,
        }
    }

    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl IntoElement for ContextMenuItem {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for ContextMenuItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let base = div()
            .px_3()
            .py_1()
            .text_sm()
            .text_color(if self.danger {
                rgb(0xf38ba8)
            } else if self.disabled {
                rgb(0x6c7086)
            } else {
                rgb(0xcdd6f4)
            });

        if self.disabled {
            base.child(self.label)
        } else {
            base.cursor_pointer()
                .hover(|s| s.bg(rgb(0x45475a)))
                .child(self.label)
        }
    }
}

pub struct ContextMenu {
    items: Vec<ContextMenuItem>,
    position: Point<Pixels>,
}

impl ContextMenu {
    pub fn new(items: Vec<ContextMenuItem>, position: Point<Pixels>) -> Self {
        Self { items, position }
    }
}

impl IntoElement for ContextMenu {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for ContextMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .absolute()
            .inset_0()
            .child(
                div()
                    .absolute()
                    .left(self.position.x)
                    .top(self.position.y)
                    .min_w_40()
                    .py_1()
                    .rounded_md()
                    .bg(rgb(0x313244))
                    .border_1()
                    .border_color(rgb(0x45475a))
                    .shadow_lg()
                    .children(self.items),
            )
    }
}
