---
title: Routing
---

> Erst verfügbar ab Version 1.6.0!

# How-to: UI Routing (`extended-framework`)

Dieser Guide beschreibt das Angular-ähnliche Routing-System für `extended-framework`.
Routes registrieren Komponenten unter Pfaden und rendern die aktive Route in `<router-outlet>`.

Nutze dieses System, wenn du eine stabile App-Shell mit dynamischem Seiteninhalt möchtest, statt direkt von Komponente zu Komponente zu navigieren.

## Wann Routing nutzen?

Nutze Routing, wenn du:

- eine Angular-ähnliche Route-Tabelle möchtest
- eine feste `index.html` als App-Shell verwenden willst
- Seiten-Komponenten in `<router-outlet>` rendern willst
- Navigation über HTML-Event-Handler mit `Router::navigate(...)` auslösen willst
- Redirects und Fallback-Komponenten brauchst
- Route-CSS vor dem Sichtbarwerden der Route laden willst
- wichtige Route-Komponenten mit `load!(...)` im Speicher halten willst

Routing ist nicht für den Legacy-Flow mit `UiRegistry` gedacht. Routing gehört zum `extended-framework`.

## 1) Feature aktivieren

```toml
[dependencies]
bevy_extended_ui = { version = "x.x.x", features = ["extended-framework"] }
bevy_extended_ui_macros = "x.x.x"
```

## 2) Empfohlene Dateistruktur

Eine praxistaugliche Angular-ähnliche Struktur ist:

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

Eine flache Struktur funktioniert ebenfalls:

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

Wichtige Kontrakte:

1. `index.html` ist der Framework-Einstieg.
2. `beu.routes.rs` enthält die Route-Tabelle.
3. Route-Ziele referenzieren `template_name`-Werte von Komponenten, keine Rust-Typnamen.
4. Jede `*.component.rs` Datei muss über `#[path]` in die Rust-App kompiliert werden.
5. `template_file` muss zur Component-Datei passen:
   - `help.component.rs` -> `help.component.html`

## 3) App-Shell mit `<router-outlet>`

Datei: `assets/index.html`

```html
<!DOCTYPE html>
<html lang="de">
<head>
  <meta charset="utf-8">
  <meta name="framework-demo">
  <title>Extended Framework Demo</title>
</head>
<body>
  <div id="app-shell">
    <button onclick="go_home">Home</button>
    <button onclick="go_help">Hilfe</button>
    <button onclick="go_settings">Einstellungen</button>

    <router-outlet></router-outlet>
  </div>
</body>
</html>
```

`<router-outlet>` wird durch die Komponente ersetzt, die zur aktiven Route gehört.

Wenn die aktive Route `/help` ist und `/help` auf `app-help` zeigt, verhält sich der Compiler so, als stünde im Outlet:

```html
<app-help></app-help>
```

Danach wird wie gewohnt `help.component.html` eingebettet und das Component-CSS injiziert.

## 4) Routes in `beu.routes.rs` definieren

Datei: `assets/components/beu.routes.rs`

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

### Route-Ziele

Der zweite Parameter von `.route(...)` ist der `template_name` der Komponente.

```rust
.route("/help", "app-help")
```

Dieser Wert muss zu einer Component-Definition passen:

```rust
pub const HELP_COMPONENT: HelpComponent = HelpComponent {
    template_name: "app-help",
    template_file: "help.component.html",
    styles: &["help.component.css"],
};
```

### Keep-alive Route-Ziele mit `load!(...)`

Nutze `load!("template-name")` für Route-Komponenten, die teuer zu bauen sind und nach dem Verlassen der Route bereit bleiben sollen.

`load!()` wird aus `bevy_extended_ui::routing` importiert.

```rust
use bevy_extended_ui::routing::{Routes, load};

#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/help", "app-help")
}
```

`load!("app-main")` bedeutet:

1. `app-main` wird als Keep-alive Route in das Router-Outlet gerendert.
2. Wenn eine andere Route aktiv wird, wird `app-main` mit `display: none` versteckt.
3. Die versteckte Route nimmt keinen Layout-Platz ein.
4. Wenn der User zurück zu `/` navigiert, wird der bestehende Widget-Baum wiederverwendet und nicht komplett neu gespawnt.

Nutze das für schwere oder wichtige Seiten, zum Beispiel Dashboard oder Home. Markiere nicht pauschal jede Route als Keep-alive; inaktive Keep-alive Routes bleiben im Speicher.

#### Normale Route vs Keep-alive Route

Normale Route:

```rust
.route("/help", "app-help")
```

Verhalten:

1. Die Route-Komponente wird gerendert, wenn `/help` aktiv ist.
2. Wenn eine andere Route aktiv wird, kann die Komponente aus dem aktiven Outlet-Baum entfernt werden.
3. Beim Zurücknavigieren kann der Komponentenbaum neu aufgebaut werden.

Keep-alive Route:

```rust
.route("/", load!("app-main"))
```

Verhalten:

1. Die Route-Komponente wird in einen stabilen Keep-alive Wrapper gerendert.
2. Wenn eine andere Route aktiv wird, bleibt der Wrapper im Outlet und wird mit `display: none` versteckt.
3. Der versteckte Wrapper beeinflusst das Layout nicht.
4. Beim Zurücknavigieren wird der Wrapper wieder auf `display: flex` gesetzt.

#### Mehrere Keep-alive Routes

Du kannst mehr als eine schwere Route im Speicher halten.

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

Das ist sinnvoll, wenn zwei Seiten teuer sind und häufig gewechselt werden.

Halte die Liste klein. Jede Keep-alive Route behält ihren Widget-Baum, zustandsbehaftete Widgets und geladene Styles im Speicher.

#### Keep-alive mit mehreren Route-Dateien

`load!()` funktioniert in jeder Route-Datei, die über `#[beu_routes]` gesammelt wird.

Datei: `assets/components/beu.routes.rs`

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

Datei: `assets/components/secondary.routes.rs`

```rust
use bevy_extended_ui::routing::{Routes, load};
use bevy_extended_ui_macros::beu_routes;

#[beu_routes]
pub fn secondary_routes() -> Routes {
    Routes::new()
        .route("/settings", "app-settings")
        .route("/info", load!("app-infopage"))
}
```

Beide Route-Tabellen werden über Inventory gesammelt und vom Routing-Plugin zusammengeführt.

#### Wie das Outlet intern aussieht

Bei normalen Routes wird das Outlet als einzelnes Component-Tag kompiliert:

```html
<app-help></app-help>
```

Sobald mindestens eine Route `load!()` nutzt, werden Keep-alive Routes in Route-Wrapper kompiliert.

Beispiel während `/help` aktiv ist und `/` Keep-alive ist:

```html
<div class="beu-route beu-route-cached" style="display: none;">
  <app-main></app-main>
</div>
<div class="beu-route beu-route-active" style="display: flex; width: 100%; height: 100%; flex-direction: column;">
  <app-help></app-help>
</div>
```

Nach der Component-Kompilierung werden die Component-Tags durch ihr Component-HTML ersetzt.

#### Wann `load!()` nutzen?

Gute Kandidaten:

1. Home-Seiten mit vielen Widgets.
2. Dashboards mit großen Forms, Images, Dialogen oder datengetriebenen Listen.
3. Seiten, zu denen User häufig zurückwechseln.
4. Seiten, bei denen die erste Navigation langsamer sein darf, aber wiederholte Navigation sofort sein muss.

Vermeide `load!()` für:

1. Kleine Seiten wie eine Help-Seite mit einer Headline und einem Button.
2. Selten besuchte Seiten.
3. Seiten, deren Widget-Baum bei jedem Öffnen bewusst zurückgesetzt werden soll.
4. Sehr viele Routes, weil sie alle im Speicher bleiben.

#### Häufige Fehler

Fehlender Import:

```rust
use bevy_extended_ui::routing::{Routes, load};
```

Falsches Ziel:

```rust
.route("/", load!("MainComponent")) // Falsch: Rust-Typname
```

Korrektes Ziel:

```rust
.route("/", load!("app-main")) // Richtig: Component template_name
```

`load!()` als Pfad verwenden:

```rust
.route(load!("/"), "app-main") // Falsch
```

`load!()` wrapped nur das Component-Ziel, nicht den Pfad.

### Redirects

Redirects normalisieren einen Pfad auf einen anderen Pfad, bevor die Komponente aufgelöst wird.

```rust
.redirect("", "/")
.redirect("/home", "/")
```

Das ist nützlich für leere Pfade, Aliase oder alte Menüpfade.

### Fallback-Komponente

Fallback wird verwendet, wenn keine Route passt.

```rust
.fallback("app-not-found")
```

Der Fallback-Wert ist ebenfalls ein Component-`template_name`.

Beispiel:

```rust
#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", "app-main")
        .route("/help", "app-help")
        .fallback("app-not-found")
}
```

## 5) Route- und Component-Dateien in den Rust-Build einbinden

Rust lädt Dateinamen wie `beu.routes.rs` oder `main.component.rs` nicht automatisch als normale Module.
Binde sie über `#[path]` in einer normalen Rust-Datei ein.

Datei: `src/assets_components.rs`

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

Dann `assets_components` in `main.rs` einbinden:

```rust
#[cfg(feature = "extended-framework")]
mod assets_components;
```

Ohne diese Einbindung werden `#[beu_routes]`, `#[ui_component]` und `#[html_fn]` aus den Dateien nicht kompiliert/registriert.

## 6) Component-Beispiel: Home-Route

Datei: `assets/components/main.component.rs`

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

Datei: `assets/components/main.component.html`

```html
<div class="page home-page">
  <h1>Home</h1>
  <p>Diese Route wird für <code>/</code> gerendert.</p>
  <button onclick="go_help">Hilfe öffnen</button>
</div>
```

Datei: `assets/components/main.component.css`

```css
.home-page {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 24px;
}
```

## 7) Component-Beispiel: Help-Route

Datei: `assets/components/help/help.component.rs`

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

Datei: `assets/components/help/help.component.html`

```html
<div class="page help-page">
  <button onclick="go_home">Home</button>
  <h1>Hilfe</h1>
  <p>Diese Seite erklärt die Anwendung.</p>
</div>
```

Datei: `assets/components/help/help.component.css`

```css
.help-page {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 24px;
  border-radius: 12px;
}
```

## 8) Navigation aus Rust-Systemen

Jedes System kann navigieren, wenn es `ResMut<Router>` erhält.

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

System normal registrieren:

```rust
app.add_systems(Update, keyboard_navigation);
```

## 9) Navigation aus HTML-Handlern

Das typische Muster ist ein Button mit `onclick` und ein `#[html_fn]` Handler.

HTML:

```html
<button onclick="go_settings">Einstellungen</button>
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

Der Handler kann in jedem kompilierten Rust-Modul liegen. Er muss nicht in der Ziel-Komponente liegen. In der Praxis ist es aber oft übersichtlicher, Navigationshandler nahe bei der Komponente zu halten, die den Button besitzt.

## 10) App-Konfiguration

Eine vollständige lokale Konfiguration sieht typischerweise so aus:

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

## 11) Verschachtelte Component-Ordner

Komponenten können in Unterordnern organisiert werden.

```text
assets/
  components/
    pages/
      dashboard/
        dashboard.component.rs
        dashboard.component.html
        dashboard.component.css
```

Component-Definition:

```rust
pub const DASHBOARD_COMPONENT: DashboardComponent = DashboardComponent {
    template_name: "app-dashboard",
    template_file: "dashboard.component.html",
    styles: &["dashboard.component.css"],
};
```

Route-Tabelle:

```rust
#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/dashboard", "app-dashboard")
        .fallback("app-dashboard")
}
```

Build-Einbindung:

```rust
#[path = "../assets/components/pages/dashboard/dashboard.component.rs"]
mod dashboard_component_mod;
```

Der Compiler sucht passende HTML/CSS-Dateien zuerst relativ zum Component-Quellordner.

## 12) CSS-Verhalten und Flicker-Schutz

Wenn `index.html` ein `<router-outlet>` enthält, werden Styles aller registrierten Route-Komponenten in die Shell injiziert.
Dadurch wird Route-CSS früh erkannt und kann vor der Navigation geladen werden.


Während eines Route-Wechsels bleibt die neu aufgebaute Route verborgen, bis:

1. die referenzierten CSS-Assets geladen sind
2. CSS in `UiStyle` zusammengeführt wurde
3. ein aktiver Style berechnet wurde

Das verhindert einen sichtbaren Zwischenzustand, in dem alle Elemente ungestylt in einer Reihe erscheinen.

Sehr große Route-CSS-Dateien solltest du trotzdem vermeiden, wenn Navigation sofort wirken soll.

## 13) Mehrere Route-Dateien

Du kannst Routes nach Features in mehrere `#[beu_routes]` Dateien aufteilen. Das Routing-Plugin sammelt alle über Inventory.

Datei: `assets/components/admin.routes.rs`

```rust
use bevy_extended_ui::routing::Routes;
use bevy_extended_ui_macros::beu_routes;

#[beu_routes]
pub fn admin_routes() -> Routes {
    Routes::new()
        .route("/admin", "app-admin")
        .route("/admin/users", "app-admin-users")
}
```

Datei: `assets/components/beu.routes.rs`

```rust
use bevy_extended_ui::routing::{Routes, load};
use bevy_extended_ui_macros::beu_routes;

#[beu_routes]
pub fn routes() -> Routes {
    Routes::new()
        .route("/", load!("app-main"))
        .route("/help", "app-help")
        .fallback("app-main")
}
```

Jede Route-Datei muss über `#[path]` aus deiner `assets_components.rs` in die Rust-App kompiliert werden.

`load!()` kann in jeder dieser Route-Dateien verwendet werden. Das Keep-alive Flag wird auf der Route gespeichert und bleibt beim Zusammenführen der Route-Tabellen erhalten.

## 14) Mehrere `#[beu_routes]` Registrierungen

Das Routing-Plugin sammelt alle `#[beu_routes]` Funktionen über Inventory.

Das funktioniert:

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

Regeln:

1. Routes mit gleichem Pfad überschreiben früher gesammelte Routes.
2. Redirects werden angehängt.
3. Der zuletzt gesammelte Fallback gewinnt.

Da Inventory-Reihenfolge nicht als App-Logik verwendet werden sollte, ist eine finale `beu.routes.rs` Route-Tabelle besser, wenn Konflikte relevant sind.

## 15) Aktuelle Route lesen

Du kannst die aktuelle Route aus jedem System lesen.

```rust
use bevy::prelude::*;
use bevy_extended_ui::routing::Router;

fn log_current_route(router: Res<Router>) {
    info!("Aktuelle UI-Route: {}", router.current_path());
}
```

Die aufgelöste Komponente kannst du ebenfalls auslesen:

```rust
fn log_active_component(router: Res<Router>) {
    if let Some(component) = router.active_component() {
        info!("Aktive Komponente: {}", component);
    }
}
```

## 16) Normalisierung von Route-Pfaden

Route-Pfade werden normalisiert:

- `""` wird zu `"/"`
- `"help"` wird zu `"/help"`
- `"/help/"` wird zu `"/help"`
- Backslashes werden zu `/`

Diese Navigationen sind gleichwertig:

```rust
router.navigate("help");
router.navigate("/help");
router.navigate("/help/");
```

Für Lesbarkeit solltest du Routes trotzdem mit führendem `/` schreiben.

## 17) Typische Fehlerquellen

1. `<router-outlet>` fehlt in `index.html`.
   Der Router kann den Pfad ändern, aber es wird keine Route-Komponente gerendert.

2. `beu.routes.rs` wurde nicht über `#[path]` eingebunden.
   Die Route-Tabelle wird nicht kompiliert und der Router hat keine konfigurierten Routes.

3. Eine Route zeigt auf einen nicht existierenden `template_name`.
   Der Framework-Compiler schlägt fehl, weil die Route-Komponente nicht registriert ist.

4. Die Component-Rust-Datei ist eingebunden, aber HTML/CSS-Dateien fehlen.
   Die Component-Asset-Validierung schlägt fehl.

5. `template_file` passt nicht zum Component-Dateinamen.
   Beispiel: `help.component.rs` muss `template_file: "help.component.html"` nutzen.

6. Handlername passt nicht.
   HTML `onclick="go_home"` braucht Rust `#[html_fn("go_home")]`.

7. Route-CSS erscheint spät.
   Stelle sicher, dass Route-Komponenten in `beu.routes.rs` registriert sind; Route-CSS wird aus der Route-Tabelle vorgeladen.

## 18) Minimales komplettes Beispiel

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
<html lang="de">
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
