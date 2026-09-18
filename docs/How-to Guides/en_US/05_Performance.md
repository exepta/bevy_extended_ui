---
title: HTML and CSS Performance
---

# HTML and CSS Performance

The HTML/CSS runtime avoids work on unchanged data. No additional feature flag
or application API is required.

## What Changed

- HTML binding snapshots and fingerprints are only recomputed when their input
  resources change. Notifications are cleared on the next idle frame.
- Framework store values and registered shared values are published together.
  Unchanged store data no longer disappears and reappears in every frame.
- Parsed CSS and selector chains are shared within each Bevy App. Ordered source
  lists preserve cross-stylesheet root variables and stylesheet precedence.
- Selector candidates are indexed by the rightmost ID, class, or tag. Compound
  selectors, ancestor matching, pseudo states, and media checks still apply.
- CSS asset additions, modifications, and removals invalidate dependent source
  lists. Separate Apps cannot accidentally reuse each other's asset indices.
- `calc(...)` only marks a Bevy `Node` changed when a resolved property changes.
  Parent sizes, viewport sizes, hierarchy changes, and motion are still evaluated.

## Reproduce the Benchmark

Run from the repository root:

```bash
cargo bench --bench html_css --features extended-framework
```

The benchmark uses 1,000 UI nodes, 250 unrelated CSS rules, 1,000 shared values,
warm-up updates, and 30 samples per case. It reports median and p95 times for
CSS reapplication, idle `calc`, HTML parsing, idle bindings, and framework store
synchronization. Parsing is a separate startup workload.

This is a headless CPU benchmark: it excludes GPU rendering, text shaping, and
Bevy's layout pass. Its timings are not application FPS. The idle `Changed<Node>`
count detects unnecessary layout invalidation independently of machine speed.

For the small CI smoke test:

```bash
cargo bench --bench html_css --profile dev --features extended-framework -- --smoke
```

The smoke test uses 100 nodes and three samples. It asserts that idle `calc`
does not dirty layout; it has no hardware-dependent timing threshold.

## Measured Comparison

Measured on an AMD Ryzen 9 5900X, Linux x86_64, Rust 1.95.0, Bevy 0.19,
`--profile dev --features extended-framework`, 1,000 nodes, 30 samples.
Baseline: commit `5a608bc`; after: the performance changes in this working tree.
These are unoptimized diagnostic measurements, not release timings.

| Workload | Before (median) | After (median) |
| --- | ---: | ---: |
| CSS reapplication | 566.390 ms | 31.671 ms |
| Idle HTML bindings | 4.780 ms | 0.394 ms |
| HTML parsing | 64.186 ms | 65.365 ms |
| Idle calc CPU loop, excluding layout | 1.360 ms | 1.581 ms |
| Idle nodes marked changed | 1,000 | 0 |

CSS reapplication improved about 18x and idle bindings about 12x in this fixture.
HTML parsing was not optimized by this change. The calc fix prevents downstream
layout work; the headless loop alone does not show a CPU time improvement.

Final optimized run on the same machine, after concurrent builds finished:

| Workload | Median | p95 |
| --- | ---: | ---: |
| CSS reapplication | 2.185 ms | 3.410 ms |
| Idle calc, excluding layout | 0.085 ms | 0.348 ms |
| HTML parsing | 6.424 ms | 7.921 ms |
| Idle HTML bindings | 0.034 ms | 0.054 ms |
| Idle framework store synchronization | 0.380 ms | 0.835 ms |

The debug comparison above is the before/after measurement. These optimized
results describe the updated implementation only.

## Application Measurements

Scroll containers now retain their transforms and inherited visibility between
frames. Their structure-maintenance pass only changes layout fields when values
actually differ. Inherited font families ending in `.ttf` or `.otf` load that file
directly, including quoted paths; folder families still resolve weighted files.
Non-text nodes do not request font assets during inheritance.

### Mounting Game Screens

`html::builder::mount_html_fragment` mounts parsed widget nodes under an optional
ECS parent and returns the root entities. Its visitor receives each entity and
`HtmlWidgetNode`, whose `meta()` exposes CSS IDs and classes for controller binding.
Attach your stylesheet using `CssSource` in the visitor. Parse each instance
separately so generated widget IDs remain unique. Fragments are caller-owned:
despawn their roots to remove them; they do not participate in `HtmlSource` diffing.
Construct screens once, then update text/state instead of reparsing every frame.

Widget initialization runs after spawning and creates presentation components.
Attach native render targets or other presentation overrides after initialization
(when `TagName` is present). Gameplay markers and observers can be attached in the
visitor immediately. This keeps the normal HTML widgets, CSS cascade and scrolling
while allowing game controllers to retain their existing ECS ownership.

Run the local widget example with optimizations enabled:

```bash
cargo run --release --manifest-path crates/local-examples/Cargo.toml -- widget-overview
```

Compare the same scene, resolution, assets, and interaction sequence. Measure idle,
hover, resize, animation, and hot reload separately. A large number of matching
rules or genuinely changing nodes still requires work. Avoid writing unchanged
application resources each frame; use Bevy's `set_if_neq` where appropriate.

Regression tests cover unchanged layout ticks, parent resizing, transitions,
reparenting, selector indexing, cache reuse and App isolation, cross-sheet variable
reload/removal, and binding updates after idle frames.

Local validation: 298 unit/integration tests, 32 doc-tests (one existing ignored
doc-test), the CI benchmark smoke test, the no-default-features library build,
and the local examples with `extended-framework` passed. The existing CI coverage
configuration reached 91.52% regions against its 90% threshold; the new CSS cache
module reached 98.48%. The GitHub-hosted workflow still runs after pushing changes.
