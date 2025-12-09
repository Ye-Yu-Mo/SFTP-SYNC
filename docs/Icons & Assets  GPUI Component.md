---
title: "Icons & Assets | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/assets"
author:
published:
created: 2025-12-05
description: "Rust GUI components for building fantastic cross-platform desktop application by using GPUI."
tags:
  - "clippings"
---
## Icons & Assets

The [IconName](https://docs.rs/gpui_component/latest/gpui_component/icon/enum.IconName.html) and [Icon](https://docs.rs/gpui_component/latest/gpui_component/icon/struct.Icon.html) in GPUI Component provide a comprehensive set of icons and assets that can be easily integrated into your GPUI applications.

But for minimal size applications, **we have not embedded any icon assets by default** in `gpui-component` crate.

We split the icon assets into a separate crate [gpui-component-assets](https://crates.io/crates/gpui-component-assets) to allow developers to choose whether to include the icon assets in their applications or if you don't need the icons at all, you can build your own assets.

## Use default bundled assets

The [gpui-component-assets](https://crates.io/crates/gpui-component-assets) crate provides a default bundled assets implementation that includes all the icon files in the `assets/icons` folder.

To use the default bundled assets, you need to add the `gpui-component-assets` crate as a dependency in your `Cargo.toml`:

```toml
toml[dependencies]

gpui-component = "0.5.0-preview2"

gpui-component-assets = "0.5.0-preview2"
```

Then we need call the `with_assets` method when creating the GPUI application to register the asset source:

```
rsuse gpui::*;

use gpui_component_assets::Assets;

let app = Application::new().with_assets(Assets);
```

Now, we can use `IconName` and `Icon` in our application as usual, the all icon assets are loaded from the default bundled assets.

Continue [Use the icons](https://longbridge.github.io/gpui-component/docs/#use-the-icons) section to see how to use the icons in your application.

## Build you own assets

You may have a specific set of icons that you want to use in your application, or you may want to reduce the size of your application binary by including only the icons you need.

In this case, you can build your own assets by following these steps.

The [assets](https://github.com/longbridge/gpui-component/tree/main/crates/assets/assets/) folder in source code contains all the available icons in SVG format, every file is that GPUI Component support, it matched with the [IconName](https://docs.rs/gpui_component/latest/gpui_component/icon/enum.IconName.html) enum.

You can download the SVG files you need from the [assets](https://github.com/longbridge/gpui-component/tree/main/crates/assets/assets/) folder, or you can use your own SVG files by following the [IconName](https://docs.rs/gpui_component/latest/gpui_component/icon/enum.IconName.html) naming convention.

In GPUI application, we can use the [rust-embed](https://docs.rs/rust-embed/latest/rust_embed/) crate to embed the SVG files into the application binary.

And GPUI Application providers an `AssetSource` trait to load the assets.

We need call the `with_assets` method when creating the GPUI application to register the asset source:

## Use the icons

Now we can use the icons in our application:

```
rspub struct Example;

impl Render for Example {

    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {

        v_flex()

            .gap_2()

            .size_full()

            .items_center()

            .justify_center()

            .text_center()

            .child(IconName::Inbox)

            .child(IconName::Bot)

    }

}
```

## Resources

- [Lucide Icons](https://lucide.dev/) - The icon set used in GPUI Component is based on the open-source Lucide Icons library, which provides a wide range of customizable SVG icons.