---
title: "Menu | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/menu"
author:
published:
created: 2025-12-05
description: "Context menus and popup menus with support for icons, shortcuts, submenus, and various menu item types."
tags:
  - "clippings"
---
## PopupMenu

The Menu component provides both context menus (right-click menus) and popup menus with comprehensive features including icons, keyboard shortcuts, submenus, separators, checkable items, and custom elements. Built with accessibility and keyboard navigation in mind.

## Import

## Usage

### ContextMenu

Context menus appear when right-clicking on an element:

Dropdown menus are triggered by buttons or other interactive elements:

TIP

As you see, the each menu item is associated with an [Action](https://docs.rs/gpui/latest/gpui/trait.Action.html), we choice this design to better integrate with GPUI's action and key binding system, allowing menu items to automatically display keyboard shortcuts when applicable.

So, the [Action](https://docs.rs/gpui/latest/gpui/trait.Action.html) is the recommended way to define menu item behaviors.

However, if you prefer not to use [Action](https://docs.rs/gpui/latest/gpui/trait.Action.html) s, you can create custom menu items using the `item` method along with [PopupMenuItem](https://docs.rs/gpui-component/latest/gpui_component/menu/struct.PopupMenuItem.html). There have a `on_click` callback to handle the click event directly.

### Anchor Position

Control where the dropdown menu appears relative to the trigger:

### Icons

Add icons to menu items for better visual clarity:

### Disabled State

Create disabled menu items that cannot be activated:

### Check state

Create menu items that show a check state:

By default, the check icon will be shown on the left side of the menu item, if this menu item has an icon, the check icon will replace the icon on the left side.

There also have a `check_side` option for you to config the check icon to be shown on the right side:

### Separators

Use separators to group related menu items:

### Labels

Add non-interactive labels to organize menu sections:

Create menu items that open external links:

### Custom Element

Create custom menu items with complex content:

### Keyboard Shortcuts

Menu items automatically display keyboard shortcuts if they're bound to actions:

Create nested menus with submenu support:

Add icons to submenu headers:

For menus with many items, enable scrolling:

Control menu dimensions:

### Action Context

Set the focus context for handling menu actions:

## API Reference

- [PopupMenu](https://docs.rs/gpui-component/latest/gpui_component/menu/struct.PopupMenu.html)
- [context\_menu](https://docs.rs/gpui-component/latest/gpui_component/menu/trait.ContextMenuExt.html#method.context_menu)
- [PopupMenuItem](https://docs.rs/gpui-component/latest/gpui_component/menu/struct.PopupMenuItem.html)

## Examples

Sometimes you may not like to define an action for a menu item, you just want add a `on_click` handler, in this case, the `item` and [PopupMenuItem](https://docs.rs/gpui-component/latest/gpui_component/menu/struct.PopupMenuItem.html) can help you:

## Keyboard Shortcuts

| Key | Action |
| --- | --- |
| `↑` / `↓` | Navigate menu items |
| `←` / `→` | Navigate submenus |
| `Enter` / `Space` | Activate menu item |
| `Escape` | Close menu |
| `Tab` | Close menu and focus next element |

## Best Practices

1. **Group Related Items**: Use separators to group related functionality
2. **Consistent Icons**: Use consistent iconography across your application
3. **Logical Order**: Place most common actions at the top
4. **Keyboard Shortcuts**: Provide shortcuts for frequently used actions
5. **Context Awareness**: Show only relevant items for the current context
6. **Progressive Disclosure**: Use submenus for complex hierarchies
7. **Clear Labels**: Use descriptive, action-oriented labels
8. **Reasonable Limits**: Use scrollable menus for more than 10-15 items