---
title: "Theme | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/theme"
author:
published:
created: 2025-12-05
description: "Rust GUI components for building fantastic cross-platform desktop application by using GPUI."
tags:
  - "clippings"
---
[Skip to content](https://longbridge.github.io/gpui-component/docs/#VPContent)

## Theme

All components support theming through the built-in Theme system, the [ActiveTheme](https://docs.rs/gpui-component/latest/gpui_component/theme/trait.ActiveTheme.html) trait provides access to the current theme colors:

```
rsuse gpui_component::{ActiveTheme as _};

// Access theme colors in your components

cx.theme().primary

cx.theme().background

cx.theme().foreground
```

So if you want use the colors from the current theme, you should keep your component or view have [App](https://docs.rs/gpui/latest/gpui/struct.App.html) context.

## Theme Registry

There have more than 20 built-in themes available in [themes](https://github.com/longbridge/gpui-component/tree/main/themes) folder.

[https://github.com/longbridge/gpui-component/tree/main/themes](https://github.com/longbridge/gpui-component/tree/main/themes)

And we have a [ThemeRegistry](https://docs.rs/gpui-component/latest/gpui_component/theme/struct.ThemeRegistry.html) to help us to load themes.