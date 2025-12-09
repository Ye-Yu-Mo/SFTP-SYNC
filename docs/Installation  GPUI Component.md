---
title: "Installation | GPUI Component"
source: "https://longbridge.github.io/gpui-component/docs/installation"
author:
published:
created: 2025-12-05
description: "Rust GUI components for building fantastic cross-platform desktop application by using GPUI."
tags:
  - "clippings"
---
[Skip to content](https://longbridge.github.io/gpui-component/docs/#VPContent)

## Installation

Before you start to build your application with `gpui-component`, you need to install the library.

## System Requirements

We can development application on macOS, Windows or Linux.

### macOS

- macOS 15 or later
- Xcode command line tools

## Windows

- Windows 10 or later

There have a bootstrap script to help install the required toolchain and dependencies.

You can run the script in PowerShell:

```
ps.\script\install-window.ps1
```

## Linux

Run `./script/bootstrap` to install system dependencies.

## Rust and Cargo

We use Rust programming language to build the `gpui-component` library. Make sure you have Rust and Cargo installed on your system.

- Rust 1.90 or later
- Cargo (comes with Rust)

To install the `gpui-component` library, you can use Cargo, the Rust package manager. Add the following line to your `Cargo.toml` file under the `[dependencies]` section:

```toml
tomlgpui = "0.2.2"

gpui-component = "0.5.0-preview2"
```