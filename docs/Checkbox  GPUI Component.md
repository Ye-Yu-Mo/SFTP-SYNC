---
title: "Checkbox | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/checkbox"
author:
published:
created: 2025-12-05
description: "A control that allows the user to toggle between checked and not checked."
tags:
  - "clippings"
---
[Skip to content](https://longbridge.github.io/gpui-component/docs/components/#VPContent)

## Checkbox

A checkbox component for binary choices. Supports labels, disabled state, and different sizes.

## Import

```rust
rustuse gpui_component::checkbox::Checkbox;
```

## Usage

### Basic Checkbox

The `on_click` callback is triggered when the user toggles the checkbox, receiving the **new checked state**.

### Controlled Checkbox

```rust
ruststruct MyView {

    is_checked: bool,

}

impl Render for MyView {

    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {

        Checkbox::new("checkbox")

            .label("Option")

            .checked(self.is_checked)

            .on_click(cx.listener(|view, checked, _, cx| {

                view.is_checked = *checked;

                cx.notify();

            }))

    }

}
```

### Different Sizes

```rust
rustCheckbox::new("cb").text_xs().label("Extra Small")

Checkbox::new("cb").text_sm().label("Small")

Checkbox::new("cb").label("Medium") // default

Checkbox::new("cb").text_lg().label("Large")
```

### Disabled State

```rust
rustCheckbox::new("checkbox")

    .label("Disabled checkbox")

    .disabled(true)

    .checked(false)
```

### Without Label

```rust
rustCheckbox::new("checkbox")

    .checked(true)
```

### Custom Tab Order

```rust
rustCheckbox::new("checkbox")

    .label("Custom tab order")

    .tab_index(2)

    .tab_stop(true)
```

## API Reference

- [Checkbox](https://docs.rs/gpui-component/latest/gpui_component/checkbox/struct.Checkbox.html)

### Styling

Implements `Sizable` and `Disableable` traits:

- `text_xs()` - Extra small text
- `text_sm()` - Small text
- `text_base()` - Base text (default)
- `text_lg()` - Large text
- `disabled(bool)` - Disabled state

## Examples

### Checkbox List

```rust
rustv_flex()

    .gap_2()

    .child(Checkbox::new("cb1").label("Option 1").checked(true))

    .child(Checkbox::new("cb2").label("Option 2").checked(false))

    .child(Checkbox::new("cb3").label("Option 3").checked(false))
```