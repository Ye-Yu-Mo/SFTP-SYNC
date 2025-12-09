---
title: "ElementId | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/element_id"
author:
published:
created: 2025-12-05
description: "To introduce the ElementId concept in GPUI."
tags:
  - "clippings"
---
The [ElementId](https://docs.rs/gpui/latest/gpui/enum.ElementId.html) is a unique identifier for a GPUI element. It is used to reference elements in the GPUI component tree.

Before you start using GPUI and GPUI Component, you need to understand the [ElementId](https://docs.rs/gpui/latest/gpui/enum.ElementId.html).

For example:

```
rsdiv().id("my-element").child("Hello, World!")
```

In this case, the `div` element has an `id` of `"my-element"`. The add `id` is used for GPUI for binding events, for example `on_click` or `on_mouse_move`, the `element` with `id` in GPUI we call [Stateful<E>](https://docs.rs/gpui/latest/gpui/struct.Stateful.html).

We also use `id` (actually, it uses [GlobalElementId](https://docs.rs/gpui/latest/gpui/struct.GlobalElementId.html) internally in GPUI) to manage the `state` in some elements, by using `window.use_keyed_state`, so it is important to keep the `id` unique.

## Unique

The `id` should be unique within the layout scope (In a same [Stateful<E>](https://docs.rs/gpui/latest/gpui/struct.Stateful.html) parent).

For example we have a list with multiple items:

```
rsdiv().id("app").child(

    div().id("list1").child(vec![

        div().id(1).child("Item 1"),

        div().id(2).child("Item 2"),

        div().id(3).child("Item 3"),

    ])

).child(

    div().id("list2").child(vec![

        div().id(1).child("Item 1"),

    ])

)
```

In this case, we can named the child items with a very simple id, because they are have a parent `list1` element with an `id`.

GPUI internal will generate [GlobalElementId](https://docs.rs/gpui/latest/gpui/struct.GlobalElementId.html) with the parent elements's `id`, in this example, the `Item 1` will have global\_id:

```
rs["app", "list1", 1]
```

And the `Item 1` in `list2` will have global\_id:

```
rs["app", "list2", 1]
```

So we can named the child items with a very simple id.