![Spike mit seinem Heft am See im Mondlicht](docs/spike-cover.png)

# Spike

[English](README.md) · [Русский](README-ru.md) · [Deutsch](README-de.md)

Drück **PWR**, und Spike erscheint auf der Steinterrasse am See. Tipp ihn an, dann hüpft er vor Freude. Lass den Finger einen Moment liegen, und er schnurrt. Der kleine Reisende im pflaumenfarbenen Umhang ist das Maskottchen des Projekts. Auf seinem runden Display findest du auch eine Mathe-Lernkarte für die Klassen 10 und 11.

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
