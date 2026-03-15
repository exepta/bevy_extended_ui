---
title: Button
---

# Button

## Überblick

`Button` ist ein interaktives Aktions-Widget für Klick- und Formularabläufe. Der Widget-Text wird aus dem Inneren des Tags gelesen, ein optionales `<icon src="...">` kann automatisch links oder rechts vom Text platziert werden, und über den Typ (`button`, `submit`, `reset`) steuert das Widget direkt das Formularverhalten.

- Rust-Komponente: Button
- HTML-Tag: button
- Empfohlene Quellreferenz: src/widgets/mod.rs

## Attributes

Wichtige eigene Attributes (ausführlich):

- `type`: Steuert den Verhaltensmodus des Buttons.
  Zulässige Werte:
  `button` (normale Klickaktion ohne Submit), `submit` (löst Form-Submit aus), `reset` (setzt Formularwerte zurück).
  Ohne Angabe wird der interne Standard `Auto` verwendet.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert den Button initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

Zusätzlich relevant:

- `<icon src="..."></icon>`: Kein Attribut, sondern ein Kind-Element. Der Parser erkennt das Icon automatisch und setzt die Platzierung abhängig von der Textposition (`links` oder `rechts`).

## WASM Vorschauen

### Button states
<iframe
id="button"
title="Button States"
src="{base.url}/examples/base"
width="50%"
height="250px"
loading="lazy">
</iframe>

#### Html Example

```html
<button id="save-btn" class="cta" type="submit" onclick="on_save_click">
  Save
  <icon src="extended_ui/icons/check-mark.png"></icon>
</button>
```

## Rust Example

```rust
fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
    let handle: Handle<HtmlAsset> = asset_server.load("ui/button.html");
    reg.add_and_use("button-demo".to_string(), HtmlSource::from_handle(handle));
}

#[html_fn("on_save_click")]
fn on_save_click(In(event): In<HtmlClick>, query: Query<&Button>) {
    if let Ok(widget) = query.get(event.entity) {
        info!(
            "Button clicked: text='{}' type={:?} icon={:?}",
            widget.text,
            widget.button_type,
            widget.icon_path
        );
    }
}
```

### Icon Button

<iframe
id="button-icon-only"
title="Icon Button"
src="{base.url}/examples/base"
width="50%"
height="250px"
loading="lazy">
</iframe>

#### Html Example

```html
<button style="width: 50px; height: 50px; border-radius: 50%;">
  <icon src="icons/check-mark.png"></icon>
</button>
```

## Rust Example

```rust
fn load_ui(mut reg: ResMut<UiRegistry>, asset_server: Res<AssetServer>) {
  let handle: Handle<HtmlAsset> = asset_server.load("ui/button.html");
  reg.add_and_use("button-demo".to_string(), HtmlSource::from_handle(handle));
}
```

## Ersteller vom Widget

<div style="display: flex; align-items: center; justify-content: flex-start; padding: 15px; border: 1px solid #5658db; border-radius: 10px; gap: 15px; width: 300px;">
  <img
    src="https://avatars.githubusercontent.com/u/84874606?v=4"
    alt="exepta avatar"
    width="64"
    height="64"
    style="width: 64px; height: 64px; border-radius: 50%; object-fit: cover;"
  />
  <div style="display: flex; flex-direction: column; align-items: flex-start; justify-content: center;">
    <strong>exepta</strong>
    <a href="https://github.com/exepta" style="margin-top: 10px;">Link to GitHub</a>
  </div>
</div>
