# TODO List

This document tracks known issues and planned improvements in the codebase.

## High Priority

### CPU (`src/gb/cpu.rs`)
- **CB-prefix opcodes**: Implement all ~256 extended opcodes (BIT, SET, RES, rotates, shifts, SWAP)
- **Interrupt handling**: Wire up IME flag, add interrupt checking in execution loop
- **EI/DI**: Currently stubbed - need to actually enable/disable interrupts
- **HALT/STOP**: Return cycle counts but need proper implementation

### PPU (`src/gb/ppu.rs`)
- **Pixel rendering**: Add framebuffer and actual tile/sprite drawing
- **Background rendering**: Parse tile maps and tile data from VRAM
- **Sprite rendering**: Parse OAM, handle 8x8 and 8x16 sprites
- **Display output**: Integrate with minifb/pixels/SDL2
- **Interrupt generation**: Fire interrupts on mode transitions and LYC match

### Timer (NEW FILE NEEDED: `src/gb/timer.rs`)
- **DIV register** (0xFF04): Increments at 16384 Hz
- **TIMA register** (0xFF05): Programmable timer counter
- **TMA register** (0xFF06): Timer modulo (reload value)
- **TAC register** (0xFF07): Timer control

### Cartridge (`src/gb/cartridge.rs`)
- **Line 127-132**: Potential banking bug in `read()` - addresses 0x0000-0x3FFF should always read from ROM bank 0
- **MBC3**: Implement write operations (currently stubbed)

## Medium Priority

### Code Quality
- Consider refactoring CPU instruction implementations to separate functions
- Review and optimize integer types to minimize casting

## Low Priority

### Register (`src/gb/register.rs`)
- Decision: Keep private attributes for safety (current design is fine)

## Completed ✓

- ✅ Migrated to Rust 2021 edition
- ✅ Updated dependencies (num-traits, num-derive)
- ✅ Fixed Register byte ordering bug
- ✅ Added comprehensive test coverage
- ✅ Created README.md and ROADMAP.md
- ✅ Fixed deprecated range patterns
- ✅ **Full memory map** (VRAM, WRAM, HRAM, OAM, I/O stubs, Echo RAM)
- ✅ **245+ CPU opcodes** (all standard opcodes except CB-prefix)
- ✅ **PPU timing framework** (modes, scanlines, all registers)
- ✅ **TUI debugger** (breakpoints, stepping, memory viewer)

## Next Steps (Priority Order)

1. **CB-prefix opcodes** - Required by most games (3-5 days)
2. **PPU rendering** - Get pixels on screen (1-2 weeks)
3. **Timer subsystem** - Simple but required (2-3 days)
4. **Interrupt handling** - Wire up IME and service routines (2-3 days)
5. **Input handling** - Joypad register (2-3 days)
6. **APU** - Audio (optional, 2-3 weeks)

**Path to Tetris:** CB opcodes → PPU rendering → Timer → Interrupts → Input

---

**Note:** The emulator is ~35% complete. Main gaps are CB opcodes, actual graphics rendering, timer, and interrupt logic.
