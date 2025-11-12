mod config;
mod model;
mod view;

use gpui::*;
use gpui_component::Root;

use model::AppState;
use view::AppView;

fn main() {
    let app = Application::new();

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            let initial_settings = config::load_settings();

            cx.open_window(WindowOptions::default(), |window, cx| {
                let state = cx.new(|_| AppState::new(initial_settings.clone()));
                let view = cx.new(|_| AppView::new(state.clone()));

                cx.new(|cx| Root::new(view.into(), window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
