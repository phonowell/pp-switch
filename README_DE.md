<div align="center">

<img src="assets/icon.svg" width="72" height="72" alt="pp-switch icon">

# pp-switch

**Pure Provider Switch. Deine Konfiguration bleibt deine.**

Kleiner nativer Anbieterwechsler. Fünf Harnesses in der Entwicklungsversion 0.0.5.

[![CI](https://github.com/phonowell/pp-switch/actions/workflows/ci.yml/badge.svg)](https://github.com/phonowell/pp-switch/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/badge/license-MIT-465bab.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/macOS%20%7C%20Windows%20%7C%20Linux-native-465bab)

**[English](README.md) · [简体中文](README_ZH.md) · [日本語](README_JA.md) · Deutsch**

[Download](https://github.com/phonowell/pp-switch/releases) · [Anleitung (Englisch)](docs/user-guide.md) · [Messwerte](docs/verification.md) · [Spezifikation](SPEC.md)

</div>

![v0.0.5 main-window schematic, light and dark; not a native screenshot](docs/illustrations/v0.0.5/main-en.svg)

## Warum pp-switch?

Verbindungen wechseln und die bestehende Clientkonfiguration behalten. PPS verwaltet
URL/Schlüssel, ausdrücklich angegebene Modelle und begrenzte native/API-Wechsel.
Andere Einstellungen, Kommentare und Formatierung bleiben erhalten; nicht unterstützte
Konfigurationen werden abgelehnt.

Ein natives Fenster mit Lucide Bearbeiten/Wechseln je Zeile, ohne Tray oder Dauerprozess.
Updates nur nach manueller Prüfung und Bestätigung. Rust + FLTK, Systemschriften und
Vektorpfade; kein gebündeltes WebView, keine Datenbank oder JavaScript-Laufzeit.

Entwicklung: Claude Code, Codex CLI/App, Gemini CLI, Grok Build und DSH CLI/Web.
**Die veröffentlichte Version 0.0.4 unterstützt die ersten drei; 0.0.5 ist unveröffentlicht.**
[Umfang und cc-switch-Quellen](docs/verification.md#community-sources).

## Download

**0.0.4 wurde am 2026-10-12 abgenommen und veröffentlicht.** Chinesische Funktionstests und alle Sprachstile sind geprüft; [Nachweise](docs/smoke.md).

**[v0.0.4 Funktionsvorschau](https://github.com/phonowell/pp-switch/releases/tag/v0.0.4)**. macOS überschreitet das Speicherbudget; [Messwerte](docs/verification.md) sind öffentlich.

| Plattform | Paket | Start |
| --- | --- | --- |
| macOS · Apple Silicon | [ZIP](https://github.com/phonowell/pp-switch/releases/download/v0.0.4/pp-switch-v0.0.4-aarch64-apple-darwin.zip) | Entpacken, `.app` nach Applications verschieben |
| macOS · Intel | [ZIP](https://github.com/phonowell/pp-switch/releases/download/v0.0.4/pp-switch-v0.0.4-x86_64-apple-darwin.zip) | Ebenso |
| Windows · x64 | [ZIP](https://github.com/phonowell/pp-switch/releases/download/v0.0.4/pp-switch-v0.0.4-x86_64-pc-windows-msvc.zip) | Entpacken, `pp-switch.exe` starten |
| Linux · x64 | [tar.gz](https://github.com/phonowell/pp-switch/releases/download/v0.0.4/pp-switch-v0.0.4-x86_64-unknown-linux-gnu.tar.gz) | Entpacken, `pp-switch` auf einem Desktop starten |

Rust oder Node.js werden zum Ausführen nicht benötigt. macOS-Pakete deklarieren macOS 11+, sind aber **nicht mit Apple Developer signiert oder notarisiert**. Windows benötigt Windows 10+. Linux benötigt X11/XWayland, X11-/fontconfig-Bibliotheken und Systemschriften für die gewählte Sprache. Mindestversionen bedeuten nicht, dass jede OS-Version getestet wurde; siehe [Testumgebungen](docs/verification.md).

## Verwendung

1. Harness wählen, **Add** anklicken, Name/URL/Schlüssel eingeben und speichern.
2. Optionales Modell prüfen. Ohne konkretes Modell trägt Claude `claude-sonnet-5` ein; ein leeres Feld behält Client-Modelle bei.
3. Das Wechsel-Symbol der Zeile anklicken; den Client bei Bedarf neu starten.

Speichern ändert nur die Anbieterliste. Blaues **In use** prüft unterstützte
Benutzerdateien, keine laufende Sitzung oder erfolgreiche Anfrage. Namen sind pro
Client eindeutig; Importkollisionen erhalten ` (2)`, ` (3)`. Gleiche Verbindungen
können verschiedene Namen haben.

In 0.0.5 hat jeder Harness eine **Official**-Zeile für begrenzte native Wiederherstellung.
Codex CLI/App teilen ihre Konfiguration; DSH CLI/Web teilen initialisierte Home-Profile.
CCS/Sub2API-Links öffnen ungespeicherte Vorschauen; explizite Modelle haben Vorrang vor Editorvorschlägen. Prüfen Sie, ob Ihr Anbieter das vorgeschlagene Modell unterstützt.
Externe Key-Helper, Cloud-Modi und nicht unterstützte Schlüsselquellen werden abgelehnt.
[Einrichtung, genaue Schreibzugriffe und Fehlerhilfe](docs/user-guide.md).

Settings enthält Sprache, explizite Linkregistrierung und manuelle Updates. Oberfläche:
English, 简体中文, 繁體中文, 日本語; System-Hell/Dunkel. Schlüssel liegen in lokalen,
benutzerbeschränkten Dateien; Maskierung ist keine Verschlüsselung. Keine automatischen
Anbieteranfragen. Updates prüfen SHA-256 dieses Repositorys und ersetzen nur App-Dateien.
[Verhaltensvertrag](SPEC.md).

## Leistung mit Belegen

**Native CI-Messungen für 0.0.4:**

| Native CI | Installiert MiB | Leerlauf-RSS MiB | Skriptbudget |
| --- | ---: | ---: | --- |
| macOS 15 · ARM64 | 2.43 | 48.44 | Nicht bestanden |
| macOS 15 · Intel | 2.67 | 31.97 | Nicht bestanden |
| Windows Server 2022 · x64 | 2.78 | 15.08 | Bestanden |
| Ubuntu 22.04 · x64 · Xvfb | 3.01 | 16.96 | Nicht bestanden |

Windows besteht beide Szenarien; macOS überschreitet das Speicherbudget. Linux besteht das leere Fenster, überschreitet aber das Leerlaufbudget im Editor. Echter Kaltstart und Updatespitzen sind ungeprüft. [SPEC PP-11](SPEC.md#acceptance-contract) · [Raw data](docs/verification.md#release-004).

## Mitwirken

Beginne mit [SPEC.md](SPEC.md). Build-, Prüf- und Releaseanweisungen stehen im
[Beitragsleitfaden (Englisch)](CONTRIBUTING.md).

## Dank und Lizenz

Danke an [cc-switch](https://github.com/farion1231/cc-switch) für offene Protokoll- und Implementierungsreferenzen und an [fltk-rs](https://github.com/fltk-rs/fltk-rs) / [FLTK](https://www.fltk.org). pp-switch ist nicht mit cc-switch verbunden. [MIT](LICENSE). Abhängigkeiten behalten ihre [Lizenzen und Hinweise](THIRD_PARTY_NOTICES.txt).
