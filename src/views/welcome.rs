use crate::state::RecentProjects;
use gpui::prelude::*;
use gpui::*;
use std::path::PathBuf;

pub struct WelcomeView {
    recent_projects: Entity<RecentProjects>,
    on_open_repository: Option<Box<dyn Fn(&PathBuf, &mut Window, &mut App) + 'static>>,
}

impl WelcomeView {
    pub fn new(recent_projects: Entity<RecentProjects>) -> Self {
        Self {
            recent_projects,
            on_open_repository: None,
        }
    }

    pub fn on_open_repository(
        mut self,
        handler: impl Fn(&PathBuf, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open_repository = Some(Box::new(handler));
        self
    }
}

impl IntoElement for WelcomeView {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div()
    }
}

impl RenderOnce for WelcomeView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let recent = self.recent_projects.read(cx);
        let projects: Vec<_> = recent.projects().to_vec();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .gap_8()
            // Logo / Title
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .text_3xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("Awabancha"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x9399b2))
                            .child("A fast Git GUI client"),
                    ),
            )
            // Open Repository Button
            .child(
                div()
                    .id("open-repo-button")
                    .px_6()
                    .py_3()
                    .rounded_lg()
                    .bg(rgb(0x89b4fa))
                    .text_color(rgb(0x1e1e2e))
                    .font_weight(FontWeight::SEMIBOLD)
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0xb4befe)))
                    .active(|s| s.bg(rgb(0x7287fd)))
                    .child("Open Repository"),
            )
            // Recent Projects
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w_80()
                    .when(!projects.is_empty(), |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x9399b2))
                                .mb_2()
                                .child("Recent Projects"),
                        )
                    })
                    .children(projects.into_iter().map(|project| {
                        div()
                            .id(ElementId::Name(
                                format!("recent-{}", project.path.display()).into(),
                            ))
                            .flex()
                            .items_center()
                            .gap_3()
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .bg(rgb(0x313244))
                            .hover(|s| s.bg(rgb(0x45475a)))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xcdd6f4))
                                            .text_ellipsis()
                                            .child(project.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x6c7086))
                                            .text_ellipsis()
                                            .child(project.path.display().to_string()),
                                    ),
                            )
                    })),
            )
    }
}
