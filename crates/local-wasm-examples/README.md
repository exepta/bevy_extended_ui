# Local WASM Examples

This crate is intentionally local-only and not part of the root workspace.

## Run in browser (WASM)

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd crates/local-wasm-examples
trunk serve --config Trunk.toml
```

Default demo is `theme-provider`.

To run widget overview instead:

```bash
trunk serve --config Trunk.toml --no-default-features --features widget-overview
```

## Optional native fallback

```bash
cargo run --manifest-path crates/local-wasm-examples/Cargo.toml
```

## Included demos

- `theming_provider_example` (default feature: `theme-provider`)
- `widget_overview_example` (feature: `widget-overview`)
