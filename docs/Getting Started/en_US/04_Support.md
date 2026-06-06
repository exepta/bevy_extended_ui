---
title: Support
---

# Support and Troubleshooting

## Where to start

Use the local examples before opening an issue. They cover the most common integration paths:

```bash
cargo run --manifest-path crates/local-examples/Cargo.toml
cd crates/local-wasm-examples
trunk serve --config Trunk.toml --no-default-features --features widget-overview
```

## Useful diagnostics

- Check that `ExtendedUiPlugin` is registered after Bevy `DefaultPlugins`.
- Check that asset paths are relative to the active Bevy asset root.
- Check that HTML handler names exactly match `#[html_fn("...")]` strings.
- Check `CSS_USAGE.md` before assuming a browser CSS property is supported.
- For WASM, open the browser console and verify that `assets/` files were copied by Trunk.

## Issue checklist

When reporting a problem, include:

- Bevy version and `bevy_extended_ui` version.
- Target: native or `wasm32-unknown-unknown`.
- Enabled Cargo features.
- Minimal HTML, CSS, and Rust handler code.
- Browser console output for WASM issues.
- A short note about expected behavior and actual behavior.

## Next step

For practical examples, continue with [Examples](03_Examples.md).
