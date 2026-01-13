#![allow(dead_code)]

use crate::git::{DiffLineType, FileDiff};
use gpui::prelude::*;
use gpui::*;

#[derive(IntoElement)]
pub struct DiffViewer {
    diff: FileDiff,
}

impl DiffViewer {
    pub fn new(diff: FileDiff) -> Self {
        Self { diff }
    }
}

impl RenderOnce for DiffViewer {
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
                    .w(px(800.0))
                    .h(px(600.0))
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
                                    .flex()
                                    .items_center()
                                    .gap_4()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xcdd6f4))
                                            .child(self.diff.path.clone()),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .text_xs()
                                            .child(
                                                div()
                                                    .text_color(rgb(0xa6e3a1))
                                                    .child(format!("+{}", self.diff.additions)),
                                            )
                                            .child(
                                                div()
                                                    .text_color(rgb(0xf38ba8))
                                                    .child(format!("-{}", self.diff.deletions)),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .id("close-diff")
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
                    // Diff content
                    .child(
                        div()
                            .id("diff-scroll")
                            .flex_1()
                            .overflow_y_scroll()
                            .p_2()
                            .children(
                                self.diff.lines.iter().map(|line| DiffLine::new(line.clone())),
                            ),
                    ),
            )
    }
}

#[derive(IntoElement)]
pub struct DiffLine {
    line: crate::git::DiffLine,
}

impl DiffLine {
    pub fn new(line: crate::git::DiffLine) -> Self {
        Self { line }
    }
}

impl RenderOnce for DiffLine {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (bg_color, text_color, prefix) = match self.line.line_type {
            DiffLineType::Addition => (rgb(0x1a3d2e), rgb(0xa6e3a1), "+"),
            DiffLineType::Deletion => (rgb(0x3d1a1a), rgb(0xf38ba8), "-"),
            DiffLineType::Context => (rgb(0x1e1e2e), rgb(0xcdd6f4), " "),
            DiffLineType::Header => (rgb(0x313244), rgb(0x89b4fa), ""),
        };

        div()
            .flex()
            .items_start()
            .text_sm()
            .bg(bg_color)
            // Line numbers
            .child(
                div()
                    .flex()
                    .items_center()
                    .w_20()
                    .px_2()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child(
                        self.line
                            .old_lineno
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| " ".to_string()),
                    )
                    .child(
                        self.line
                            .new_lineno
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| " ".to_string()),
                    ),
            )
            // Prefix
            .child(div().w_4().text_color(text_color).child(prefix.to_string()))
            // Content
            .child(
                div()
                    .flex_1()
                    .text_color(text_color)
                    .child(self.line.content.trim_end().to_string()),
            )
    }
}
