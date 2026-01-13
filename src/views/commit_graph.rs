#![allow(dead_code)]

use crate::state::GitState;
use gpui::prelude::*;
use gpui::*;

const NODE_RADIUS: f32 = 4.0;
const COLUMN_WIDTH: f32 = 16.0;
const ROW_HEIGHT: f32 = 32.0;
const GRAPH_PADDING: f32 = 8.0;

pub struct CommitGraph {
    git_state: Entity<GitState>,
}

impl CommitGraph {
    pub fn new(git_state: Entity<GitState>) -> Self {
        Self { git_state }
    }
}

impl IntoElement for CommitGraph {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for CommitGraph {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let git_state = self.git_state.clone();
        let git_state_read = git_state.read(cx);

        let commits = git_state_read.commits.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .when(commits.is_some(), |this| {
                let commits = commits.unwrap();
                this.children(commits.nodes.iter().enumerate().map(|(idx, node)| {
                    CommitRow::new(node.clone(), idx, commits.max_column)
                }))
            })
            .when(git_state_read.commits.is_none(), |this| {
                this.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py_8()
                        .text_sm()
                        .text_color(rgb(0x6c7086))
                        .child("No commits"),
                )
            })
    }
}

pub struct CommitRow {
    node: crate::git::GraphNode,
    row_index: usize,
    max_column: usize,
}

impl CommitRow {
    pub fn new(node: crate::git::GraphNode, row_index: usize, max_column: usize) -> Self {
        Self {
            node,
            row_index,
            max_column,
        }
    }
}

impl IntoElement for CommitRow {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for CommitRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let commit = &self.node.commit;
        let graph_width =
            ((self.max_column + 1) as f32 * COLUMN_WIDTH + GRAPH_PADDING * 2.0) as i32;

        div()
            .id(ElementId::Name(format!("commit-{}", commit.sha).into()))
            .flex()
            .items_center()
            .h(px(ROW_HEIGHT))
            .px_2()
            .cursor_pointer()
            .hover(|s| s.bg(rgb(0x313244)))
            // Graph column
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(graph_width as f32))
                    .h_full()
                    .child(GraphNode::new(self.node.column, self.node.color)),
            )
            // Commit info
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_hidden()
                    .gap_px()
                    // Message and refs
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .overflow_hidden()
                            // Branch labels
                            .children(commit.branches.iter().take(2).map(|branch| {
                                div()
                                    .px_1()
                                    .rounded_sm()
                                    .bg(rgb(0x89b4fa))
                                    .text_xs()
                                    .text_color(rgb(0x1e1e2e))
                                    .child(branch.clone())
                            }))
                            // Tag labels
                            .children(commit.tags.iter().take(1).map(|tag| {
                                div()
                                    .px_1()
                                    .rounded_sm()
                                    .bg(rgb(0xf9e2af))
                                    .text_xs()
                                    .text_color(rgb(0x1e1e2e))
                                    .child(tag.clone())
                            }))
                            // Commit message
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xcdd6f4))
                                    .text_ellipsis()
                                    .child(commit.message.clone()),
                            ),
                    )
                    // Author and time
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(commit.author.clone())
                            .child("·")
                            .child(commit.relative_time()),
                    ),
            )
            // SHA
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child(commit.short_sha.clone()),
            )
    }
}

pub struct GraphNode {
    column: usize,
    color: u32,
}

impl GraphNode {
    pub fn new(column: usize, color: u32) -> Self {
        Self { column, color }
    }
}

impl IntoElement for GraphNode {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for GraphNode {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let x = GRAPH_PADDING + (self.column as f32 * COLUMN_WIDTH) + COLUMN_WIDTH / 2.0;

        div()
            .absolute()
            .left(px(x - NODE_RADIUS))
            .size(px(NODE_RADIUS * 2.0))
            .rounded_full()
            .bg(rgb(self.color))
    }
}
