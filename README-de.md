![Spike mit seinem Heft am See im Mondlicht](docs/spike-cover.png)

# Spike

[English](README.md) · [Русский](README-ru.md) · [Deutsch](README-de.md)

Drück **PWR**, und Spike erscheint auf der Steinterrasse am See. Tipp ihn an, dann hüpft er vor Freude. Lass den Finger einen Moment liegen, und er schnurrt. Der kleine Reisende im pflaumenfarbenen Umhang ist das Maskottchen des Projekts. Auf seinem runden Display findest du auch eine Mathe-Lernkarte für die Klassen 10 und 11.

## Wie Spike entstanden ist

- **Die ersten Pixel.** Am Anfang stand nur `hello everynwan` auf dem Display. Auf dem Board war der Text verkehrt herum zu sehen. Heute dreht der Treiber jeden Bildstreifen um 180° und löscht beim Start auch den Displayspeicher am Rand des sichtbaren Kreises.
- **Vom Porträt zur Figur.** Zuerst bestand Spike aus einem einzigen Sprite. Eigene Posen für Schritte, Sprünge und Schlaf machen seine Reaktionen schon am Umriss erkennbar. Später kamen die Terrasse am See, Spitzbögen und elfenbeinfarbene Schrift hinzu.
- **Eine Karte für die Fingerspitze.** Aus dem ersten Themenbaum wurden 14 Lernblätter. Beim Test auf dem Board reichte eine leichte Fingerbewegung, um die Karte versehentlich zu verschieben. Größere Schaltflächen und 24 Pixel Spielraum machen auch Berührungen am Rand zuverlässig. Die Rückkehr zur Karte erhält die Position.
- **Strombedarf und Releases.** Nach einer Eingabepause schläft das Display auch bei laufender Animation ein. Für Release-Tags baut der Workflow getrennte russische und englische Images. Stichproben in den fertigen Dateien bestätigten, dass die geprüften Texte nur in der jeweiligen Sprachversion vorkommen.

## Bedienung

| Aktion | Funktion |
| --- | --- |
| PWR drücken | Spike erscheint. Wische seitwärts zum Gehen, nach oben zum Springen oder nach unten zum Einschlafen. Erneutes Drücken führt zur Begrüßung zurück. |
| BOOT kurz drücken und loslassen | Öffnet die Einstellungen. Wähle **30**, **60** oder **120** Sekunden bis zum Abschalten des Displays. Die Auswahl gilt bis zum Neustart. Voreingestellt sind 60 Sekunden. |
| BOOT 0,9 Sekunden halten | Öffnet die Lernkarte. Ziehe sie seitwärts für weitere Themen. Wische nach oben, um von Klasse 11 zu Klasse 10 zu gelangen. |
| Ein Thema antippen | Öffnet das Lernblatt. Mit **Back to map** (englische Firmware) kehrst du zur selben Stelle zurück. |
| Bei ausgeschaltetem Display eine Taste drücken | Weckt die aktuelle Ansicht. Der nächste Tastendruck hat wieder die übliche Funktion. |

Die 14 Lernblätter beruhen auf MCKO-Materialien für Klasse 10 (2026) und Klasse 11 (2023). Beginne bei der Ableitung und ziehe die Karte zum benachbarten Thema Funktionen. [Themen für Klasse 10](https://100ballnik.com/wp-content/uploads/2026/04/opisanie_ma_10_profil_mcko2026.pdf) · [Themen für Klasse 11](https://100ballnik.com/wp-content/uploads/2023/09/mcko_2023_mat_11klass_variant.pdf)

## Bauen und flashen

Du brauchst ein **Waveshare ESP32-S3-Touch-AMOLED-1.32** und ein USB-Datenkabel. Installiere die Xtensa-Rust-Toolchain unter dem Namen `esp` sowie das Programm `espflash`. Die CI verwendet Toolchain 1.97.0.0 und espflash 4.7.0. Führe die Befehle im Projektverzeichnis aus. Der zweite Befehl schreibt die Firmware auf das Board und öffnet den seriellen Monitor. [espup](https://github.com/esp-rs/espup) · [espflash](https://github.com/esp-rs/espflash)

**Russische Firmware (Standard)**

```sh
cargo build --release --locked
cargo run --release --locked
```

**Englische Firmware**

```sh
cargo build --release --locked --features english
cargo run --release --locked --features english
```

Jedes Image enthält einen Satz übersetzter Texte und einen Zeichensatz für die Oberfläche. Die Startmeldung `hello everynwan` ist in beiden Ausgaben gleich. Um die Sprache zu wechseln, flashe die andere Ausgabe. Deutsch steht als Dokumentationssprache zur Verfügung.

## Hinter dem runden Display

![Die acht Posen von Spike aus der Firmware](assets/spike-poses.png)

Die Firmware zeichnet das AMOLED mit 466 × 466 Pixeln in Streifen von vier Zeilen. Dafür nutzt sie einen DMA-Puffer von 3840 Byte. Beim Bauen werden Spikes acht Posen als 4-Bit-Palettenbilder und der See als Folgen gleichfarbiger RGB565-Pixel gepackt. Beides liegt im Flash-Speicher. Bei der Animation wird nur Spikes Szenenbereich aktualisiert.

Die Grafiken liegen in [assets](assets), die übersetzten Texte und Lernblätter in [src/lang](src/lang) und die Gestenerkennung in [src/gestures.rs](src/gestures.rs). Lokale Prüfungen beschreibt der [Leitfaden für Mitwirkende](AGENTS.md). Ein Tag nach dem Muster `v*` auf einem Commit aus der Historie von `main` erzeugt zwei Release-Images, `spike-esp32s3.bin` (RU) und `spike-esp32s3-en.bin` (EN), jeweils mit SHA-256-Prüfsumme. [Release-Workflow](.github/workflows/firmware.yml)

## Ideen für die nächste Version

1. **Abschaltzeit merken.** Den Wert bei einer Änderung speichern, damit er nach dem Neustart wieder gilt.
2. **Mit den Lernblättern üben.** Mit einer Aufgabe zur Ableitung beginnen und den Rechenweg durch Antippen einblenden. Die Formel bleibt zum schnellen Nachschlagen zugänglich.
3. **Stromaufnahme messen.** Auf demselben Board drei Zustände vergleichen: Spike in Bewegung, die geöffnete Lernkarte und das schlafende Display. Die Messwerte helfen bei der Entscheidung, ob sich ein tieferer Schlafmodus lohnt.
