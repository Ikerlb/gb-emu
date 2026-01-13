# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build the project
cargo build

# Run with a ROM file
cargo run -- --file-path "Tetris (World).gb"

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

# Release build
cargo build --release
```

## Architecture Overview

This is a Game Boy emulator written in Rust, currently ~5% complete. The architecture follows a modular design:

```
GameBoy (orchestrator)
├── CPU (Sharp LR35902)
│   ├── Registers: PC, SP, AF, BC, DE, HL
│   └── Flags: Z, N, H, C
└── Interconnect (memory bus)
    └── Cartridge (ROM/RAM + MBC)
```

### Key Modules

- **`src/gb/gameboy.rs`** - Top-level emulation loop, orchestrates CPU and memory
- **`src/gb/cpu.rs`** - CPU implementation with opcode execution (~8 of 256+ opcodes implemented)
- **`src/gb/register.rs`** - 16-bit register pair abstraction (hi/lo byte access)
- **`src/gb/opcode.rs`** - Opcode enum definitions using `num-derive` for conversions
- **`src/gb/interconnect.rs`** - Memory bus routing to appropriate hardware components
- **`src/gb/cartridge.rs`** - ROM loading, RAM allocation, and Memory Bank Controller (MBC0, MBC1 implemented; MBC2, MBC3 stubbed)
- **`src/gb/debug.rs`** - Debug configuration (DebugConfig struct with CLI flag settings)
- **`src/gb/debugger.rs`** - Interactive debugger with REPL, breakpoints, vi keybindings

### Interactive Debugger

Run with `--interactive` or `-i` to enter the debugger. Commands:

| Command | Alias | Description |
|---------|-------|-------------|
| `step` | `s` | Execute one instruction |
| `continue` | `c` | Run until breakpoint or halt |
| `break <addr>` | `b` | Set breakpoint (e.g., `b 0x150`) |
| `delete <id>` | `d` | Remove breakpoint by ID |
| `list` | `l` | Show all breakpoints |
| `reg` | `r` | Show CPU registers |
| `mem <range>` | `m` | Dump memory (e.g., `m 0x0000:0x00FF`) |
| `help` | `h` | Show help |
| `quit` | `q` | Exit |

Features: Vi keybindings (ESC for normal mode), tab completion, command hints, persistent history (~/.gb_emu_history), empty Enter repeats last command.

### Memory Map Status

Currently implemented:
- `0x0000-0x7FFF` - Cartridge ROM (via Cartridge)
- `0xA000-0xBFFF` - Cartridge External RAM (via Cartridge)

Not yet implemented (will panic):
- `0x8000-0x9FFF` - VRAM
- `0xC000-0xDFFF` - Work RAM
- `0xFE00-0xFE9F` - OAM
- `0xFF00-0xFF7F` - I/O registers
- `0xFF80-0xFFFE` - High RAM
- `0xFFFF` - Interrupt Enable

### Testing

Tests are inline `#[cfg(test)]` modules within each source file. Key test areas:
- CPU initialization and opcode execution
- Register byte ordering (big-endian for hi/lo)
- Cartridge MBC detection and ROM/RAM size parsing
- Interconnect memory reads with boundary conditions

## Development Notes

- The project uses Rust 2021 edition
- Opcode enum variants currently use snake_case (e.g., `Ld_Bc_d16`) - there are compiler warnings about this
- CPU execution returns cycle counts, though timing is not yet cycle-accurate
- `ROADMAP.md` contains a detailed 12-phase development plan
- `TODO.md` tracks known technical debt

## Key References

- [Pan Docs](https://gbdev.io/pandocs/) - Comprehensive GB technical reference
- [Game Boy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
- [Codeslinger's GB Tutorial](http://www.codeslinger.co.uk/pages/projects/gameboy.html)
