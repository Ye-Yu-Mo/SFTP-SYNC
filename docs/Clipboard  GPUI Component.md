---
title: "Clipboard | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/clipboard"
author:
published:
created: 2025-12-05
description: "A button component that helps you copy text or other content to your clipboard."
tags:
  - "clippings"
---
## Clipboard

The Clipboard component provides an easy way to copy text or other data to the user's clipboard. It renders as a button with a copy icon that changes to a checkmark when content is successfully copied. The component supports both static values and dynamic content through callback functions.

## Import

```rust
rustuse gpui_component::clipboard::Clipboard;
```

## Usage

### Basic Clipboard

```rust
rustClipboard::new("my-clipboard")

    .value("Text to copy")

    .on_copied(|value, window, cx| {

        window.push_notification(format!("Copied: {}", value), cx)

    })
```

### Using Dynamic Values

The `value_fn` method allows you to provide a closure that generates the content to be copied at the time of the copy action.

- This is useful when the content to be copied depends on the current state of the application.
- And in some cases, it may have a larger overhead to compute, so you only want to do it when the user actually clicks the copy button.
```rust
rustlet state = some_state.clone();

Clipboard::new("dynamic-clipboard")

    .value_fn(move |_, cx| {

        state.read(cx).get_current_value()

    })

    .on_copied(|value, window, cx| {

        window.push_notification(format!("Copied: {}", value), cx)

    })
```

### With Custom Content

### In Input Fields

The Clipboard component is commonly used as a suffix in input fields:

```rust
rustuse gpui_component::input::{InputState, Input};

let url_state = cx.new(|cx| InputState::new(window, cx).default_value("https://github.com"));

Input::new(&url_state)

    .suffix(

        Clipboard::new("url-clipboard")

            .value_fn({

                let state = url_state.clone();

                move |_, cx| state.read(cx).value()

            })

            .on_copied(|value, window, cx| {

                window.push_notification(format!("URL copied: {}", value), cx)

            })

    )
```

## API Reference

- [Clipboard](https://docs.rs/gpui-component/latest/gpui_component/clipboard/struct.Clipboard.html)

## Examples

### Simple Text Copy

```rust
rustClipboard::new("simple")

    .value("Hello, World!")
```
```rust
rusth_flex()

    .gap_2()

    .child(Label::new("Your API Key:"))

    .child(

        Clipboard::new("feedback")

            .value("sk-1234567890abcdef")

            .on_copied(|_, window, cx| {

                window.push_notification("API key copied to clipboard", cx)

            })

    )
```

### Form Field Integration

```rust
rustuse gpui_component::{

    input::{InputState, Input},

    h_flex, label::Label

};

let api_key = "sk-1234567890abcdef";

h_flex()

    .gap_2()

    .items_center()

    .child(Label::new("API Key:"))

    .child(

        Input::new(&input_state)

            .value(api_key)

            .readonly(true)

            .suffix(

                Clipboard::new("api-key-copy")

                    .value(api_key)

                    .on_copied(|_, window, cx| {

                        window.push_notification("API key copied!", cx)

                    })

            )

    )
```

### Dynamic Content Copy

## Data Types

The Clipboard component currently supports copying text strings to the clipboard. It uses GPUI's `ClipboardItem::new_string()` method, which handles:

- Plain text strings
- UTF-8 encoded content
- Cross-platform clipboard integration