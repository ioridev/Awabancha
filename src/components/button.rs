use gpui::prelude::*;
use gpui::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

pub struct Button {
    label: SharedString,
    variant: ButtonVariant,
    disabled: bool,
    loading: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Button {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Primary,
            disabled: false,
            loading: false,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (bg, hover_bg, text_color) = match self.variant {
            ButtonVariant::Primary => (rgb(0x89b4fa), rgb(0xb4befe), rgb(0x1e1e2e)),
            ButtonVariant::Secondary => (rgb(0x313244), rgb(0x45475a), rgb(0xcdd6f4)),
            ButtonVariant::Danger => (rgb(0xf38ba8), rgb(0xeba0ac), rgb(0x1e1e2e)),
            ButtonVariant::Ghost => (rgba(0x00000000), rgb(0x313244), rgb(0xcdd6f4)),
        };

        let disabled = self.disabled || self.loading;
        let on_click = self.on_click;

        let base = div()
            .id(ElementId::Name(format!("button-{}", self.label).into()))
            .flex()
            .items_center()
            .justify_center()
            .px_4()
            .py_2()
            .rounded_md()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .bg(if disabled { rgb(0x45475a) } else { bg })
            .text_color(if disabled { rgb(0x6c7086) } else { text_color })
            .child(if self.loading {
                "Loading...".to_string()
            } else {
                self.label.to_string()
            });

        if disabled {
            base
        } else {
            let base = base
                .cursor_pointer()
                .hover(|s| s.bg(hover_bg))
                .active(|s| s.opacity(0.9));

            if let Some(handler) = on_click {
                base.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                })
            } else {
                base
            }
        }
    }
}
