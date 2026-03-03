---
title: Überblick
---

# Bevy Extended UI Überblick

<p class="description">Bevy Extended UI ist eine Open-Source-Crate, die Bevy-UI um einen HTML/CSS-Workflow und wiederverwendbare Widgets erweitert.</p>

## Welches Problem wird gelöst?

Die Standard-UI von Bevy ist leistungsfähig, aber größere Oberflächen werden schnell aufwändig.
Bevy Extended UI ermöglicht eine deklarative Struktur mit HTML, CSS-Styles und direkten Event-Bindings zu Rust-Funktionen.

Damit lassen sich komplexe Benutzeroberflächen deutlich schneller aufbauen und ändern.

## Zentrale Fähigkeiten

- HTML-basierte UI-Quellen als Bevy-Assets
- CSS-Unterstützung inklusive Breakpoints und `@keyframes`
- Viele Widgets wie Slider, Checkboxen, Choice-Boxen und Date-Picker
- Event-Bindings von HTML-Attributen nach Rust (`onclick`, `onchange`, `action`, Tastatur- und Drag-Events)
- Hot Reload für schnellere Entwicklung
- Optionale Sprach-Backends (`fluent`, `properties-lang`)
- WASM-Features (`wasm-default`, `wasm-breakpoints`, `clipboard-wasm`)

## Typischer Ablauf

1. Abhängigkeiten in `Cargo.toml` eintragen.
2. `ExtendedUiPlugin` in der App registrieren.
3. HTML-Datei aus `assets/` laden.
4. Quelle im `UiRegistry` registrieren.
5. Handler mit `#[html_fn(...)]` aus `bevy_extended_ui_macros` anbinden.

## Wann passt die Crate gut?

- Für umfangreiche Menüs und HUDs
- Für formularähnliche Oberflächen mit vielen Zuständen
- Für Projekte, in denen HTML/CSS-Workflows gewünscht sind

## Nächster Schritt

Weiter mit der Installationsanleitung: [Installation](./Installation.md).

Zusätzlicher Hinweis zu ÄÖÜ: Diese deutsche Dokumentation nutzt bewusst Umlaute wie Ä, Ö und Ü.
