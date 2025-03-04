# DupFi - Duplicate File Finder

DupFi ist ein leistungsstarker Duplikat-Finder mit einer benutzerfreundlichen grafischen Oberfläche, der Ihnen hilft, doppelte Dateien zu finden und zu verwalten.

## Features

✅ **Benutzerfreundliche Oberfläche**
- Integrierter Datei-Explorer zur Verzeichnisauswahl
- Fortschrittsanzeige für große Scans
- Übersichtliche Darstellung der Duplikate

✅ **Leistungsstarke Duplikat-Erkennung**
- Schnelle Erkennung durch SHA256-Hashing
- Größenbasierte Vorfilterung für optimale Performance
- Multithreading für schnelle Scans

✅ **Flexible Verwaltungsoptionen**
- Löschen von Duplikaten
- Erstellen von Hardlinks zur Speicherplatzoptimierung
- Verschieben von Dateien
- Vorschau für Text- und Bilddateien

✅ **Filter-Optionen**
- Ausschließen von Dateitypen
- Anpassbare Filterregeln

## Installation

1. Laden Sie die neueste Version von DupFi herunter
2. Entpacken Sie die ZIP-Datei
3. Starten Sie `dupfi.exe`

## Verwendung

1. Klicken Sie auf "📁 Select Directory" um ein Verzeichnis auszuwählen
2. Optional: Fügen Sie Filter hinzu, um bestimmte Dateitypen auszuschließen
3. Klicken Sie auf "🔍 Start Scan" um die Suche zu starten
4. Verwalten Sie gefundene Duplikate mit den verfügbaren Optionen:
   - 🗑️ Löschen
   - 🔗 Hardlink erstellen
   - 📦 Verschieben

## Technische Details

- Geschrieben in Rust
- Verwendet egui für die Benutzeroberfläche
- Multithreading mit rayon
- Sichere Dateiverwaltung mit Fehlerbehandlung

## Build from Source

```bash
# Repository klonen
git clone https://github.com/yourusername/dupfi.git
cd dupfi

# Release-Version bauen
cargo build --release

# Ausführen
cargo run --release
```

## Lizenz

MIT License
