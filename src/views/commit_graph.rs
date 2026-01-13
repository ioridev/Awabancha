#![allow(dead_code)]

use crate::git::ResetMode;
use crate::state::GitState;
use gpui::prelude::*;
use gpui::*;

const NODE_RADIUS: f32 = 4.0;
const COLUMN_WIDTH: f32 = 16.0;
const ROW_HEIGHT: f32 = 32.0;
const GRAPH_PADDING: f32 = 8.0;

pub struct CommitGraph {
    git_state: Entity<GitState>,
    /// Context menu state
    context_menu: Option<ContextMenuState>,
}

#[derive(Clone)]
struct ContextMenuState {
    sha: String,
    position: Point<Pixels>,
    is_merge_commit: bool,
}

impl CommitGraph {
    pub fn new(git_state: Entity<GitState>, cx: &mut Context<Self>) -> Self {
        // Observe git state changes
        cx.observe(&git_state, |_this, _git_state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            git_state,
            context_menu: None,
        }
    }

    fn show_context_menu(
        &mut self,
        sha: String,
        position: Point<Pixels>,
        is_merge_commit: bool,
        cx: &mut Context<Self>,
    ) {
        self.context_menu = Some(ContextMenuState {
            sha,
            position,
            is_merge_commit,
        });
        cx.notify();
    }

    fn hide_context_menu(&mut self, cx: &mut Context<Self>) {
        self.context_menu = None;
        cx.notify();
    }

    fn checkout_commit(&mut self, sha: &str, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.checkout_commit(sha, cx) {
                log::error!("Failed to checkout commit: {}", e);
            }
        });
        self.hide_context_menu(cx);
    }

    fn create_branch_from(&mut self, sha: &str, _window: &mut Window, cx: &mut Context<Self>) {
        // For now, create a branch with a default name
        // TODO: Show dialog to input branch name
        let branch_name = format!("branch-{}", &sha[..7]);
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.checkout_commit(sha, cx) {
                log::error!("Failed to checkout: {}", e);
                return;
            }
            if let Err(e) = state.create_branch(&branch_name, cx) {
                log::error!("Failed to create branch: {}", e);
            }
        });
        self.hide_context_menu(cx);
    }

    fn create_tag_at(&mut self, sha: &str, _window: &mut Window, cx: &mut Context<Self>) {
        // For now, create a tag with a default name
        // TODO: Show dialog to input tag name and message
        let tag_name = format!("tag-{}", &sha[..7]);
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.create_tag(&tag_name, sha, None, cx) {
                log::error!("Failed to create tag: {}", e);
            }
        });
        self.hide_context_menu(cx);
    }

    fn cherry_pick(&mut self, sha: &str, _window: &mut Window, cx: &mut Context<Self>) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.cherry_pick(sha, cx) {
                log::error!("Failed to cherry-pick: {}", e);
            }
        });
        self.hide_context_menu(cx);
    }

    fn revert_commit(
        &mut self,
        sha: &str,
        mainline: Option<u32>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.revert_commit(sha, mainline, cx) {
                log::error!("Failed to revert: {}", e);
            }
        });
        self.hide_context_menu(cx);
    }

    fn reset_to_commit(
        &mut self,
        sha: &str,
        mode: ResetMode,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.git_state.update(cx, |state, cx| {
            if let Err(e) = state.reset_to_commit(sha, mode, cx) {
                log::error!("Failed to reset: {}", e);
            }
        });
        self.hide_context_menu(cx);
    }
}

impl Render for CommitGraph {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let git_state_read = self.git_state.read(cx);
        let commits = git_state_read.commits.clone();
        let context_menu = self.context_menu.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .relative()
            // Click outside to close context menu
            .when(context_menu.is_some(), |this| {
                this.on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                        this.hide_context_menu(cx);
                    }),
                )
            })
            .when(commits.is_some(), |this| {
                let commits = commits.unwrap();
                this.children(commits.nodes.iter().enumerate().map(|(idx, node)| {
                    let sha = node.commit.sha.clone();
                    let is_merge = node.commit.parents.len() > 1;
                    div()
                        .child(CommitRow::new(node.clone(), idx, commits.max_column))
                        .on_mouse_down(
                            MouseButton::Right,
                            cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                                this.show_context_menu(
                                    sha.clone(),
                                    event.position,
                                    is_merge,
                                    cx,
                                );
                            }),
                        )
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
            // Context menu
            .when_some(context_menu.clone(), |this, menu| {
                this.child(self.render_context_menu(menu, cx))
            })
    }
}

impl CommitGraph {
    fn render_context_menu(
        &self,
        menu: ContextMenuState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let sha = menu.sha.clone();
        let sha_checkout = sha.clone();
        let sha_branch = sha.clone();
        let sha_tag = sha.clone();
        let sha_cherry = sha.clone();
        let sha_revert = sha.clone();
        let sha_reset_soft = sha.clone();
        let sha_reset_mixed = sha.clone();
        let sha_reset_hard = sha.clone();
        let is_merge = menu.is_merge_commit;

        div()
            .absolute()
            .left(menu.position.x)
            .top(menu.position.y)
            .w(px(200.0))
            .rounded_lg()
            .bg(rgb(0x181825))
            .border_1()
            .border_color(rgb(0x313244))
            .shadow_lg()
            .py_1()
            .flex()
            .flex_col()
            // Checkout
            .child(
                div()
                    .id("ctx-checkout")
                    .px_3()
                    .py_2()
                    .text_sm()
                    .text_color(rgb(0xcdd6f4))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x313244)))
                    .child("Checkout")
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.checkout_commit(&sha_checkout, window, cx);
                    })),
            )
            // Create branch
            .child(
                div()
                    .id("ctx-branch")
                    .px_3()
                    .py_2()
                    .text_sm()
                    .text_color(rgb(0xcdd6f4))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x313244)))
                    .child("Create Branch")
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.create_branch_from(&sha_branch, window, cx);
                    })),
            )
            // Create tag
            .child(
                div()
                    .id("ctx-tag")
                    .px_3()
                    .py_2()
                    .text_sm()
                    .text_color(rgb(0xcdd6f4))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x313244)))
                    .child("Create Tag")
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.create_tag_at(&sha_tag, window, cx);
                    })),
            )
            // Separator
            .child(div().h_px().bg(rgb(0x313244)).my_1())
            // Cherry-pick
            .child(
                div()
                    .id("ctx-cherry-pick")
                    .px_3()
                    .py_2()
                    .text_sm()
                    .text_color(rgb(0xcdd6f4))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x313244)))
                    .child("Cherry-pick")
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.cherry_pick(&sha_cherry, window, cx);
                    })),
            )
            // Revert
            .child(
                div()
                    .id("ctx-revert")
                    .px_3()
                    .py_2()
                    .text_sm()
                    .text_color(rgb(0xcdd6f4))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x313244)))
                    .child(if is_merge {
                        "Revert (mainline 1)"
                    } else {
                        "Revert"
                    })
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        let mainline = if is_merge { Some(1) } else { None };
                        this.revert_commit(&sha_revert, mainline, window, cx);
                    })),
            )
            // Separator
            .child(div().h_px().bg(rgb(0x313244)).my_1())
            // Reset submenu
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .px_3()
                    .py_1()
                    .child("Reset to this commit:"),
            )
            .child(
                div()
                    .id("ctx-reset-soft")
                    .px_3()
                    .py_2()
                    .text_sm()
                    .text_color(rgb(0xa6e3a1))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x313244)))
                    .child("Soft (keep changes staged)")
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.reset_to_commit(&sha_reset_soft, ResetMode::Soft, window, cx);
                    })),
            )
            .child(
                div()
                    .id("ctx-reset-mixed")
                    .px_3()
                    .py_2()
                    .text_sm()
                    .text_color(rgb(0xf9e2af))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x313244)))
                    .child("Mixed (keep changes unstaged)")
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.reset_to_commit(&sha_reset_mixed, ResetMode::Mixed, window, cx);
                    })),
            )
            .child(
                div()
                    .id("ctx-reset-hard")
                    .px_3()
                    .py_2()
                    .text_sm()
                    .text_color(rgb(0xf38ba8))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x313244)))
                    .child("Hard (discard all changes)")
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.reset_to_commit(&sha_reset_hard, ResetMode::Hard, window, cx);
                    })),
            )
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
