# Rust Game Boy Emulator

A Game Boy (DMG) emulator written in Rust, aiming for accuracy and clean architecture.

## Project Status

⚠️ **Early Development** - This project is in very early stages (~5% complete). Currently, the emulator can:
- Load ROM files
- Execute basic CPU opcodes (~8 out of 256+)
- Handle cartridge memory banking (MBC1 partially implemented)

### What's Implemented

- ✅ Basic CPU structure (Sharp LR35902)
- ✅ Register management
- ✅ Cartridge loading & ROM parsing
- ✅ Memory Bank Controllers (MBC0, MBC1 mostly complete, MBC2/3 partial)
- ✅ Basic interconnect (memory bus)
- ✅ ~8 opcodes (NOP, DEC, CPL, JP, LD variants)

### What's Missing

- ❌ GPU/PPU (graphics rendering)
- ❌ APU (audio processing)
- ❌ Input handling
- ❌ Timers
- ❌ Interrupts
- ❌ Most CPU opcodes (~97% remaining)
- ❌ Complete memory map
- ❌ Serial I/O
- ❌ DMA transfers

## Architecture

The emulator follows a modular component-based architecture:

```
GameBoy (Top-level orchestrator)
    ├── CPU (Sharp LR35902 processor)
    │   ├── Registers (PC, SP, AF, BC, DE, HL)
    │   ├── Flags (Z, N, H, C)
    │   └── Opcode executor
    └── Interconnect (Memory bus)
        └── Cartridge (ROM/RAM + MBC)
```

### Key Components

- **`GameBoy`** (`src/gb/gameboy.rs`) - Main emulator struct, execution loop
- **`CPU`** (`src/gb/cpu.rs`) - CPU implementation with register management
- **`Register`** (`src/gb/register.rs`) - 16-bit register abstraction
- **`Opcode`** (`src/gb/opcode.rs`) - Enum-based opcode definitions
- **`Interconnect`** (`src/gb/interconnect.rs`) - Memory bus routing
- **`Cartridge`** (`src/gb/cartridge.rs`) - ROM/RAM and bank switching

## Building

Requires Rust 1.56+ (uses 2021 edition)

```bash
cargo build
```

Run tests:
```bash
cargo test
```

## Running

```bash
cargo run -- path/to/rom.gb
```

Example with the included Tetris ROM:
```bash
cargo run -- "Tetris (World).gb"
```

**Note:** Currently the emulator will only execute a few opcodes before hitting `unimplemented!()` panics. This is expected given the early development stage.

## Development

### Dependencies

- `num-traits` - Numeric trait abstractions
- `num-derive` - Derive macros for numeric conversions

### Recent Updates

- ✨ Migrated to Rust 2021 edition
- ✨ Updated dependencies to modern versions
- ✨ Fixed deprecated range patterns (`...` → `..=`)
- ✨ Modernized module imports

### Roadmap

See [ROADMAP.md](ROADMAP.md) for the detailed development plan.

## Resources

Useful references for Game Boy emulator development:
- [Pan Docs](https://gbdev.io/pandocs/) - Comprehensive GB technical reference
- [Game Boy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
- [Codeslinger's GB Tutorial](http://www.codeslinger.co.uk/ps-emu/gameboy/gameboy.html)
- [/r/EmuDev](https://www.reddit.com/r/EmuDev/)

## License

Dual licensed under MIT or Apache-2.0, at your option.

## Contributing

This is a personal learning project currently in early development. The codebase will undergo significant changes as core functionality is implemented.
