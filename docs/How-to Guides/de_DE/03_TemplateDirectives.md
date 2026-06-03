---
title: Template-Direktiven
---

# How-to: Template-Direktiven (`@use`, `@if`, `@else`, `@for`)

Dieser Guide beschreibt die Template-Direktiven, die vor dem HTML-Parsing ausgewertet werden.
Damit kannst du Shared Rust-Daten in Templates importieren, Bedingungen auswerten und Listen rendern.

## Wann Template-Direktiven nutzen?

Nutze Template-Direktiven, wenn du:

- UI-Blöcke abhängig von Daten ein- oder ausblenden willst
- Listen aus JSON-/Rust-Daten rendern willst
- `#[html_shared]`- oder `#[html_use]`-Werte in HTML verwenden willst
- Imports im Template explizit und lesbar halten willst

Wichtig: Direktiven laufen vor dem DOM-Parsing. Sie erzeugen normales HTML, das danach vom Converter verarbeitet wird.

## 1) Daten für Templates bereitstellen

Einfache Laufzeitvariablen kannst du über `UiLangVariables` setzen. Werte werden als JSON geparst, wenn möglich.

```rust
use bevy::prelude::*;
use bevy_extended_ui::UiLangVariables;

fn setup_vars(mut vars: ResMut<UiLangVariables>) {
    vars.set("enabled", "true");
    vars.set("data", r#"{"users":[{"name":"Ada"},{"name":"Bob"}]}"#);
}
```

Für typisierte Rust-Daten markierst du Ressourcen mit `#[html_shared]` und `Serialize`.

```rust
use bevy::prelude::*;
use bevy_extended_ui_macros::html_shared;
use serde::Serialize;

#[html_shared]
#[derive(Resource, Serialize)]
pub struct DataPack {
    pub version: String,
    pub used: bool,
}

#[html_shared]
#[derive(Resource, Serialize)]
pub enum DataState {
    Active,
    Inactive,
}
```

Wenn die Ressource in der Bevy-World existiert, wird sie für Templates verfügbar.

## 2) `@use` verwenden

`@use` importiert Shared Values in den Template-Kontext.
Jede `@use`-Zeile braucht ein Semikolon.

### Einzelnen Typ importieren

```html
@use "crate::data_structs::DataPack";

<p>Version: {{ data_pack.version }}</p>
```

Ohne Alias wird aus dem Typnamen automatisch ein Template-Alias gebildet:

- `DataPack` -> `data_pack`
- `DataState` -> `data_state`
- `PlayerProfile` -> `player_profile`

### Alias vergeben

```html
@use "crate::data_structs::DataState" as state;

@if(state == DataState::Inactive) {
  <p>Inactive</p>
}
```

### Mehrere Typen gruppiert importieren

```html
@use "crate::data_structs::{DataState, DataPack}";

<p>{{ data_pack.version }}</p>
<p>{{ data_state }}</p>
```

### Alias innerhalb einer Gruppe vergeben

```html
@use "crate::data_structs::{DataState as current_state, DataPack}";

@if(current_state == DataState::Inactive) {
  <p>Inactive</p>
}
```

### Alle Shared Types aus einem Pfad importieren

```html
@use "crate::data_structs::*";

<p>{{ data_pack.version }}</p>
<p>{{ data_state }}</p>
```

Bei `path::*` wird jeder passende Typ mit seinem Default-Alias importiert.
`as` wird in dieser Form nicht verwendet.

Nicht verwenden:

```html
@use "crate::data_structs::*" as data;
```

### Felder eines Struct-Werts direkt importieren

`as *` ist etwas anderes als `path::*`: Es importiert Felder eines einzelnen Struct-/Objektwerts direkt in den Kontext.

```html
@use "Player" as *;

<p>{{ name }}</p>
@if(state) {
  <p>Online</p>
}
```

Nutze `as *` sparsam, weil Feldnamen leicht mit anderen Variablen kollidieren können.

## 3) `@if` und `@else` verwenden

`@if` rendert den Block nur, wenn die Bedingung wahr ist.
`@else` ist optional.

```html
@if(enabled) {
  <button>Start</button>
} @else {
  <p>Disabled</p>
}
```

Du kannst Objektpfade, Negation, Vergleiche und logische Operatoren verwenden.

```html
@use "crate::data_structs::*";

@if(!data_pack.used && data_state == DataState::Inactive) {
  <p>Ready</p>
} @else {
  <p>Already used or active</p>
}
```

Unterstützte Ausdrücke:

- boolesche Werte: `enabled`, `!enabled`
- Objektpfade: `data.user.name`, `data_pack.version`
- Gleichheit: `state == "active"`, `data_state == DataState::Inactive`
- Logik: `&&`, `||`
- Klammern: `(a || b) && c`
- Methoden auf Strings/Listen: `startsWidth`, `startsWith`, `endsWidth`, `endsWith`, `contains`

Beispiel:

```html
@if(data.username.startsWith("Net") && enabled) {
  <p>Allowed</p>
}
```

## 4) `@for` verwenden

`@for` rendert einen Block für jedes Element einer Liste.

```html
@for(user in data.users) {
  <p>{{ user.name }}</p>
}
```

Mit Index:

```html
@for(user, index in data.users) {
  <p>{{ index }}: {{ user.name }}</p>
}
```

`@for` kann mit `@if` kombiniert werden.

```html
@for(user, index in data.users) {
  @if(user.active || index == 0) {
    <p>{{ index }} - {{ user.name }}</p>
  }
}
```

Getter-Methoden ohne Argumente funktionieren für serialisierte Felder, wenn sie dem Schema `get_<feldname>` folgen.

```html
@use "crate::data_structs::DataPack";

@for(entry, index in data_pack.get_data()) {
  <p>Byte {{ index }}: {{ entry }}</p>
}
```

## 5) Komplettes Beispiel

```html
@use "crate::data_structs::*";

<div id="main">
  @if(data_state == DataState::Inactive && !data_pack.used) {
    <h3>Data Pack {{ data_pack.version }}</h3>
  } @else {
    <h3>Data unavailable</h3>
  }

  @for(entry, index in data_pack.get_data()) {
    <p>Byte {{ index }}: {{ entry }}</p>
  }
</div>
```

## 6) Typische Stolperfallen

1. Jede `@use`-Zeile braucht ein Semikolon.
2. `@use "path::*"` importiert Typen mit Default-Alias; `as` ist dort nicht sinnvoll.
3. `@use "Type" as *` importiert Felder eines einzelnen Objektwerts, nicht alle Typen eines Moduls.
4. Aliasnamen müssen im Template verwendet werden, z. B. `data_state`, nicht `DataState`.
5. Enum-Varianten werden als Literale verglichen, z. B. `DataState::Inactive`.
6. Direktiven müssen gültiges HTML erzeugen, nachdem sie ausgewertet wurden.
