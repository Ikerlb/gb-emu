# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build the project
cargo build

# Build release (optimized, recommended for playing)
cargo build --release

# Run with display (play a game)
cargo run --release -- -f "Tetris (World).gb" --display

# Run with larger scale (2x, 3x, or 4x)
cargo run --release -- -f "rom.gb" --display --scale 4

# Run with debug output (compact format)
cargo run -- -f "Tetris (World).gb" --debug

# Run with verbose debug output (multi-line format)
cargo run -- -f "Tetris (World).gb" --debug --verbose

# Run limited number of instructions (useful for testing)
cargo run -- -f "Tetris (World).gb" --debug --max-instructions 10

# Short flags version
cargo run -- -f "Tetris (World).gb" -d -v -m 10

# Run interactive debugger
cargo run -- -f "Tetris (World).gb" --interactive

# Dump memory range after execution
cargo run -- -f "Tetris (World).gb" -m 10 --dump-mem 0x0100:0x010F

# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture
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

## Architecture Overview

This is a Game Boy emulator written in Rust. **Tetris is fully playable.** The architecture follows a modular design:

```
GameBoy (orchestrator)
├── CPU (Sharp LR35902) - 100% complete
│   ├── All 512 opcodes (256 standard + 256 CB-prefix)
│   ├── Interrupt handling (IME, EI, DI, RETI, HALT)
│   └── Flags: Z, N, H, C
├── PPU (Pixel Processing Unit) - Complete
│   ├── Background rendering with SCX/SCY scrolling
│   ├── Window layer (WX, WY)
│   ├── Sprite rendering (8x8 and 8x16, priority, flipping)
│   ├── Mode transitions (OAM → Drawing → HBlank → VBlank)
│   └── 160x144 framebuffer with DMG green palette
├── Timer - Complete
│   ├── DIV (0xFF04) - Divider register
│   ├── TIMA (0xFF05) - Timer counter
│   ├── TMA (0xFF06) - Timer modulo
│   ├── TAC (0xFF07) - Timer control
│   └── Overflow interrupt generation
├── Joypad - Complete
│   ├── P1/JOYP register (0xFF00)
│   ├── Direction/action button selection
│   └── Keyboard input mapping
└── Interconnect (memory bus) - Complete
    ├── Cartridge (ROM/RAM + MBC0/MBC1)
    ├── VRAM (8KB), WRAM (8KB), HRAM (127B), OAM (160B)
    ├── OAM DMA transfer (0xFF46)
    ├── Timer register routing (0xFF04-0xFF07)
    ├── Joypad register routing (0xFF00)
    ├── PPU register routing (0xFF40-0xFF4B)
    └── Interrupt registers (IE: 0xFFFF, IF: 0xFF0F)
```

### Key Modules

- **`src/gb/gameboy.rs`** - Top-level emulation loop, orchestrates CPU/PPU/Timer
- **`src/gb/cpu.rs`** - CPU with all 512 opcodes and interrupt handling
- **`src/gb/ppu.rs`** - PPU with full scanline rendering (BG, window, sprites)
- **`src/gb/timer.rs`** - Timer subsystem with interrupt generation
- **`src/gb/joypad.rs`** - Joypad input handling with button selection logic
- **`src/gb/interconnect.rs`** - Memory bus routing all hardware components
- **`src/gb/mbc/`** - Memory Bank Controllers (MBC0, MBC1 implemented)
- **`src/gb/register.rs`** - 16-bit register pair abstraction (hi/lo byte access)
- **`src/gb/opcode.rs`** - Opcode enum definitions using `num-derive`
- **`src/gb/debug.rs`** - Debug configuration (DebugConfig struct)
- **`src/gb/debugger/`** - TUI debugger module (core.rs, tui.rs)

### Interactive TUI Debugger

Run with `--interactive` or `-i` to enter the TUI debugger:

```
┌─ Registers ─────────────────┐┌─ Flags ─────┐
│ PC:0100  SP:FFFE            ││ Z:1  N:0    │
│ AF:01B0  BC:0013            ││ H:1  C:0    │
│ DE:00D8  HL:014D            │└─────────────┘
└─────────────────────────────┘┌─ Breakpoints ┐
┌─ Memory 0x0100-0x017F ──────┐│ 0: 0x0150    │
│ 0100 │ 00 C3 50 01 ... │ ..│└──────────────┘
└─────────────────────────────┘
```

**Commands:** `step`/`s`, `continue`/`c`, `break <addr>`/`b`, `delete <id>`/`d`, `list`/`l`, `reg`/`r`, `mem <range>`/`m`, `help`/`h`, `quit`/`q`

### Memory Map

| Address Range | Description | Implementation |
|---------------|-------------|----------------|
| 0x0000-0x7FFF | Cartridge ROM | via Cartridge/MBC |
| 0x8000-0x9FFF | VRAM (8KB) | vram array |
| 0xA000-0xBFFF | External RAM | via Cartridge/MBC |
| 0xC000-0xDFFF | Work RAM (8KB) | wram array |
| 0xE000-0xFDFF | Echo RAM | mirrors WRAM |
| 0xFE00-0xFE9F | OAM (160B) | oam array |
| 0xFEA0-0xFEFF | Unusable | returns 0xFF |
| 0xFF00 | Joypad (P1) | Joypad module |
| 0xFF04-0xFF07 | Timer | Timer module |
| 0xFF0F | IF (Interrupt Flags) | if_register |
| 0xFF40-0xFF4B | PPU registers | PPU module |
| 0xFF46 | OAM DMA | oam_dma() |
| 0xFF80-0xFFFE | High RAM (127B) | hram array |
| 0xFFFF | IE (Interrupt Enable) | ie_register |

### I/O Register Routing in Interconnect

The interconnect routes I/O registers to appropriate subsystems:

```rust
// Read routing
0xFF00 => self.joypad.read(),           // Joypad
0xFF04..=0xFF07 => self.timer.read(),   // Timer
0xFF0F => self.if_register | 0xE0,      // IF
0xFF46 => 0xFF,                         // DMA (write-only)
0xFF40..=0xFF4B => self.ppu.read(),     // PPU (excluding 0xFF46)

// Write routing
0xFF00 => self.joypad.write(data),      // Joypad
0xFF04..=0xFF07 => self.timer.write(),  // Timer
0xFF0F => self.if_register = data,      // IF
0xFF46 => self.oam_dma(data),           // DMA transfer
0xFF40..=0xFF4B => self.ppu.write(),    // PPU (excluding 0xFF46)
```

### Interrupt System

Interrupts are handled in `cpu.handle_interrupts()`:

1. Check if IME (Interrupt Master Enable) is set
2. Check IF & IE for pending interrupts
3. If interrupt pending: disable IME, push PC, jump to vector
4. Interrupt vectors: VBlank=0x40, STAT=0x48, Timer=0x50, Serial=0x58, Joypad=0x60

### OAM DMA Transfer

When 0xFF46 is written, `oam_dma()` copies 160 bytes:
- Source: (written_value << 8) to (written_value << 8) + 0x9F
- Destination: 0xFE00-0xFE9F (OAM)

This is essential for sprite rendering - games use DMA to update sprite data.

## What's Missing

- **Audio (APU)** - Sound channels not implemented
- **MBC2/MBC3/MBC5** - Only MBC0/MBC1 supported
- **Save files (.sav)** - Battery-backed RAM not persisted
- **Serial I/O** - Link cable not implemented
- **STAT interrupts** - LCD STAT interrupt sources

## Testing

Tests are inline `#[cfg(test)]` modules. Key test files:
- `cpu.rs` - Opcode execution, CB-prefix opcodes
- `timer.rs` - Timer counting, overflow, clock select
- `ppu.rs` - Mode transitions, scanline timing
- `joypad.rs` - Button state, selection logic
- `interconnect.rs` - Memory routing, DMA

## Development Notes

- Rust 2021 edition
- `--release` recommended for playable speed
- Frame timing: 70224 cycles per frame (~60 FPS)
- Opcode enum uses snake_case (compiler warnings expected)

## Key References

- [Pan Docs](https://gbdev.io/pandocs/) - Comprehensive GB technical reference
- [Game Boy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
- [Codeslinger's GB Tutorial](http://www.codeslinger.co.uk/pages/projects/gameboy.html)
