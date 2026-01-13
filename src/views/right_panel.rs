use crate::components::{TextInputChanged, TextInputView};
use crate::git::CommitInfo;
use crate::state::GitState;
use crate::views::CommitGraph;
use chrono::Datelike;
use gpui::prelude::*;
use gpui::*;

pub struct RightPanel {
    git_state: Entity<GitState>,
    commit_graph: Entity<CommitGraph>,
    search_input: Entity<TextInputView>,
    search_query: String,
    search_results: Vec<CommitInfo>,
}

impl RightPanel {
    pub fn new(git_state: Entity<GitState>, cx: &mut Context<Self>) -> Self {
        let git_state_clone = git_state.clone();
        let commit_graph = cx.new(|cx| CommitGraph::new(git_state_clone.clone(), cx));

        // Create search input
        let search_input = cx.new(|cx| {
            TextInputView::new(cx).with_placeholder("Search commits by message, author, or SHA...")
        });

        // Handle search input changes via subscription
        let git_state_for_search = git_state.clone();
        cx.subscribe(&search_input, move |this, _input, event: &TextInputChanged, cx| {
            this.search_query = event.0.to_string();
            // Search commits
            let results = git_state_for_search
                .read(cx)
                .search_commits(&this.search_query, 50);
            this.search_results = results;
            cx.notify();
        })
        .detach();

        Self {
            git_state,
            commit_graph,
            search_input,
            search_query: String::new(),
            search_results: Vec::new(),
        }
    }

    fn clear_search(&mut self, cx: &mut Context<Self>) {
        self.search_query.clear();
        self.search_results.clear();
        self.search_input.update(cx, |input, cx| {
            input.set_content("", cx);
        });
        cx.notify();
    }

    fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
        use chrono::Timelike;
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            timestamp.year(),
            timestamp.month(),
            timestamp.day(),
            timestamp.hour(),
            timestamp.minute()
        )
    }
}

impl Render for RightPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_search = !self.search_query.is_empty();
        let search_results = self.search_results.clone();
        let commit_count = self
            .git_state
            .read(cx)
            .commits
            .as_ref()
            .map(|c| c.nodes.len())
            .unwrap_or(0);

        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            // Header with title and search
            .child(
                div()
                    .flex()
                    .flex_col()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x181825))
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .gap_2()
                    // Title row
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0x9399b2))
                                            .child("Commit History"),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_px()
                                            .rounded_sm()
                                            .bg(rgb(0x313244))
                                            .text_xs()
                                            .text_color(rgb(0x6c7086))
                                            .child(if has_search {
                                                format!("{} results", search_results.len())
                                            } else {
                                                format!("{} commits", commit_count)
                                            }),
                                    ),
                            ),
                    )
                    // Search input row
                    .child(
                        div()
                            .relative()
                            .child(self.search_input.clone())
                            .when(has_search, |this| {
                                this.child(
                                    div()
                                        .id("clear-search")
                                        .absolute()
                                        .right_2()
                                        .top_0()
                                        .bottom_0()
                                        .flex()
                                        .items_center()
                                        .child(
                                            div()
                                                .id("clear-search-btn")
                                                .px_2()
                                                .py_1()
                                                .rounded_sm()
                                                .text_xs()
                                                .text_color(rgb(0x9399b2))
                                                .cursor_pointer()
                                                .hover(|s| {
                                                    s.bg(rgb(0x45475a)).text_color(rgb(0xcdd6f4))
                                                })
                                                .child("×")
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.clear_search(cx);
                                                })),
                                        ),
                                )
                            }),
                    ),
            )
            // Content: Search results or commit graph
            .child(
                div()
                    .id("commit-content")
                    .flex_1()
                    .overflow_scroll()
                    .when(has_search, |this| {
                        // Show search results as a list
                        this.child(
                            div().flex().flex_col().when(
                                search_results.is_empty(),
                                |this| {
                                    this.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .h_32()
                                            .text_sm()
                                            .text_color(rgb(0x6c7086))
                                            .child("No commits found"),
                                    )
                                },
                            )
                            .when(!search_results.is_empty(), |this| {
                                this.children(search_results.into_iter().map(|commit| {
                                    SearchResultItem::new(commit)
                                }))
                            }),
                        )
                    })
                    .when(!has_search, |this| {
                        // Show commit graph
                        this.child(self.commit_graph.clone())
                    }),
            )
    }
}

/// A search result item component
struct SearchResultItem {
    commit: CommitInfo,
}

impl SearchResultItem {
    fn new(commit: CommitInfo) -> Self {
        Self { commit }
    }
}

impl IntoElement for SearchResultItem {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for SearchResultItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let timestamp = RightPanel::format_timestamp(&self.commit.timestamp);

        div()
            .id(SharedString::from(self.commit.sha.clone()))
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(rgb(0x313244))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(0x313244)))
            .child(
                div()
                    .flex()
                    .items_start()
                    .gap_3()
                    // SHA badge
                    .child(
                        div()
                            .px_2()
                            .py_px()
                            .rounded_sm()
                            .bg(rgb(0x313244))
                            .text_xs()
                            .font_family("monospace")
                            .text_color(rgb(0x89b4fa))
                            .child(self.commit.short_sha.clone()),
                    )
                    // Commit details
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xcdd6f4))
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .child(self.commit.message.lines().next().unwrap_or("").to_string()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .mt_1()
                                    .text_xs()
                                    .text_color(rgb(0x6c7086))
                                    .child(self.commit.author.clone())
                                    .child("•")
                                    .child(timestamp),
                            ),
                    ),
            )
            // Branch/tag badges if any
            .when(!self.commit.branches.is_empty() || !self.commit.tags.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_1()
                        .mt_2()
                        .px_4()
                        .children(self.commit.branches.iter().map(|branch| {
                            div()
                                .px_2()
                                .py_px()
                                .rounded_sm()
                                .bg(rgb(0x89b4fa))
                                .text_xs()
                                .text_color(rgb(0x1e1e2e))
                                .child(branch.clone())
                        }))
                        .children(self.commit.tags.iter().map(|tag| {
                            div()
                                .px_2()
                                .py_px()
                                .rounded_sm()
                                .bg(rgb(0xf9e2af))
                                .text_xs()
                                .text_color(rgb(0x1e1e2e))
                                .child(tag.clone())
                        })),
                )
            })
    }
}
