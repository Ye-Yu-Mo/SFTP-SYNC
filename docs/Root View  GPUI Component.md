---
title: "Root View | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/root"
author:
published:
created: 2025-12-05
description: "Rust GUI components for building fantastic cross-platform desktop application by using GPUI."
tags:
  - "clippings"
---
## Root View

The [Root](https://docs.rs/gpui-component/latest/gpui_component/root/struct.Root.html) component for as the root provider of GPUI Component features in a window. We must to use [Root](https://docs.rs/gpui-component/latest/gpui_component/root/struct.Root.html) as the **first level child** of a window to enable GPUI Component features.

This is important, if we don't use [Root](https://docs.rs/gpui-component/latest/gpui_component/root/struct.Root.html) as the first level child of a window, there will have some unexpected behaviors.

```
rsfn main() {

    let app = Application::new();

    app.run(move |cx| {

        // This must be called before using any GPUI Component features.

        gpui_component::init(cx);

        cx.spawn(async move |cx| {

            cx.open_window(WindowOptions::default(), |window, cx| {

                let view = cx.new(|_| Example);

                // This first level on the window, should be a Root.

                cx.new(|cx| Root::new(view, window, cx))

            })?;

            Ok::<_, anyhow::Error>(())

        })

        .detach();

    });

}
```

## Overlays

We have dialogs, sheets, notifications, we need placement for them to show, so [Root](https://docs.rs/gpui-component/latest/gpui_component/root/struct.Root.html) provides methods to render these overlays:

- [Root::render\_dialog\_layer](https://docs.rs/gpui-component/latest/gpui_component/struct.Root.html#method.render_dialog_layer) - Render the current opened modals.
- [Root::render\_sheet\_layer](https://docs.rs/gpui-component/latest/gpui_component/struct.Root.html#method.render_sheet_layer) - Render the current opened drawers.
- [Root::render\_notification\_layer](https://docs.rs/gpui-component/latest/gpui_component/struct.Root.html#method.render_notification_layer) - Render the notification list.

We can put these layers in the `render` method your first level view (Root > YourFirstView):

```
rsstruct MyApp;

impl Render for MyApp {

    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {

        div()

            .size_full()

            .child("My App Content")

            .children(Root::render_dialog_layer(cx))

            .children(Root::render_sheet_layer(cx))

            .children(Root::render_notification_layer(cx))

    }

}
```

TIP

Here the example we used `children` method, it because if there is no opened dialogs, sheets, notifications, these methods will return `None`, so GPUI will not render anything.