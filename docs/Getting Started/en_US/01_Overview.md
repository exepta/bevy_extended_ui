---
title: Overview
---

# Bevy Extended UI Overview

<p class="description">Bevy Extended UI is an open-source crate that extends Bevy's default UI with an HTML/CSS workflow and reusable widgets.</p>

## What this crate solves

Bevy's built-in UI is fast and flexible, but larger interfaces often require a lot of manual setup.
Bevy Extended UI adds a declarative workflow so you can define UI structure in HTML, style it with CSS, and connect events directly to Rust handlers.

This is especially useful when you need form-like or app-like interfaces with many interactive elements.

## Core capabilities

- HTML-based UI sources loaded through Bevy assets
- CSS styling support, including breakpoints and animations
- Built-in widgets (for example sliders, checkboxes, choice boxes, date pickers)
- Event bindings from HTML attributes to Rust (`onclick`, `onchange`, `action`, keyboard and drag events)
- Hot reload support for faster UI iteration
- Optional localization backends (`fluent`, `properties-lang`)
- WASM-oriented feature presets (`wasm-default`, `wasm-breakpoints`, `clipboard-wasm`)

## Typical workflow

1. Add dependencies to `Cargo.toml`.
2. Add `ExtendedUiPlugin` to your app.
3. Create and load an HTML file from `assets/`.
4. Register the HTML source in `UiRegistry`.
5. Bind event handlers using `#[html_fn(...)]` from `bevy_extended_ui_macros`.

## Good use cases

- Complex HUD or menu screens in games
- Tooling-like interfaces with forms and dynamic state
- Projects that benefit from shared HTML/CSS authoring patterns

## Next step

Continue with the installation guide: [Install](02_Install.md).
