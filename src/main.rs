mod config;
mod connection;
mod secrets;
mod security;
mod model;
mod sync;
mod task_queue;
mod watcher;
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
            let (initial_settings, initial_targets) = config::load_state();

            cx.open_window(WindowOptions::default(), |window, cx| {
                let state =
                    cx.new(|_| AppState::new(initial_settings.clone(), initial_targets.clone()));
                let view = cx.new(|_| AppView::new(state.clone()));

                cx.new(|cx| Root::new(view.into(), window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
