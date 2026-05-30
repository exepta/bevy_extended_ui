# Local Examples

This crate is intentionally local-only and not part of the root workspace.

Includes examples:

- `typed_values_example` — ChoiceBox/RadioButton typed values (i32, bool, enum, object)
- `theming_provider_example`
- `widget_overview_example`
- `breakpoint_example` — media query / breakpoint behavior and selector matching
- `framework_example` (requires `--features extended-framework`)

Run the default demo (`widget-overview`):

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

Run framework mode:

```bash
cargo run --manifest-path crates/local-examples/Cargo.toml --features extended-framework -- framework
```
