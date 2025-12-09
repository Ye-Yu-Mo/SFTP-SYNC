---
title: "Alert | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/alert"
author:
published:
created: 2025-12-05
description: "Displays a callout for user attention."
tags:
  - "clippings"
---
## Alert

A versatile alert component for displaying important messages to users. Supports multiple variants (info, success, warning, error), custom icons, optional titles, closable functionality, and banner mode. Perfect for notifications, status messages, and user feedback.

## Import

```rust
rustuse gpui_component::alert::Alert;
```

## Usage

### Basic Alert

```rust
rustAlert::new("alert-id", "This is a basic alert message.")
```

### Alert with Title

```rust
rustAlert::new("alert-with-title", "Your changes have been saved successfully.")

    .title("Success!")
```

### Alert Variants

```rust
rust// Info alert (blue)

Alert::info("info-alert", "This is an informational message.")

    .title("Information")

// Success alert (green)

Alert::success("success-alert", "Your operation completed successfully.")

    .title("Success!")

// Warning alert (yellow/orange)

Alert::warning("warning-alert", "Please review your settings before proceeding.")

    .title("Warning")

// Error alert (red)

Alert::error("error-alert", "An error occurred while processing your request.")

    .title("Error")
```

### Alert Sizes

```rust
rustuse gpui_component::{alert::Alert, Sizable as _};

Alert::info("alert", "Message content")

    .xsmall()

    .title("XSmall Alert")

Alert::info("alert", "Message content")

    .small()

    .title("Small Alert")

Alert::info("alert", "Message content")

    .title("Medium Alert")

Alert::info("alert", "Message content")

    .large()

    .title("Large Alert")
```

### Closable Alerts

When you add an `on_close` handler, a close button appears on the alert:

```rust
rustAlert::info("closable-alert", "This alert can be dismissed.")

    .title("Dismissible")

    .on_close(|_event, _window, _cx| {

        println!("Alert was closed");

        // Handle alert dismissal

    })
```

### Banner Mode

Banner alerts take full width and don't display titles:

### Custom Icons

```rust
rustuse gpui_component::IconName;

Alert::new("custom-icon", "Meeting scheduled for tomorrow at 3 PM.")

    .title("Calendar Reminder")

    .icon(IconName::Calendar)
```

### With Markdown Content

We can use `TextView` to render formatted (Markdown or HTML) text within the alert, for displaying lists, bold text, links, etc.

```rust
rustuse gpui_component::text::TextView;

Alert::error(

    "error-with-markdown",

    TextView::markdown(

        "error-message",

        "Please verify your billing information and try again.\n\

        - Check your card details\n\

        - Ensure sufficient funds\n\

        - Verify billing address",

        window,

        cx,

    ),

)

.title("Payment Failed")
```

### Conditional Visibility

```rust
rustAlert::info("conditional-alert", "This alert may be hidden.")

    .title("Conditional")

    .visible(should_show_alert) // boolean condition
```

## API Reference

- [Alert](https://docs.rs/gpui-component/latest/gpui_component/alert/struct.Alert.html)

## Examples

### Success Notification

```rust
rustAlert::success("save-success", "Your profile has been updated successfully.")

    .title("Changes Saved")

    .on_close(|_, _, _| {

        // Auto-dismiss after showing

    })
```

### System Status Banner

```rust
rustAlert::info("update-available", "A new version of the application is available.")

    .title("Update Available")

    .icon(IconName::Download)

    .on_close(cx.listener(|this, _, _, cx| {

        // Handle update or dismiss

        this.handle_update_notification(cx);

    }))
```