---
title: Routing
---

> Supported by version `1.6.0` and above.

# How-to: UI Routing (`extended-framework`)

This guide explains the Angular-like routing system for `extended-framework`.
Routes let you register components under paths and render the active route inside `<router-outlet>`.

Use this when you want one stable app shell with dynamic page content, instead of navigating directly from component to component.

## When should you use routing?

Use routing when you want:

- an Angular-like route table
- one fixed `index.html` app shell
- page components rendered into `<router-outlet>`
- navigation from HTML event handlers with `Router::navigate(...)`
- redirects and fallback components
- route component CSS preloaded before the route becomes visible
- important route components kept alive with `load!(...)`

Do not use routing with the legacy `UiRegistry` flow. Routing is designed for `extended-framework`.

## 1) Enable the feature

```toml
[dependencies]
bevy_extended_ui = { version = "x.x.x", features = ["extended-framework"] }
bevy_extended_ui_macros = "x.x.x"
```

## 2) Recommended file structure

A practical Angular-like structure is:

```text
assets/
  index.html
  components/
    beu.routes.rs
    main.component.rs
    main.component.html
    main.component.css
    help/
      help.component.rs
      help.component.html
      help.component.css
    settings/
      settings.component.rs
      settings.component.html
      settings.component.css
src/
  assets_components.rs
  main.rs
```

A flatter structure also works:

```text
assets/
  index.html
  components/
    beu.routes.rs
    main.component.rs
    main.component.html
    main.component.css
    help.component.rs
    help.component.html
    help.component.css
```

The important contracts are:

1. `index.html` is the framework entrypoint.
2. `beu.routes.rs` contains the route table.
3. Route targets reference component `template_name` values, not Rust type names.
4. Every `*.component.rs` file must be compiled into your Rust app through `#[path]`.
5. `template_file` must match the component filename convention:
   - `help.component.rs` -> `help.component.html`

## 3) App shell with `<router-outlet>`

File: `assets/index.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="framework-demo">
  <title>Extended Framework Demo</title>
</head>
<body>
  <div id="app-shell">
    <button onclick="go_home">Home</button>
    <button onclick="go_help">Help</button>
    <button onclick="go_settings">Settings</button>

    <router-outlet></router-outlet>
  </div>
</body>
</html>
```

`<router-outlet>` is replaced with the component configured for the active route.

If the active route is `/help` and `/help` points to `app-help`, the compiler behaves as if the outlet contained:

```html
<app-help></app-help>
```

Then the normal component compiler inlines `help.component.html` and injects the component styles.

## 4) Define routes in `beu.routes.rs`

File: `assets/components/beu.routes.rs`

```rust
use bevy_extended_ui::routing::{Routes, load};
use bevy_extended_ui_macros::beu_routes;

#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/help", "app-help")
        .route("/settings", "app-settings")
        .redirect("", "/")
        .fallback("app-main")
}
```

### Route targets

The second argument to `.route(...)` is the component `template_name`.

```rust
.route("/help", "app-help")
```

This must match a component definition:

```rust
pub const HELP_COMPONENT: HelpComponent = HelpComponent {
    template_name: "app-help",
    template_file: "help.component.html",
    styles: &["help.component.css"],
};
```

### Keep-alive route targets with `load!(...)`

Use `load!("template-name")` for route components that are expensive to build and should stay ready after the user leaves the route.

`load!()` is imported from `bevy_extended_ui::routing`.

```rust
use bevy_extended_ui::routing::{Routes, load};

#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/help", "app-help")
}
```

`load!("app-main")` means:

1. `app-main` is rendered into the router outlet as a keep-alive route.
2. When another route becomes active, `app-main` is hidden with `display: none`.
3. The hidden route does not consume layout space.
4. When the user navigates back to `/`, the existing widget tree is reused instead of being spawned from scratch.

Use this for heavy or important pages, for example a dashboard or home page. Do not mark every route as keep-alive by default; inactive keep-alive routes remain in memory.

#### Normal route vs keep-alive route

Normal route:

```rust
.route("/help", "app-help")
```

Behavior:

1. The route component is rendered when `/help` is active.
2. When another route becomes active, the component can be removed from the active outlet tree.
3. Navigating back may rebuild the component tree.

Keep-alive route:

```rust
.route("/", load!("app-main"))
```

Behavior:

1. The route component is rendered into a stable keep-alive wrapper.
2. When another route becomes active, the wrapper stays in the outlet but is hidden with `display: none`.
3. The hidden wrapper does not affect layout.
4. Navigating back changes the wrapper back to `display: flex`.

#### Multiple keep-alive routes

You can keep more than one heavy route alive.

```rust
use bevy_extended_ui::routing::{Routes, load};
use bevy_extended_ui_macros::beu_routes;

#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/dashboard", load!("app-dashboard"))
        .route("/help", "app-help")
        .fallback("app-main")
}
```

This is useful when two pages are expensive and commonly switched between.

Keep the list small. Every keep-alive route keeps its widget tree, stateful widgets, and loaded styles around.

#### Keep-alive with multiple route files

`load!()` works in route helper files that are merged into your main `beu.routes.rs`.

File: `assets/components/beu.routes.rs`

```rust
use bevy_extended_ui::routing::{Routes, load};
use bevy_extended_ui_macros::beu_routes;

#[path = "secondary.routes.rs"]
mod secondary_routes;

#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/help", "app-help")
        .merge(secondary_routes::secondary_routes)
        .redirect("", "/")
        .fallback("app-main")
}
```

File: `assets/components/secondary.routes.rs`

```rust
use bevy_extended_ui::routing::{Routes, load};

pub fn secondary_routes() -> Routes {
    Routes::new()
        .route("/settings", "app-settings")
        .route("/info", load!("app-infopage"))
}
```

Only the main `routes()` function needs `#[beu_routes]` in this composition style. The secondary file is a plain Rust helper that returns `Routes`.

#### What the outlet looks like internally

For normal routes, the outlet is compiled as a single component tag:

```html
<app-help></app-help>
```

When at least one route uses `load!()`, keep-alive routes are compiled into route wrappers.

Example while `/help` is active and `/` is keep-alive:

```html
<div class="beu-route beu-route-cached" style="display: none;">
  <app-main></app-main>
</div>
<div class="beu-route beu-route-active" style="display: flex; width: 100%; height: 100%; flex-direction: column;">
  <app-help></app-help>
</div>
```

After component compilation, the component tags are replaced with their component HTML.

#### When to use `load!()`

Good candidates:

1. Home pages with many widgets.
2. Dashboards with large forms, images, dialogs, or data-driven lists.
3. Pages that users frequently return to.
4. Pages where first navigation can be slower, but repeated navigation must be instant.

Avoid `load!()` for:

1. Small pages like a help page with a headline and one button.
2. Rarely visited pages.
3. Pages that should reset their widget tree every time they are opened.
4. Large numbers of routes, because they all stay in memory.

#### Common mistakes

Missing import:

```rust
use bevy_extended_ui::routing::{Routes, load};
```

Wrong target:

```rust
.route("/", load!("MainComponent")) // Wrong: Rust type name
```

Correct target:

```rust
.route("/", load!("app-main")) // Correct: component template_name
```

Using `load!()` as a path:

```rust
.route(load!("/"), "app-main") // Wrong
```

`load!()` only wraps the component target, not the path.

### Redirects

Redirects normalize one path into another before resolving the component.

```rust
.redirect("", "/")
.redirect("/home", "/")
```

This is useful for empty paths, aliases, or old menu paths.

### Fallback component

Fallback is used when no registered route matches.

```rust
.fallback("app-not-found")
```

The fallback value is also a component `template_name`.

Example:

```rust
#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", "app-main")
        .route("/help", "app-help")
        .fallback("app-not-found")
}
```

## 5) Include route and component files in the Rust build

Rust cannot load file names like `beu.routes.rs` or `main.component.rs` as regular module names automatically.
Use `#[path]` in a normal Rust file.

File: `src/assets_components.rs`

```rust
#[cfg(feature = "extended-framework")]
use bevy_extended_ui_macros::beu_registry;

#[cfg(feature = "extended-framework")]
#[beu_registry]
mod beu_registry_marker {}

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/beu.routes.rs"]
mod beu_routes;

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/main.component.rs"]
mod main_component_mod;

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/help/help.component.rs"]
mod help_component_mod;

#[cfg(feature = "extended-framework")]
#[allow(dead_code)]
#[path = "../assets/components/settings/settings.component.rs"]
mod settings_component_mod;
```

Then include `assets_components` from `main.rs`:

```rust
#[cfg(feature = "extended-framework")]
mod assets_components;
```

Without this, `#[beu_routes]`, `#[ui_component]`, and `#[html_fn]` registrations inside those files will not be compiled.

## 6) Component example: Home route

File: `assets/components/main.component.rs`

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui::routing::Router;
use bevy_extended_ui_macros::{html_fn, ui_component};

#[ui_component]
pub struct MainComponent {
    pub template_name: &'static str,
    pub template_file: &'static str,
    pub styles: &'static [&'static str],
}

pub const MAIN_COMPONENT: MainComponent = MainComponent {
    template_name: "app-main",
    template_file: "main.component.html",
    styles: &["main.component.css"],
};

#[html_fn("go_help")]
pub fn go_help(In(_): In<HtmlClick>, mut router: ResMut<Router>) {
    router.navigate("/help");
}
```

File: `assets/components/main.component.html`

```html
<div class="page home-page">
  <h1>Home</h1>
  <p>This is the route rendered for <code>/</code>.</p>
  <button onclick="go_help">Open Help</button>
</div>
```

File: `assets/components/main.component.css`

```css
.home-page {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 24px;
}
```

## 7) Component example: Help route

File: `assets/components/help/help.component.rs`

```rust
use bevy::prelude::{In, ResMut};
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui::routing::Router;
use bevy_extended_ui_macros::{html_fn, ui_component};

#[ui_component]
pub struct HelpComponent {
    pub template_name: &'static str,
    pub template_file: &'static str,
    pub styles: &'static [&'static str],
}

pub const HELP_COMPONENT: HelpComponent = HelpComponent {
    template_name: "app-help",
    template_file: "help.component.html",
    styles: &["help.component.css"],
};

#[html_fn("go_home")]
pub fn go_home(In(_): In<HtmlClick>, mut router: ResMut<Router>) {
    router.navigate("/");
}
```

File: `assets/components/help/help.component.html`

```html
<div class="page help-page">
  <button onclick="go_home">Home</button>
  <h1>Help</h1>
  <p>This page explains how the application works.</p>
</div>
```

File: `assets/components/help/help.component.css`

```css
.help-page {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 24px;
  border-radius: 12px;
}
```

## 8) Navigating from Rust systems

Any system can navigate if it receives `ResMut<Router>`.

```rust
use bevy::prelude::*;
use bevy_extended_ui::routing::Router;

fn keyboard_navigation(input: Res<ButtonInput<KeyCode>>, mut router: ResMut<Router>) {
    if input.just_pressed(KeyCode::F1) {
        router.navigate("/");
    }

    if input.just_pressed(KeyCode::F2) {
        router.navigate("/help");
    }
}
```

Add the system normally:

```rust
app.add_systems(Update, keyboard_navigation);
```

## 9) Navigating from HTML handlers

The common pattern is a button with `onclick` and an `#[html_fn]` handler.

HTML:

```html
<button onclick="go_settings">Settings</button>
```

Rust:

```rust
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlClick;
use bevy_extended_ui::routing::Router;
use bevy_extended_ui_macros::html_fn;

#[html_fn("go_settings")]
pub fn go_settings(In(_): In<HtmlClick>, mut router: ResMut<Router>) {
    router.navigate("/settings");
}
```

The handler can live in any compiled Rust module. It does not have to be in the target component, but keeping navigation handlers near the component that owns the button is usually easier to maintain.

## 10) App setup

A complete local setup usually looks like this:

```rust
#[cfg(feature = "extended-framework")]
mod assets_components;

use bevy::asset::{AssetMetaCheck, AssetPlugin};
use bevy::prelude::*;
use bevy_extended_ui::framework::ExtendedFrameworkConfiguration;
use bevy_extended_ui::{ExtendedCam, ExtendedUiConfiguration, ExtendedUiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(ExtendedUiPlugin)
        .add_systems(Startup, configure_ui)
        .run();
}

fn configure_ui(
    mut ui_config: ResMut<ExtendedUiConfiguration>,
    mut framework_config: ResMut<ExtendedFrameworkConfiguration>,
) {
    ui_config.camera = ExtendedCam::Default;
    ui_config.framework_components_path = "components".to_string();

    framework_config.asset_root_fs_path = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    framework_config.assets_component_root = "components".to_string();
    framework_config.index_html_file = "index.html".to_string();
}
```

## 11) Nested component folders

Components can be organized in subfolders.

```text
assets/
  components/
    pages/
      dashboard/
        dashboard.component.rs
        dashboard.component.html
        dashboard.component.css
```

Component definition:

```rust
pub const DASHBOARD_COMPONENT: DashboardComponent = DashboardComponent {
    template_name: "app-dashboard",
    template_file: "dashboard.component.html",
    styles: &["dashboard.component.css"],
};
```

Route table:

```rust
#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/dashboard", "app-dashboard")
        .fallback("app-dashboard")
}
```

Build include:

```rust
#[path = "../assets/components/pages/dashboard/dashboard.component.rs"]
mod dashboard_component_mod;
```

The compiler resolves the matching HTML/CSS files relative to the component's source folder first.

## 12) CSS behavior and flicker protection

When `index.html` contains `<router-outlet>`, styles for all registered route components are injected into the shell.
That means route CSS is discovered early and can be loaded before navigation.


During a route change, the rebuilt route stays hidden until:

1. the referenced CSS assets are loaded
2. CSS has been merged into `UiStyle`
3. an active style has been calculated

This prevents a visible one-frame layout where all elements appear unstyled in a row.

You should still avoid huge route CSS files if you need instant navigation.

## 13) Multiple route files

You can split routes by feature into helper route files and merge them in your main `beu.routes.rs`.

This is the recommended structure when one route table should be the explicit entry point.

File: `assets/components/admin.routes.rs`

```rust
use bevy_extended_ui::routing::Routes;

pub fn admin_routes() -> Routes {
    Routes::new()
        .route("/admin", "app-admin")
        .route("/admin/users", "app-admin-users")
}
```

File: `assets/components/beu.routes.rs`

```rust
use bevy_extended_ui::routing::{Routes, load};
use bevy_extended_ui_macros::beu_routes;

#[path = "admin.routes.rs"]
mod admin_routes;

#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/help", "app-help")
        .merge(admin_routes::admin_routes)
        .fallback("app-main")
}
```

Only `beu.routes.rs` has to be included from your `assets_components.rs`. The helper route file is included by `beu.routes.rs`.

`load!()` can be used in helper route files too. The keep-alive flag is stored on the route and survives `.merge(...)`.

Do not put `#[beu_routes]` on helper functions that are also merged manually. Otherwise the same route table is both merged manually and collected by inventory.

## 14) Multiple `#[beu_routes]` registrations

The routing plugin collects all `#[beu_routes]` functions through inventory.

This works:

```rust
#[beu_routes]
pub fn public_routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/help", "app-help")
}

#[beu_routes]
pub fn admin_routes() -> Routes {
    Routes::new()
        .route("/admin", "app-admin")
        .fallback("app-main")
}
```

Rules:

1. Routes with the same path override earlier collected routes.
2. Redirects are appended.
3. The last collected fallback wins.

Because inventory order should not be used for application logic, prefer one final `beu.routes.rs` route table when route conflicts matter.

## 15) Reading the current route

You can inspect the current route from any system.

```rust
use bevy::prelude::*;
use bevy_extended_ui::routing::Router;

fn log_current_route(router: Res<Router>) {
    info!("Current UI route: {}", router.current_path());
}
```

You can also inspect the resolved component:

```rust
fn log_active_component(router: Res<Router>) {
    if let Some(component) = router.active_component() {
        info!("Active component: {}", component);
    }
}
```

## 16) Route path normalization

Route paths are normalized:

- `""` becomes `"/"`
- `"help"` becomes `"/help"`
- `"/help/"` becomes `"/help"`
- backslashes are converted to `/`

These are equivalent for navigation:

```rust
router.navigate("help");
router.navigate("/help");
router.navigate("/help/");
```

Prefer writing routes with a leading `/` for readability.

## 17) Common failures

1. `<router-outlet>` is missing from `index.html`.
   The router can still change path, but no route component is rendered.

2. `beu.routes.rs` is not included via `#[path]`.
   The route table is not compiled and the router has no configured routes.

3. A route points to a non-existing `template_name`.
   The framework compiler fails because the route component is not registered.

4. The component Rust file is included, but its HTML/CSS files are missing.
   Component asset validation fails.

5. `template_file` does not match the component filename.
   Example: `help.component.rs` must use `template_file: "help.component.html"`.

6. Handler name mismatch.
   HTML `onclick="go_home"` requires Rust `#[html_fn("go_home")]`.

7. Route CSS appears late.
   Ensure route components are registered in `beu.routes.rs`; route component CSS is preloaded from the route table.

## 18) Minimal complete example

```text
assets/
  index.html
  components/
    beu.routes.rs
    main.component.rs
    main.component.html
    main.component.css
    help.component.rs
    help.component.html
    help.component.css
src/
  assets_components.rs
  main.rs
```

`assets/index.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="app">
</head>
<body>
  <router-outlet></router-outlet>
</body>
</html>
```

`assets/components/beu.routes.rs`:

```rust
use bevy_extended_ui::routing::{Routes, load};
use bevy_extended_ui_macros::beu_routes;

#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/help", "app-help")
        .redirect("", "/")
        .fallback("app-main")
}
```

`assets/components/help.component.html`:

```html
<button onclick="go_home">Home</button>
<h4>Hello Help</h4>
```

`src/assets_components.rs`:

```rust
#[path = "../assets/components/beu.routes.rs"]
mod beu_routes;

#[path = "../assets/components/main.component.rs"]
mod main_component_mod;

#[path = "../assets/components/help.component.rs"]
mod help_component_mod;
```
