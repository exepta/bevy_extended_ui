---
title: List
---

# List

### Überblick

Listen-Widget auf Basis von `<listbox>`, bei dem alle Optionen gleichzeitig sichtbar sind.

- Rust-Komponente: ListBox
- HTML-Tag: listbox
- Empfohlene Quellreferenz: src/widgets/mod.rs

### Attributes

Wichtige eigene Attributes (ausführlich):

- `multiselect` auf `<listbox>` aktiviert Mehrfachauswahl.
- Ohne `multiselect` ist nur eine Auswahl gleichzeitig aktiv.
- `<option value="...">` setzt den internen Wert.
- `<option selected>` markiert initial ausgewaehlte Eintraege.
- `<option icon="...">` setzt optional ein Icon pro Eintrag.
- `<option internal-value-type="...">` legt den Zieltyp fuer den geparsten Wert fest.

Unterstützte globale HTML-Attribute:

- `id`: Eindeutige ID fuer CSS-Selektoren, Event-Zuordnung und spaetere Widget-Referenzierung.
- `class`: Uebergibt CSS-Klassen fuer visuelles Styling und zustandsabhaengige Regeln.
- `style`: Uebergibt Inline-CSS, das in `HtmlStyle` geparsed und in die Style-Pipeline uebernommen wird.
- `hidden`: Rendert das Widget initial unsichtbar.
- `disabled`: Deaktiviert Interaktionen; Klicks und Fokuswechsel werden entsprechend geblockt.
- `readonly`: Wird als Widget-State uebernommen, um ein konsistentes Zustandsmodell zu gewaehrleisten.
- Event-Attribute wie `onclick`, `onmousedown`, `onmouseup`, `onmouseover`, `onmouseout`, `onfocus`, `oninit`, `onchange`, `onscroll`, `onwheel`, `onkeydown`, `onkeyup`: Verknuepfen Handler-Funktionen direkt mit dem Event-Binding-System.

### WASM Vorschauen

### List
<iframe
  id="listbox"
  title="List"
  src="{base.url}/examples/base"
  width="100%"
  height="420"
  loading="lazy">
</iframe>

#### Html Example

```html
<listbox id="difficulty" onchange="log_listbox">
  <option value="easy">Easy</option>
  <option value="normal" selected>Normal</option>
  <option value="hard">Hard</option>
</listbox>
```

#### Rust Example

```rust
fn spawn_list_widget(mut commands: Commands) {
    let easy = ChoiceOption::new("Easy").with_value(String::from("easy"));
    let normal = ChoiceOption::new("Normal").with_value(String::from("normal"));
    let hard = ChoiceOption::new("Hard").with_value(String::from("hard"));

    commands.spawn((
        ListBox {
            options: vec![easy.clone(), normal.clone(), hard.clone()],
            values: vec![normal],
            multiselect: false,
            ..default()
        },
        Node::default(),
    ));
}
```

### Ersteller vom Widget

<div style="display: flex; align-items: center; justify-content: flex-start; padding: 15px; border: 1px solid #5658db; border-radius: 10px; gap: 15px; width: 300px;">
  <img
    src="https://avatars.githubusercontent.com/shadow-wolftousen"
    alt="shadow-wolftousen avatar"
    width="64"
    height="64"
    style="width: 64px; height: 64px; border-radius: 50%; object-fit: cover;"
  />
  <div style="display: flex; flex-direction: column; align-items: flex-start; justify-content: center;">
    <strong>shadow-wolftousen</strong>
    <a href="https://github.com/shadow-wolftousen" style="margin-top: 10px; color: #5658db;">Link zu GitHub</a>
  </div>
</div>
