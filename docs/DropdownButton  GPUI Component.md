---
title: "DropdownButton | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/dropdown_button"
author:
published:
created: 2025-12-05
description: "A DropdownButton is a combination of a button and a trigger button. It allows us to display a dropdown menu when the trigger is clicked, but the left Button can still respond to independent events."
tags:
  - "clippings"
---
[Skip to content](https://longbridge.github.io/gpui-component/docs/components/#VPContent)

## DropdownButton

A [DropdownButton](https://docs.rs/gpui-component/latest/gpui_component/button/struct.DropdownButton.html) is a combination of a button and a trigger button. It allows us to display a dropdown menu when the trigger is clicked, but the left Button can still respond to independent events.

And more option methods of [Button](https://docs.rs/gpui-component/latest/gpui_component/button/struct.Button.html) are also available for the DropdownButton, such as setting different variants using [ButtonCustomVariant](https://docs.rs/gpui-component/latest/gpui_component/button/struct.ButtonCustomVariant.html), sizes using [Sizable](https://docs.rs/gpui-component/latest/gpui_component/trait.Sizable.html), adding icons, loading states.

## Import

```rust
rustuse gpui_component::button::{Button, DropdownButton};
```

## Usage

### Variants

Same as [Button](https://docs.rs/gpui-component/latest/gpui_component/button/struct.Button.html), DropdownButton supports different variants.