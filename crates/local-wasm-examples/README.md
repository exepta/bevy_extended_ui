# Local WASM Examples

This crate is intentionally local-only and not part of the root workspace.

## Run in browser (WASM)

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd crates/local-wasm-examples
trunk serve --config Trunk.toml
```

## Optional native fallback

```bash
cargo run --manifest-path crates/local-wasm-examples/Cargo.toml
```
