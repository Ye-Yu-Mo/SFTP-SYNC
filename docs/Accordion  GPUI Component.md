---
title: "Accordion | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/accordion"
author:
published:
created: 2025-12-05
description: "The accordion uses collapse internally to make it collapsible."
tags:
  - "clippings"
---
[Skip to content](https://longbridge.github.io/gpui-component/docs/components/#VPContent)

## Accordion

An accordion component that allows users to show and hide sections of content. It uses collapse functionality internally to create collapsible panels.

## Import

```rust
rustuse gpui_component::accordion::Accordion;
```

## Usage

### Basic Accordion

```rust
rustAccordion::new("my-accordion")

    .item(|item| {

        item.title("Section 1")

            .child("Content for section 1")

    })

    .item(|item| {

        item.title("Section 2")

            .child("Content for section 2")

    })

    .item(|item| {

        item.title("Section 3")

            .child("Content for section 3")

    })
```

### Multiple Open Items

By default, only one accordion item can be open at a time. Use `multiple()` to allow multiple items to be open:

```rust
rustAccordion::new("my-accordion")

    .multiple(true)

    .item(|item| item.title("Section 1").child("Content 1"))

    .item(|item| item.title("Section 2").child("Content 2"))
```

### With Borders

```rust
rustAccordion::new("my-accordion")

    .bordered(true)

    .item(|item| item.title("Section 1").child("Content 1"))
```

### Different Sizes

```rust
rustuse gpui_component::{Sizable as _, Size};

Accordion::new("my-accordion")

    .small()

    .item(|item| item.title("Small Section").child("Content"))

Accordion::new("my-accordion")

    .large()

    .item(|item| item.title("Large Section").child("Content"))
```

### Handle Toggle Events

```rust
rustAccordion::new("my-accordion")

    .on_toggle_click(|open_indices, window, cx| {

        println!("Open items: {:?}", open_indices);

    })

    .item(|item| item.title("Section 1").child("Content 1"))
```

### Disabled State

```rust
rustAccordion::new("my-accordion")

    .disabled(true)

    .item(|item| item.title("Disabled Section").child("Content"))
```

## API Reference

- [Accordion](https://docs.rs/gpui-component/latest/gpui_component/accordion/struct.Accordion.html)
- [AccordionItem](https://docs.rs/gpui-component/latest/gpui_component/accordion/struct.AccordionItem.html)

### Sizing

Implements [Sizable](https://docs.rs/gpui-component/latest/gpui_component/trait.Sizable.html) trait:

- `small()` - Small size
- `medium()` - Medium size (default)
- `large()` - Large size
- `xsmall()` - Extra small size

## Examples

### With Custom Icons

```rust
rustAccordion::new("my-accordion")

    .item(|item| {

        item.title(

            h_flex()

                .gap_2()

                .child(Icon::new(IconName::Settings))

                .child("Settings")

        )

        .child("Settings content here")

    })
```

### Nested Accordions

```rust
rustAccordion::new("outer")

    .item(|item| {

        item.title("Parent Section")

            .content(

                Accordion::new("inner")

                    .item(|item| item.title("Child 1").child("Content"))

                    .item(|item| item.title("Child 2").child("Content"))

            )

    })
```