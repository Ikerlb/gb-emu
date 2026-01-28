# Rust Game Boy Emulator

A Game Boy (DMG) emulator written in Rust, aiming for accuracy and clean architecture.

## Project Status

**Playable!** - Tetris runs and is fully playable with graphics and input.

### What's Implemented

- **CPU** - All 512 opcodes (256 standard + 256 CB-prefix)
- **PPU** - Full rendering (background, window, sprites with proper priority)
- **Memory Map** - Complete implementation (VRAM, WRAM, HRAM, OAM, Echo RAM)
- **Cartridge** - ROM loading, MBC0/MBC1 bank switching
- **Timer** - DIV, TIMA, TMA, TAC registers with overflow interrupts
- **Interrupts** - VBlank and Timer interrupts, IME, HALT
- **Input** - Joypad with keyboard mapping
- **OAM DMA** - Sprite data transfer (0xFF46)
- **TUI Debugger** - Breakpoints, stepping, memory viewer, register inspector

### What's Missing

- Audio (APU)
- Serial I/O
- MBC2/MBC3/MBC5 (needed for more games)
- Save file persistence (.sav)

## Architecture

```
GameBoy (orchestrator)
├── CPU (Sharp LR35902)
│   ├── All 512 opcodes (256 + 256 CB-prefix)
│   ├── Interrupt handling (IME, HALT)
│   └── Flags: Z, N, H, C
├── PPU (Pixel Processing Unit)
│   ├── Background rendering with scrolling
│   ├── Window layer
│   ├── Sprite rendering (8x8 and 8x16)
│   ├── Mode transitions (OAM → Drawing → HBlank → VBlank)
│   └── 160x144 framebuffer output
├── Timer
│   ├── DIV (divider), TIMA (counter), TMA (modulo), TAC (control)
│   └── Overflow interrupt generation
├── Joypad
│   ├── Direction buttons (D-pad)
│   └── Action buttons (A, B, Start, Select)
└── Interconnect (memory bus)
    ├── Cartridge (ROM/RAM + MBC0/MBC1)
    ├── VRAM (8KB), WRAM (8KB), HRAM (127B), OAM (160B)
    ├── OAM DMA transfer (0xFF46)
    └── I/O register routing
```

## Building

Requires Rust 1.56+ (2021 edition)

```bash
cargo build --release
cargo test              # Run 150+ unit tests
```

## Running

```bash
# Play a game with display
cargo run --release -- -f "Tetris (World).gb" --display

# With larger window (2x, 3x, or 4x scale)
cargo run --release -- -f "rom.gb" --display --scale 4

# Debug output (headless)
cargo run -- -f "rom.gb" --debug

# Interactive TUI debugger
cargo run -- -f "rom.gb" --interactive

# Limited execution with memory dump
cargo run -- -f "rom.gb" -m 1000 --dump-mem 0xFE00:0xFE9F
```

## Controls

| Key | Game Boy Button |
|-----|-----------------|
| Arrow keys | D-pad |
| Z | A |
| X | B |
| Enter | Start |
| Backspace | Select |
| Escape | Quit |

## TUI Debugger

Run with `--interactive` for a full-featured debugger:

```
┌─ Registers ─────────────────┐┌─ Flags ─────┐
│ PC:0100  SP:FFFE            ││ Z:1  N:0    │
│ AF:01B0  BC:0013            ││ H:1  C:0    │
└─────────────────────────────┘└─────────────┘
┌─ Memory 0x0100-0x017F ──────────────────────┐
│ 0100 │ 00 C3 50 01 CE ED ... │ ..P.......  │
└─────────────────────────────────────────────┘
```

**Commands:** `step`, `continue`, `break <addr>`, `mem <range>`, `reg`, `quit`

## Dependencies

- `minifb` - Window and framebuffer display
- `num-traits`, `num-derive` - Numeric conversions
- `ratatui`, `crossterm` - TUI debugger
- `clap` - Command-line parsing

## Resources

- [Pan Docs](https://gbdev.io/pandocs/) - Comprehensive GB technical reference
- [GBEDG](https://hacktix.github.io/GBEDG/) - Game Boy Emulator Development Guide
- [Blargg's Test ROMs](https://github.com/retrio/gb-test-roms)

## License

Dual licensed under MIT or Apache-2.0.
