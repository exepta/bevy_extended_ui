---
title: Slider
---

# Slider

### Überblick

Numerisches Zieh-Widget mit Min-, Max-, Wert- und Schritt-Eigenschaften.
Unterstützt `default` (ein Thumb) und `range` (zwei Thumbs).

- Rust-Komponente: Slider
- HTML-Tag: slider
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- `min`, `max`, `step`: Zahlenbereich und Schrittweite.
- `type`: `default | range` (Standard: `default`).
- `value`:
  - `default`: einzelner Zahlenwert (z. B. `35`)
  - `range`: Bereich im Format `start - end` (z. B. `20 - 40`)
- `dots`: Anzahl der Segmente/Ticks. Beispiel `dots="5"` bei `0..100` erzeugt Ticks bei `0, 20, 40, 60, 80, 100`.
  - Wenn `dots` gesetzt ist und `<= 1` (auch `0` oder negativ), werden nur Min/Max-Ticks genutzt (wie `dots="1"`).
  - Interner Schutz: Mindestabstand zwischen Nachbar-Ticks ist 10 Werteinheiten.
- `show-labels`: `true | false` (Standard: `false`). Zeigt Tick-Labels.
- `dot-anchor`: `top | bottom` (Standard: `top`). Position der Tick-Labels relativ zum Track.
- `tip`: `true | false` (Standard: `true`). Zeigt/verbirgt Thumb-Tooltip.
- Track reagiert auf Klick und Drag.
- Im `range`-Modus zeigt der Tooltip den Wert des jeweils gehoverten Thumbs.
- Funktioniert mit onchange/oninput-Handlern.
- Aktueller Wert liegt in der Slider-Komponente.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### Slider
<iframe
  id="slider"
  title="Slider"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<slider min="0" max="100" value="25" step="1"></slider>
<slider value="0" min="0" max="100" step="1" dots="4" show-labels="true" tip="false"></slider>
```

#### Rust Example

```rust
fn spawn_slider_widget(mut commands: Commands) {
    commands.spawn((
        Slider {
            min: 0.0,
            max: 100.0,
            value: 25.0,
            step: 1.0,
            ..default()
        },
        Node::default(),
    ));
    commands.spawn((
        Slider {
            min: 0.0,
            max: 100.0,
            value: 0.0,
            step: 1.0,
            dots: Some(4),
            show_labels: true,
            show_tip: false,
            ..default()
        },
        Node::default(),
    ));
}
```

### Slider Range
<iframe
id="slider-range"
title="Slider"
src="{base.url}/examples/base"
width="100%"
height="420"
loading="lazy">
</iframe>

#### Html Example

```html
<slider id="range-slider-demo" type="range" value="20 - 40" dots="4" show-labels="true" dot-anchor="bottom"></slider>
```

#### Rust Example

```rust
fn spawn_slider_widget(mut commands: Commands) {
    commands.spawn((
        Slider {
            slider_type: SliderType::Range,
            range_start: 20.0,
            range_end: 40.0,
            dots: Some(4),
            show_labels: true,
            dot_anchor: SliderDotAnchor::Bottom,
            ..default()
        },
        Node::default(),
    ));
}
```

### Ersteller vom Widget

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
    <a href="https://github.com/exepta" style="margin-top: 10px; color: #5658db;">Link to GitHub</a>
  </div>
</div>
