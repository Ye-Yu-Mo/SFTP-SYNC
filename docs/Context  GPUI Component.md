---
title: "Context | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/context"
author:
published:
created: 2025-12-05
description: "Learn about the Window and Context in GPUI."
tags:
  - "clippings"
---
[Skip to content](https://longbridge.github.io/gpui-component/docs/#VPContent)

The [Window](https://docs.rs/gpui/latest/gpui/struct.Window.html), [App](https://docs.rs/gpui/latest/gpui/struct.App.html), [Context](https://docs.rs/gpui/latest/gpui/struct.Context.html) and [Entity](https://docs.rs/gpui/latest/gpui/struct.Entity.html) are most important things in GPUI, it appears everywhere.

- [Window](https://docs.rs/gpui/latest/gpui/struct.Window.html) - The current window instance, which for handle the **Window Level** things.
- [App](https://docs.rs/gpui/latest/gpui/struct.App.html) - The current application instance, which for handle the **Application Level** things.
- [Context](https://docs.rs/gpui/latest/gpui/struct.Context.html) - The Entity Context instance, which for handle the **Context Level** things.
- [Entity](https://docs.rs/gpui/latest/gpui/struct.Entity.html) - The Entity instance, which for handle the **Entity Level** things.

For example:

```
rsfn new(window: &mut Window, cx: &mut App) {}

impl RenderOnce for MyElement {

    fn render(self, window: &mut Window, cx: &mut App) {}

}

impl Render for MyView {

    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) {}

}
```

INFO

As you can see, we always use `cx` to represent `App` and `Context<Self>`, which is the standard naming convention for GPUI, we can follow this convention to make our code more readable and maintainable.