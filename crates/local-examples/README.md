# Local Examples

This crate contains local demo apps for `bevy_extended_ui`.
It is intentionally **not** part of the root workspace.

## What is in this crate?

You can run these examples:

- `widget-overview` (default): general widget demo
- `typed-values`: typed values for ChoiceBox/RadioButton (`i32`, `bool`, enums, objects)
- `theming-provider`: theming provider usage
- `breakpoint`: media query / breakpoint behavior and selector matching
- `framework`: framework demo (requires `--features extended-framework`)

## Quick Start

Run the default example (`widget-overview`):

```bash
cargo run --manifest-path crates/local-examples/Cargo.toml
```

Run a specific example:

```bash
cargo run --manifest-path crates/local-examples/Cargo.toml -- theming-provider
cargo run --manifest-path crates/local-examples/Cargo.toml -- typed-values
cargo run --manifest-path crates/local-examples/Cargo.toml -- widget-overview
cargo run --manifest-path crates/local-examples/Cargo.toml -- breakpoint
```

Run the framework example:

```bash
cargo run --manifest-path crates/local-examples/Cargo.toml --features extended-framework -- framework
```
