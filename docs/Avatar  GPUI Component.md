---
title: "Avatar | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/components/avatar"
author:
published:
created: 2025-12-05
description: "Displays a user avatar image with fallback options."
tags:
  - "clippings"
---
[Skip to content](https://longbridge.github.io/gpui-component/docs/components/#VPContent)

## Avatar

The Avatar component displays user profile images with intelligent fallbacks. When no image is provided, it shows user initials or a placeholder icon. The component supports various sizes and can be grouped together for team displays.

## Import

```rust
rustuse gpui_component::avatar::{Avatar, AvatarGroup};
```

## Usage

### Basic Avatar

You can create an [Avatar](https://docs.rs/gpui-component/latest/gpui_component/avatar/struct.Avatar.html) by providing an image source URL and a user name:

```rust
rustAvatar::new()

    .name("John Doe")

    .src("https://example.com/avatar.jpg")
```

### Avatar with Fallback Text

When no image source is provided, the Avatar displays user initials with an automatically generated color background:

```rust
rust// Shows "JD" initials with colored background

Avatar::new()

    .name("John Doe")

// Shows "JS" initials

Avatar::new()

    .name("Jane Smith")
```

### Avatar Placeholder

For anonymous users or when no name is provided:

```rust
rustuse gpui_component::IconName;

// Default user icon placeholder

Avatar::new()

// Custom placeholder icon

Avatar::new()

    .placeholder(IconName::Building2)
```

### Avatar Sizes

```rust
rustAvatar::new()

    .name("John Doe")

    .xsmall()

Avatar::new()

    .name("John Doe")

    .small()

Avatar::new()

    .name("John Doe")   // 48px (default medium)

Avatar::new()

    .name("John Doe")

    .large()

// Custom size

Avatar::new()

    .name("John Doe")

    .with_size(px(100.))
```

### Custom Styling

```rust
rustAvatar::new()

    .src("https://example.com/avatar.jpg")

    .with_size(px(100.))

    .border_3()

    .border_color(cx.theme().foreground)

    .shadow_sm()

    .rounded(px(20.))  // Custom border radius
```

## AvatarGroup

The [AvatarGroup](https://docs.rs/gpui-component/latest/gpui_component/avatar/struct.AvatarGroup.html) component allows you to display multiple avatars in a compact, overlapping layout:

### Basic Group

```rust
rustAvatarGroup::new()

    .child(Avatar::new().src("https://example.com/user1.jpg"))

    .child(Avatar::new().src("https://example.com/user2.jpg"))

    .child(Avatar::new().src("https://example.com/user3.jpg"))

    .child(Avatar::new().name("John Doe"))
```

### Group with Limit

```rust
rustAvatarGroup::new()

    .limit(3)  // Show maximum 3 avatars

    .child(Avatar::new().src("https://example.com/user1.jpg"))

    .child(Avatar::new().src("https://example.com/user2.jpg"))

    .child(Avatar::new().src("https://example.com/user3.jpg"))

    .child(Avatar::new().src("https://example.com/user4.jpg"))  // Hidden

    .child(Avatar::new().src("https://example.com/user5.jpg"))  // Hidden
```

### Group with Ellipsis

Show an ellipsis indicator when avatars are hidden due to the limit.

In this example, only 3 avatars are shown, and "..." indicates there are more:

```rust
rustAvatarGroup::new()

    .limit(3)

    .ellipsis()  // Shows "..." when limit is exceeded

    .child(Avatar::new().src("https://example.com/user1.jpg"))

    .child(Avatar::new().src("https://example.com/user2.jpg"))

    .child(Avatar::new().src("https://example.com/user3.jpg"))

    .child(Avatar::new().src("https://example.com/user4.jpg"))

    .child(Avatar::new().src("https://example.com/user5.jpg"))
```

### Group Sizes

The \[Sizeable\] trait can also be applied to AvatarGroup, and it will set the size for all contained avatars.

```rust
rust// Extra small group

AvatarGroup::new()

    .xsmall()

    .child(Avatar::new().name("A"))

    .child(Avatar::new().name("B"))

    .child(Avatar::new().name("C"))

// Small group

AvatarGroup::new()

    .small()

    .child(Avatar::new().name("A"))

    .child(Avatar::new().name("B"))

// Medium group (default)

AvatarGroup::new()

    .child(Avatar::new().name("A"))

    .child(Avatar::new().name("B"))

// Large group

AvatarGroup::new()

    .large()

    .child(Avatar::new().name("A"))

    .child(Avatar::new().name("B"))
```

### Adding Multiple Avatars

```rust
rustlet avatars = vec![

    Avatar::new().src("https://example.com/user1.jpg"),

    Avatar::new().src("https://example.com/user2.jpg"),

    Avatar::new().name("John Doe"),

];

AvatarGroup::new()

    .children(avatars)

    .limit(5)

    .ellipsis()
```

## API Reference

- [Avatar](https://docs.rs/gpui-component/latest/gpui_component/avatar/struct.Avatar.html)
- [AvatarGroup](https://docs.rs/gpui-component/latest/gpui_component/avatar/struct.AvatarGroup.html)

## Examples

### Team Display

```rust
rustuse gpui_component::{h_flex, v_flex};

v_flex()

    .gap_4()

    .child("Development Team")

    .child(

        AvatarGroup::new()

            .limit(4)

            .ellipsis()

            .child(Avatar::new().name("Alice Johnson").src("https://example.com/alice.jpg"))

            .child(Avatar::new().name("Bob Smith").src("https://example.com/bob.jpg"))

            .child(Avatar::new().name("Charlie Brown"))

            .child(Avatar::new().name("Diana Prince"))

            .child(Avatar::new().name("Eve Wilson"))

    )
```

### User Profile Header

```rust
rusth_flex()

    .items_center()

    .gap_4()

    .child(

        Avatar::new()

            .src("https://example.com/profile.jpg")

            .name("John Doe")

            .large()

            .border_2()

            .border_color(cx.theme().primary)

    )

    .child(

        v_flex()

            .child("John Doe")

            .child("Software Engineer")

    )
```

### Anonymous User

```rust
rustuse gpui_component::IconName;

Avatar::new()

    .placeholder(IconName::UserCircle)

    .medium()
```

### Avatar with Custom Colors

```rust
rust// The avatar automatically generates colors based on the name

// Different names will get different colors from the color palette

Avatar::new().name("Alice")    // Gets one color

Avatar::new().name("Bob")      // Gets a different color

Avatar::new().name("Charlie")  // Gets another color
```