---
title: Shortland Functions
---

# How-to: Shortland Functions (`$set`, `$add`, `$min`, `$event`)

Shortland Functions sind Inline-Aktionen direkt in HTML-Event-Attributen.
Nutze sie für kleine State-Updates, bei denen kein eigenes Rust `#[html_fn]` nötig ist.

```html
<input value="{{ player.name }}" onchange="$set(player.name, $event.value)">
<button onclick="$add(info.value, 1)">+</button>
<button onclick="$min(info.value, 1)">-</button>
```

## Wann verwenden?

Nutze Shortland Functions für:

- Input-Werte direkt in ein `BeuStore`-Feld schreiben
- numerische Werte erhöhen oder verringern
- einfachen Widget-State in reaktive Template-Daten kopieren
- triviale Handler wie `set_name`, `set_value` oder `increase_value` ersetzen

Nutze weiterhin `#[html_fn]` für:

- Routing (`router.navigate(...)`)
- Entities spawnen/despawnen
- mehrere Ressourcen mit eigener Logik aktualisieren
- Validierung oder Side Effects
- komplexe Umwandlungen wie Date-Range-Filter oder FieldSet-Mapping

## Voraussetzungen

Shortland Functions arbeiten reaktiv über den Extended-Framework-Store.
Das Ziel sollte auf einen `#[derive(BeuStore)]` Wert oder einen Wert im `UiBindingStore` zeigen.

```rust
use bevy_extended_ui::BeuStore;
use serde::Serialize;

#[derive(BeuStore, Clone, PartialEq, Serialize)]
pub struct Player {
    pub name: String,
    pub state: bool,
}

#[derive(BeuStore, Clone, PartialEq, Serialize)]
pub struct Info {
    pub value: f32,
    pub color: String,
}
```

Der Template-Alias wird aus dem Rust-Typnamen in snake_case gebildet:

- `Player` -> `player`
- `Info` -> `info`
- `PlayerProfile` -> `player_profile`

`player.name` schreibt also in den `Player` Store-Wert, `info.value` in `Info`.

## Funktionen

### `$set(target, value)`

Setzt `target` auf `value`.

```html
<input value="{{ player.name }}" onchange="$set(player.name, $event.value)">
<colorpicker value="{{ info.color }}" onchange="$set(info.color, $event.hex)"></colorpicker>
<input type="file" onchange="$set(info.image_file, $event.value)">
```

### `$add(target, value)`

Addiert einen numerischen Wert auf `target`.

```html
<button onclick="$add(info.value, 1)">Erhöhen</button>
<slider value="{{ info.value }}" onchange="$add(info.value, 0.5)"></slider>
```

`$add` akzeptiert Zahlen und numerische Strings.

### `$min(target, value)`

Subtrahiert einen numerischen Wert von `target`.

```html
<button onclick="$min(info.value, 1)">Verringern</button>
```

`$min(info.value, 1)` bedeutet `info.value = info.value - 1`.

## `$event`

`$event` liest Werte vom Widget, das das Event ausgelöst hat.
Der wichtigste Wert ist `$event.value`.

```html
<input onchange="$set(player.name, $event.value)">
<slider onchange="$set(info.value, $event.value)"></slider>
```

Unterstützte `$event` Felder:

- `$event.value`: primärer Widget-Wert
- `$event.checked`: checked/selected Boolean für Checkbox, Radio, Toggle und Switch
- `$event.selected`: selected Boolean für auswählbare Widgets
- `$event.text`: Label/Text für optionartige Widgets
- `$event.hex`: ColorPicker-Wert als `#RRGGBB`
- `$event.rgb`: ColorPicker-Wert als `rgb(r, g, b)`
- `$event.rgba`: ColorPicker-Wert als `rgba(r, g, b, a)`
- `$event.red`, `$event.green`, `$event.blue`, `$event.alpha`: Farbkanäle als Zahlen

## Value Types

Direkte Store-Keys behalten ihren bestehenden Rust-Primitive-Typ, wenn möglich:

- `bool`
- `String`
- unsigned integers: `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- signed integers: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- floats: `f32`, `f64`
- `serde_json::Value`

Verschachtelte Pfade wie `info.value` aktualisieren die template-sichtbare JSON-Projektion des Store-Werts.
Dadurch werden Platzhalter wie `{{ info.value }}` nach dem Inline-Event reaktiv aktualisiert.

## Literale

Du kannst Booleans, Zahlen, Strings, Arrays, Objects und `null` übergeben.

```html
<button onclick="$set(player.state, true)">Aktivieren</button>
<button onclick="$set(player.name, 'Ada')">Ada setzen</button>
<button onclick="$set(info.value, 42)">Auf 42 setzen</button>
<button onclick="$set(info.tags, ['ui', 'debug'])">Tags setzen</button>
```

Strings mit einfachen und doppelten Anführungszeichen werden unterstützt.

## Mehrere Aktionen

Mehrere Calls können mit Semikolon getrennt werden.

```html
<button onclick="$set(player.name, 'Ada'); $add(info.value, 1)">
  Preset anwenden
</button>
```

Nutze das nur für einfache Änderungen. Wenn daraus Business-Logik wird, gehört es in `#[html_fn]`.

## Komplettes Beispiel

```html
@use "crate::data_structs::*";

<div>
  <p>Name: {{ player.name }}</p>
  <input value="{{ player.name }}" onchange="$set(player.name, $event.value)">

  <p>Wert: {{ info.value }}</p>
  <button onclick="$min(info.value, 1)">-</button>
  <button onclick="$add(info.value, 1)">+</button>
  <slider value="{{ info.value }}" onchange="$set(info.value, $event.value)"></slider>

  <p>Farbe: {{ info.color }}</p>
  <colorpicker value="{{ info.color }}" onchange="$set(info.color, $event.hex)"></colorpicker>
</div>
```

## Häufige Fehler

- Den `$` Prefix nicht vergessen: `set(...)` wird als normaler Handlername behandelt.
- Template-Aliase sind snake_case: `PlayerProfile` wird `player_profile`, nicht `PlayerProfile`.
- `$add` und `$min` brauchen numerische aktuelle Werte und Eingabewerte.
- Text-Inputs können häufige reaktive Rebuilds auslösen. Bei schweren Templates besser vorerst `#[html_fn]` oder Commit/Blur-Flow nutzen, bis Debouncing verfügbar ist.
- Komplexe Logik bleibt in Rust. Shortland Functions sind für schnelles State-Wiring, nicht für Business-Logik.
