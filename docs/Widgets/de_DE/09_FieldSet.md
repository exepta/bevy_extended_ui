---
title: FieldSet
---

# FieldSet

### Überblick

Gruppen-Widget für auswählbare Kinder wie Radio- und Toggle-Elemente mit Selektionsregeln.

- Rust-Komponente: FieldSet
- HTML-Tag: fieldset
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- mode unterstützt single, multi, count.
- allow-none steuert leere Auswahlzustände.
- Gruppiert radio/toggle Kinder mit gemeinsamem Selektionszustand.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID für CSS-Selektoren, Event-Zuordnung und spätere Widget-Referenzierung.
- `class`: Übergibt CSS-Klassen für visuelles Styling und zustandsabhängige Regeln.
- `style`: Übergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline übernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State übernommen, um ein konsistentes Zustandsmodell zu gewährleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknüpfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### FieldSet Single
<iframe
  id="fieldset-single"
  title="FieldSet"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<fieldset mode="single" allow-none="false" class="con-column">
  <radio value="easy">Easy</radio>
  <radio value="hard" selected>Hard</radio>
</fieldset>
```

#### Rust Example

```rust
fn spawn_fieldset_widget(mut commands: Commands) {
    commands
        .spawn((
            FieldSet {
                field_mode: FieldMode::Single,
                allow_none: false,
                ..default()
            },
            Node::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                RadioButton {
                    label: "Easy".to_string(),
                    value: "easy".to_string(),
                    selected: false,
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                RadioButton {
                    label: "Hard".to_string(),
                    value: "hard".to_string(),
                    selected: true,
                    ..default()
                },
                Node::default(),
            ));
        });
}
```

### FieldSet Multiple
<iframe
id="fieldset-multiple"
title="FieldSet"
src="{base.url}/examples/base"
width="100%"
height="420"
loading="lazy">
</iframe>

#### Html Example

```html
<fieldset mode="multi" allow-none="true" class="con-column">
  <toggle value="easy">Easy</toggle>
  <toggle value="medium">Medium</toggle>
  <toggle value="hard" selected>Hard</toggle>
</fieldset>
```

#### Rust Example

```rust
fn spawn_fieldset_widget(mut commands: Commands) {
    commands
        .spawn((
            FieldSet {
                field_mode: FieldMode::Multi,
                allow_none: true,
                ..default()
            },
            Node::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                ToggleButton {
                    label: "Easy".to_string(),
                    value: "easy".to_string(),
                    selected: false,
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                ToggleButton {
                    label: "Medium".to_string(),
                    value: "medium".to_string(),
                    selected: false,
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                ToggleButton {
                    label: "Hard".to_string(),
                    value: "hard".to_string(),
                    selected: true,
                    ..default()
                },
                Node::default(),
            ));
        });
}
```

### FieldSet Count
<iframe
id="fieldset-count"
title="FieldSet"
src="{base.url}/examples/base"
width="100%"
height="420"
loading="lazy">
</iframe>

#### Html Example

```html
<fieldset mode="count(2)" allow-none="false" class="con-column">
  <toggle value="very-easy">Very Easy</toggle>
  <toggle value="easy">Easy</toggle>
  <toggle value="medium">Medium</toggle>
  <toggle value="hard" selected>Hard</toggle>
  <toggle value="very-hard">Very Hard</toggle>
</fieldset>
```

#### Rust Example

```rust
fn spawn_fieldset_widget(mut commands: Commands) {
    commands
        .spawn((
            FieldSet {
                field_mode: FieldMode::Count(2),
                allow_none: false,
                ..default()
            },
            Node::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                ToggleButton {
                    label: "Very Easy".to_string(),
                    value: "very-easy".to_string(),
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                ToggleButton {
                    label: "Easy".to_string(),
                    value: "easy".to_string(),
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                ToggleButton {
                    label: "Medium".to_string(),
                    value: "medium".to_string(),
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                ToggleButton {
                    label: "Hard".to_string(),
                    value: "hard".to_string(),
                    selected: true,
                    ..default()
                },
                Node::default(),
            ));
            parent.spawn((
                ToggleButton {
                    label: "Very Hard".to_string(),
                    value: "very-hard".to_string(),
                    ..default()
                },
                Node::default(),
            ));
        });
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
