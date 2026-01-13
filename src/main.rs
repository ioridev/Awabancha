mod actions;
mod app;
mod components;
mod git;
mod state;
mod views;

use app::Awabancha;
use gpui::*;

fn main() {
    env_logger::init();

    Application::new().run(|cx: &mut App| {
        // Load assets
        cx.set_global(Awabancha::load_assets());

        // Register actions
        actions::register_actions(cx);

        // Calculate window bounds
        let bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);

        // Open the main window
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Awabancha".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(9.0), px(9.0))),
                }),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Awabancha::new(window, cx)),
        )
        .expect("Failed to open window");

        cx.activate(true);
    });
}
