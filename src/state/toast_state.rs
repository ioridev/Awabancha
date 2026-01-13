use gpui::*;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone)]
pub struct ToastMessage {
    pub id: usize,
    pub message: String,
    pub toast_type: ToastType,
}

/// Global toast notification state
pub struct ToastState {
    toasts: Vec<ToastMessage>,
    next_id: usize,
}

impl ToastState {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 0,
        }
    }

    pub fn toasts(&self) -> &[ToastMessage] {
        &self.toasts
    }

    pub fn show(&mut self, message: impl Into<String>, toast_type: ToastType, cx: &mut Context<Self>) {
        let id = self.next_id;
        self.next_id += 1;

        self.toasts.push(ToastMessage {
            id,
            message: message.into(),
            toast_type,
        });

        // Auto-dismiss after 3 seconds
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_secs(3))
                .await;
            let _ = this.update(cx, |state, cx| {
                state.dismiss(id, cx);
            });
        })
        .detach();

        cx.notify();
    }

    pub fn success(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        self.show(message, ToastType::Success, cx);
    }

    pub fn error(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        self.show(message, ToastType::Error, cx);
    }

    pub fn warning(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        self.show(message, ToastType::Warning, cx);
    }

    pub fn info(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        self.show(message, ToastType::Info, cx);
    }

    pub fn dismiss(&mut self, id: usize, cx: &mut Context<Self>) {
        self.toasts.retain(|t| t.id != id);
        cx.notify();
    }
}
