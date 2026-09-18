---
title: HTML- und CSS-Performance
---

# HTML- und CSS-Performance

Die Optimierungen sind automatisch aktiv und brauchen kein neues Feature-Flag.

## Aenderungen

- Binding-Snapshots und Fingerprints werden nur bei geaenderten Eingaberessourcen
  neu berechnet, statt unveraenderte Daten pro Frame zu kopieren und zu sortieren.
- Framework-Store und registrierte Shared Values werden gemeinsam uebernommen.
  Identische Store-Werte loesen dadurch keine kuenstlichen Aenderungen mehr aus.
- Ein Cache pro Bevy-App teilt vorbereitete Stylesheets und Selektorketten.
  Ein Index nach ID, Klasse und Tag reduziert die zu pruefenden CSS-Regeln.
  Verknuepfte Selektoren, Pseudo-Zustaende und Media Queries bleiben erhalten.
- Hinzugefuegte, geaenderte und entfernte CSS-Assets invalidieren betroffene
  Cache-Eintraege, auch bei Root-Variablen aus anderen Stylesheets.
- `calc(...)` meldet Layout-Aenderungen nur bei neuen Ergebnissen. Fenstergroesse,
  Elterngroesse, Hierarchie und Animationen werden weiterhin beruecksichtigt.

## Benchmark

Im Repository-Hauptverzeichnis:

```bash
cargo bench --bench html_css --features extended-framework
```

Der Benchmark verwendet 1.000 UI-Knoten, 250 nicht passende CSS-Regeln,
1.000 Shared Values, Aufwaermdurchlaeufe und jeweils 30 Messungen. Ausgegeben
werden Median und p95 fuer CSS-Anwendung, ruhende calc-Layouts, HTML-Parsing,
ruhende Bindings und Store-Synchronisierung.

Es handelt sich um CPU-Messungen ohne Fenster, GPU, Text-Shaping oder Bevy-Layout.
Die Ergebnisse sind keine FPS-Messung. `idle Changed<Node>` zeigt unabhaengig von
der Hardware, ob unnoetige Layout-Neuberechnungen angefordert werden.

Der kurze CI-Test prueft mit 100 Knoten und drei Messungen, dass im Leerlauf
kein Layout-Knoten als geaendert markiert wird:

```bash
cargo bench --bench html_css --profile dev --features extended-framework -- --smoke
```

## Vorher/Nachher

Gemessen auf AMD Ryzen 9 5900X, Linux x86_64, Rust 1.95.0, Bevy 0.19,
`--profile dev --features extended-framework`, 1.000 Knoten, 30 Messungen.
Ausgangsversion: Commit `5a608bc`; danach: die Performance-Aenderungen in diesem
Arbeitsverzeichnis. Dies sind unoptimierte Debug-Messungen.

| Messung | Vorher (Median) | Nachher (Median) |
| --- | ---: | ---: |
| CSS erneut anwenden | 566,390 ms | 31,671 ms |
| Unveraenderte HTML-Bindings | 4,780 ms | 0,394 ms |
| HTML-Parsing | 64,186 ms | 65,365 ms |
| calc-Schleife ohne Layout | 1,360 ms | 1,581 ms |
| Im Leerlauf als geaendert markierte Knoten | 1.000 | 0 |

In diesem Szenario ist die CSS-Anwendung etwa 18-mal und die Binding-Verarbeitung
etwa 12-mal schneller. Das Parsing wurde nicht optimiert. Der calc-Fix spart die
anschliessende Layout-Arbeit; die isolierte Schleife selbst wird nicht schneller.

Abschliessende optimierte Messung auf derselben Maschine nach Ende der parallelen
Builds, mit `cargo bench --bench html_css --features extended-framework`:

| Messung | Median | p95 |
| --- | ---: | ---: |
| CSS erneut anwenden | 2,185 ms | 3,410 ms |
| calc ohne Layout | 0,085 ms | 0,348 ms |
| HTML-Parsing | 6,424 ms | 7,921 ms |
| Unveraenderte HTML-Bindings | 0,034 ms | 0,054 ms |
| Unveraenderter Framework-Store mit Synchronisierung | 0,380 ms | 0,835 ms |

Diese optimierten Werte beschreiben nur die neue Implementierung. Der
Vorher/Nachher-Vergleich oben verwendet auf beiden Seiten das Debug-Profil.

Die vollstaendige Anwendung sollte im Release-Modus gemessen werden:

```bash
cargo run --release --manifest-path crates/local-examples/Cargo.toml -- widget-overview
```

Dabei Aufloesung und Szene konstant halten und Leerlauf, Hover, Resize,
Animationen und Hot Reload getrennt vergleichen. Unveraenderte eigene Ressourcen
nicht pro Frame neu schreiben; wo passend Bevy `set_if_neq` verwenden.

Regressionstests sichern Layout-Change-Detection, Resize, Transitionen,
Reparenting, Selektorindex, Cache-Isolation, CSS-Variablen und Hot Reload sowie
Binding-Updates nach Leerlauf und unveraenderte Framework-Stores ab.

Lokal bestanden: 298 Unit-/Integrationstests, 32 Doc-Tests (ein bestehender
ignorierter Doc-Test), CI-Smoke-Test, Library-Build ohne Default-Features und
lokale Beispiele mit `extended-framework`. Die bestehende CI-Konfiguration
erreicht 91,52 % Region-Coverage bei geforderten 90 %; das neue CSS-Cache-Modul
erreicht 98,48 %. Der GitHub-gehostete Workflow laeuft erst nach dem Push.
