# Game Boy Emulator

Ein in **[Rust](https://rust-lang.org/)** geschriebener Emulator für den originalen **Nintendo Game Boy (DMG-01)**.  
Dieses Projekt dient primär dem **tiefgehenden Verständnis der Game-Boy-Hardware**, insbesondere der **CPU (LR35902)**, Speicherarchitektur und Instruktionsausführung.

Der Fokus liegt auf **Korrektheit**, **Nachvollziehbarkeit** und **klarer Struktur**, nicht auf maximaler Performance oder Feature-Vollständigkeit.

---

## 🚧 Projektstatus

> **Work in Progress**

- CPU (LR35902): größtenteils implementiert
- Opcode-Decoding: vollständig (inkl. `CB`-Prefixed Opcodes)
- ALU-Operationen: ADD, ADC, SUB, SBC, AND, OR, XOR, CP
- Register- und Flag-Handling
- MMU (ROM, WRAM, HRAM – aktuell minimal)

Noch **nicht** oder nur teilweise implementiert:
- PPU (Grafik)
- APU (Sound)
- Timer
- Interrupts (teilweise vorbereitet)
- Joypad
- MBCs (aktuell ROM-only)

---

## 🎯 Ziele des Projekts

- Saubere, verständliche Emulator-Architektur
- Möglichst genaue Abbildung der Game-Boy-CPU
- Kein „Magic Code“ – alles ist erklärbar und testbar
- Lernprojekt mit Fokus auf **Low-Level-Emulation**

**Nicht-Ziele:**
- Geschwindigkeit um jeden Preis
- Cycle-accurate PPU/APU (vorerst)
- Unterstützung aller Cartridge-Typen

---

## 🧠 Architektur (Kurzüberblick)

```
CPU
 ├── Register (A, F, B, C, D, E, H, L, SP, PC)
 ├── Flags (Z, N, H, C)
 ├── Opcode-Decoder
 └── ALU

MMU
 ├── ROM (0x0000–0x7FFF)
 ├── WRAM (0xC000–0xDFFF)
 └── HRAM (0xFF80–0xFFFE)
```

Die CPU greift **ausschließlich über die MMU** auf Speicher zu.  
Immediate-Werte (`n8`, `n16`) werden zur Laufzeit über die MMU aus dem ROM gelesen.

---

## 🧪 Tests & Debugging

- Fokus auf kleine, isolierte CPU-Tests
- Manuelles Gegenprüfen mit Referenztabellen
- Schrittweises Ausführen einzelner Opcodes
- Logging während der Opcode-Ausführung

Geplant:
- Integration von bekannten CPU-Test-ROMs (z. B. Blargg)

---

## 📚 Verwendete Referenzen & Ressourcen

Dieses Projekt orientiert sich stark an den folgenden **exzellenten Dokumentationen**:

### CPU & Opcodes
- **Game Boy Opcode Tables**  
  https://gbdev.io/gb-opcodes/optables/

- **GBZ80 Instruction Reference (RGBDS)**  
  https://rgbds.gbdev.io/docs/v1.0.0/gbz80.7#SWAP_r8

### Register & Flags
- **Pandocs – CPU Registers and Flags**  
  https://gbdev.io/pandocs/CPU_Registers_and_Flags.html

### Allgemein
- **Pandocs (Game Boy Technical Reference)**  
  https://gbdev.io/pandocs/

Diese Quellen gelten als **maßgeblicher Standard** in der Game-Boy-Emulator-Entwicklung.

---

## ⚙️ Build & Run

Voraussetzungen:
- Rust (stable)

```bash
cargo run -- --rom_path path/to/rom.gb
```

> Hinweis: Der Emulator ist aktuell **nicht spielbar**, sondern primär ein Entwicklungs- und Debug-Tool.

---

## 📦 Rechtliches

- Dieses Projekt enthält **keine ROMs**
- Nintendo Game Boy ist eine Marke von Nintendo
- Dieses Projekt ist rein zu **Lern- und Forschungszwecken**

---

## 📝 Lizenz

Lizenz: **GPL**  
Details siehe `LICENSE`-Datei.

---

## 🙌 Beiträge

Pull Requests, Issues und Diskussionen sind willkommen –  
insbesondere zu:
- CPU-Edge-Cases
- Flag-Berechnungen
- Emulator-Architektur
- Tests & Debugging-Strategien
