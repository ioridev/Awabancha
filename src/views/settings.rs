#![allow(dead_code)]

use crate::i18n::{t, Locale};
use crate::state::{AuthMode, MergeMode, SettingsState};
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
        let locale = settings.data.locale;
        let auth_mode = settings.data.git_auth_mode;
        let merge_mode = settings.data.merge_mode;
        let username = settings.data.git_username.clone().unwrap_or_default();
        let has_token = settings.data.git_token.is_some();

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
                    .max_h(px(600.0))
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
                                    .child(t(locale, "settings.title")),
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
                            .overflow_scroll()
                            .p_4()
                            .flex()
                            .flex_col()
                            .gap_6()
                            // Language section
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
                                            .child(t(locale, "settings.general")),
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
                                                    .child(t(locale, "settings.language")),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_1()
                                                    .children(Locale::all().iter().map(|l| {
                                                        LanguageButton::new(*l, locale == *l)
                                                    })),
                                            ),
                                    ),
                            )
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
                                            .child(t(locale, "settings.gitAuth")),
                                    )
                                    // Auth Mode selector
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x9399b2))
                                                    .child(t(locale, "settings.gitAuthMethod")),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_2()
                                                    .child(SettingsButton::new(
                                                        t(locale, "auth.https"),
                                                        auth_mode == AuthMode::Https,
                                                    ))
                                                    .child(SettingsButton::new(
                                                        t(locale, "auth.ssh"),
                                                        auth_mode == AuthMode::Ssh,
                                                    )),
                                            ),
                                    )
                                    // Username display (HTTPS only)
                                    .when(auth_mode == AuthMode::Https, |this| {
                                        this.child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(rgb(0x9399b2))
                                                        .child(t(locale, "settings.gitUsername")),
                                                )
                                                .child(
                                                    div()
                                                        .px_3()
                                                        .py_1()
                                                        .rounded_md()
                                                        .bg(rgb(0x313244))
                                                        .text_sm()
                                                        .text_color(if username.is_empty() {
                                                            rgb(0x6c7086)
                                                        } else {
                                                            rgb(0xcdd6f4)
                                                        })
                                                        .child(if username.is_empty() {
                                                            t(locale, "settings.gitUsernamePlaceholder")
                                                        } else {
                                                            username
                                                        }),
                                                ),
                                        )
                                    })
                                    // Token status (HTTPS only)
                                    .when(auth_mode == AuthMode::Https, |this| {
                                        this.child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(rgb(0x9399b2))
                                                        .child(t(locale, "settings.gitToken")),
                                                )
                                                .child(
                                                    div()
                                                        .px_3()
                                                        .py_1()
                                                        .rounded_md()
                                                        .bg(rgb(0x313244))
                                                        .text_sm()
                                                        .text_color(if has_token {
                                                            rgb(0xa6e3a1)
                                                        } else {
                                                            rgb(0xf38ba8)
                                                        })
                                                        .child(if has_token {
                                                            "••••••••".to_string()
                                                        } else {
                                                            t(locale, "settings.gitTokenPlaceholder")
                                                        }),
                                                ),
                                        )
                                    })
                                    // SSH info
                                    .when(auth_mode == AuthMode::Ssh, |this| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x6c7086))
                                                .child("SSH authentication uses the system SSH agent"),
                                        )
                                    }),
                            )
                            // Merge Options section
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
                                            .child(t(locale, "settings.merge")),
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
                                                    .child(t(locale, "settings.mergeLabel")),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .gap_1()
                                                    .child(MergeButton::new(t(locale, "settings.mergeAuto"), merge_mode == MergeMode::Auto))
                                                    .child(MergeButton::new("FF", merge_mode == MergeMode::FfOnly))
                                                    .child(MergeButton::new("No-FF", merge_mode == MergeMode::NoFf))
                                                    .child(MergeButton::new(t(locale, "settings.mergeSquash"), merge_mode == MergeMode::Squash)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x6c7086))
                                            .child(match merge_mode {
                                                MergeMode::Auto => "Auto: Let git decide the best strategy",
                                                MergeMode::FfOnly => "FF-Only: Only allow fast-forward merges",
                                                MergeMode::NoFf => "No-FF: Always create merge commits",
                                                MergeMode::Squash => "Squash: Combine all commits into one",
                                            }),
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
                                            .child(t(locale, "settings.about")),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(
                                                        div()
                                                            .text_lg()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(rgb(0xcdd6f4))
                                                            .child(t(locale, "app.name")),
                                                    )
                                                    .child(
                                                        div()
                                                            .px_2()
                                                            .py_px()
                                                            .rounded_sm()
                                                            .bg(rgb(0x313244))
                                                            .text_xs()
                                                            .text_color(rgb(0x9399b2))
                                                            .child("v0.1.0"),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x6c7086))
                                                    .child(t(locale, "app.tagline")),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x6c7086))
                                                    .child("Powered by git2-rs and gpui"),
                                            ),
                                    ),
                            )
                            // Keyboard Shortcuts section
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
                                            .child(t(locale, "settings.keyboard")),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .text_xs()
                                            .text_color(rgb(0x9399b2))
                                            .child(KeyboardShortcut::new("Cmd+O", t(locale, "welcome.openRepo")))
                                            .child(KeyboardShortcut::new("Cmd+S", t(locale, "fileList.stageAll")))
                                            .child(KeyboardShortcut::new("Cmd+Enter", t(locale, "commit.button")))
                                            .child(KeyboardShortcut::new("Cmd+Shift+P", t(locale, "left.push")))
                                            .child(KeyboardShortcut::new("Cmd+Shift+L", t(locale, "left.pull")))
                                            .child(KeyboardShortcut::new("Cmd+R", t(locale, "common.refresh")))
                                            .child(KeyboardShortcut::new("Cmd+,", t(locale, "settings.title")))
                                            .child(KeyboardShortcut::new("Escape", t(locale, "common.close"))),
                                    ),
                            ),
                    ),
            )
    }
}

struct SettingsButton {
    label: String,
    selected: bool,
}

impl SettingsButton {
    fn new(label: impl Into<String>, selected: bool) -> Self {
        Self {
            label: label.into(),
            selected,
        }
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

struct MergeButton {
    label: String,
    selected: bool,
}

impl MergeButton {
    fn new(label: impl Into<String>, selected: bool) -> Self {
        Self {
            label: label.into(),
            selected,
        }
    }
}

impl IntoElement for MergeButton {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for MergeButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .text_xs()
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

struct LanguageButton {
    locale: Locale,
    selected: bool,
}

impl LanguageButton {
    fn new(locale: Locale, selected: bool) -> Self {
        Self { locale, selected }
    }
}

impl IntoElement for LanguageButton {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for LanguageButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // Use short labels for the buttons
        let label = match self.locale {
            Locale::En => "EN",
            Locale::Ja => "日本語",
            Locale::ZhHans => "简体",
            Locale::ZhHant => "繁體",
        };

        div()
            .px_2()
            .py_1()
            .rounded_md()
            .text_xs()
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
            .child(label)
    }
}

struct KeyboardShortcut {
    shortcut: &'static str,
    description: String,
}

impl KeyboardShortcut {
    fn new(shortcut: &'static str, description: impl Into<String>) -> Self {
        Self {
            shortcut,
            description: description.into(),
        }
    }
}

impl IntoElement for KeyboardShortcut {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for KeyboardShortcut {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .py_1()
            .child(
                div()
                    .text_color(rgb(0x9399b2))
                    .child(self.description),
            )
            .child(
                div()
                    .px_2()
                    .py_px()
                    .rounded_sm()
                    .bg(rgb(0x313244))
                    .text_color(rgb(0xcdd6f4))
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.shortcut),
            )
    }
}
