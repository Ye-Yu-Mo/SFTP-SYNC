---
title: "Select | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/select"
author:
published:
created: 2025-12-05
description: "Displays a list of options for the user to pick from—triggered by a button."
tags:
  - "clippings"
---
## Select

INFO

This component was named `Dropdown` in `<= 0.3.x`.

It has been renamed to `Select` to better reflect its purpose.

A select component that allows users to choose from a list of options.

Supports search functionality, grouped items, custom rendering, and various states. Built with keyboard navigation and accessibility in mind.

## Import

```rust
rustuse gpui_component::select::{

    Select, SelectState, SelectItem, SelectDelegate,

    SelectEvent, SearchableVec, SelectGroup

};
```

## Usage

### Basic Select

You can create a basic select dropdown by initializing a `SelectState` with a list of items.

The first type parameter of `SelectState` is the items for the state, which must implement the [SelectItem](https://docs.rs/gpui-component/latest/gpui_component/select/trait.SelectItem.html) trait.

The built-in implementations of `SelectItem` include common types like `String`, `SharedString`, and `&'static str`.

```rust
rustlet state = cx.new(|cx| {

    SelectState::new(

        vec!["Apple", "Orange", "Banana"],

        Some(IndexPath::default()), // Select first item

        window,

        cx,

    )

});

Select::new(&state)
```

### Placeholder

```rust
rustlet state = cx.new(|cx| {

    SelectState::new(

        vec!["Rust", "Go", "JavaScript"],

        None, // No initial selection

        window,

        cx,

    )

});

Select::new(&state)

    .placeholder("Select a language...")
```

### Searchable

Use `searchable(true)` to enable search functionality within the dropdown.

```rust
rustlet fruits = SearchableVec::new(vec![

    "Apple", "Orange", "Banana", "Grape", "Pineapple",

]);

let state = cx.new(|cx| {

    SelectState::new(fruits, None, window, cx).searchable(true)

});

Select::new(&state)

    .icon(IconName::Search) // Shows search icon
```

### Impl SelectItem

By default, we have implmemented `SelectItem` for common types like `String`, `SharedString` and `&'static str`. You can also create your own item types by implementing the `SelectItem` trait.

This is useful when you want to display complex data structures, and also want get that data type from `select_value` method.

You can also customize the search logic by overriding the `matches` method.

### Group Items

```rust
rustlet mut grouped_items = SearchableVec::new(vec![]);

// Group countries by first letter

grouped_items.push(

    SelectGroup::new("A")

        .items(vec![

            Country { name: "Australia".into(), code: "AU".into() },

            Country { name: "Austria".into(), code: "AT".into() },

        ])

);

grouped_items.push(

    SelectGroup::new("B")

        .items(vec![

            Country { name: "Brazil".into(), code: "BR".into() },

            Country { name: "Belgium".into(), code: "BE".into() },

        ])

);

let state = cx.new(|cx| {

    SelectState::new(grouped_items, None, window, cx)

});

Select::new(&state)
```

### Sizes

```rust
rustSelect::new(&state).large()

Select::new(&state) // medium (default)

Select::new(&state).small()
```

### Disabled State

```rust
rustSelect::new(&state).disabled(true)
```

### Cleanable

```rust
rustSelect::new(&state)

    .cleanable(true) // Show clear button when item is selected
```

### Custom Appearance

### Empty State

### Events

### Mutating

```rust
rust// Set by index

state.update(cx, |state, cx| {

    state.set_selected_index(Some(IndexPath::default().row(2)), window, cx);

});

// Set by value (requires PartialEq on Value type)

state.update(cx, |state, cx| {

    state.set_selected_value(&"US".into(), window, cx);

});

// Get current selection

let current_value = state.read(cx).selected_value();
```

Update items:

```rust
ruststate.update(cx, |state, cx| {

    let new_items = vec!["New Option 1".into(), "New Option 2".into()];

    state.set_items(new_items, window, cx);

});
```

## Examples

### Language Selector

```rust
rustlet languages = SearchableVec::new(vec![

    "Rust".into(),

    "TypeScript".into(),

    "Go".into(),

    "Python".into(),

    "JavaScript".into(),

]);

let state = cx.new(|cx| {

    SelectState::new(languages, None, window, cx)

});

Select::new(&state)

    .placeholder("Select language...")

    .title_prefix("Language: ")
```

### Country/Region Selector

### Integrated with Input Field

```rust
rust// Combined country code + phone input

h_flex()

    .border_1()

    .border_color(cx.theme().input)

    .rounded_lg()

    .w_full()

    .gap_1()

    .child(

        div().w(px(140.)).child(

            Select::new(&country_state)

                .appearance(false) // No border/background

                .py_2()

                .pl_3()

        )

    )

    .child(Divider::vertical())

    .child(

        div().flex_1().child(

            Input::new(&phone_input)

                .appearance(false)

                .placeholder("Phone number")

                .pr_3()

                .py_2()

        )

    )
```

### Multi-level Grouped Select

## Keyboard Shortcuts

| Key | Action |
| --- | --- |
| `Tab` | Focus dropdown |
| `Enter` | Open menu or select current item |
| `Up/Down` | Navigate options (opens menu if closed) |
| `Escape` | Close menu |
| `Space` | Open menu |

## Theming

The dropdown respects the current theme and uses the following theme tokens:

- `background` - Dropdown input background
- `input` - Border color
- `foreground` - Text color
- `muted_foreground` - Placeholder and disabled text
- `accent` - Selected item background
- `accent_foreground` - Placeholder text color
- `border` - Menu border
- `radius` - Border radius