# Game Boy Emulator - Development Roadmap

This document outlines the development plan to bring the emulator from its current ~5% completion to a fully functional state.

## Overview

Building a complete Game Boy emulator is a substantial undertaking. This roadmap breaks the work into manageable phases, prioritizing components that provide the most visible progress early on.

---

## Phase 1: Foundation & Testing ✅ (COMPLETED)

**Goal:** Modernize codebase and establish testing infrastructure

- [x] Migrate to Rust 2021 edition
- [x] Update dependencies to modern versions
- [x] Fix deprecated syntax (range patterns, module imports)
- [x] Create README and documentation
- [x] Add comprehensive unit tests for existing components
- [x] Fix critical Register byte ordering bug

**Estimated Effort:** 2-3 days

---

## Phase 2: Debug & Visualization Infrastructure ⚡ **HIGH PRIORITY**

**Goal:** Build comprehensive debugging tools to visualize emulator state

This phase is critical for efficient development. Without good debugging tools, tracking down issues in CPU opcodes, memory operations, and PPU rendering becomes exponentially harder.

### Core Debugging Features

**2.1 State Display System**
- CPU state viewer:
  - All registers (PC, SP, AF, BC, DE, HL) with hex/decimal display
  - All flags (Z, N, H, C) with visual indicators
  - Current instruction and next few instructions (disassembly)
  - Cycle counter
- Memory viewer:
  - Configurable memory window (address range selector)
  - Hex dump with ASCII representation
  - Highlighting for different regions (ROM, RAM, VRAM, etc.)
  - Watch specific addresses
- Execution state:
  - Current opcode being executed
  - Instruction history (last N instructions)
  - Call stack visualization

**2.2 Interactive Debugger**
- Step execution (step-into, step-over)
- Breakpoints:
  - Address breakpoints (break at PC = 0xXXXX)
  - Memory breakpoints (break on read/write to address)
  - Conditional breakpoints
- Run/pause/reset controls
- Save/load emulator state (snapshots)

**2.3 Logging System**
- Configurable log levels (TRACE, DEBUG, INFO, WARN, ERROR)
- Per-component logging (CPU, Memory, Cartridge, etc.)
- Log to file and/or console
- Instruction trace mode (logs every instruction executed)

**2.4 Visual Diff Tools**
- Compare emulator state against known-good state
- Useful for comparing against test ROM expected outputs
- Register diff viewer
- Memory diff viewer

### Implementation Strategy

**Phase 2A: Basic State Display (1-2 days) ✅ COMPLETED**
1. ✅ Implement `Display` traits for all stateful structs
2. ✅ Create debug print functions for CPU, memory, flags
3. ✅ Add `--debug` flag to show state after each instruction
4. ✅ Add `--verbose` flag for multi-line debug format
5. ✅ Add `--max-instructions` flag for limiting execution

**Phase 2B: Memory Viewer (1 day)** ⬅️ **NEXT**
1. Create memory dump function
2. Add address range viewer
3. Implement hex dump formatting

**Phase 2C: Interactive Debugger (2-3 days)**
1. Add step-by-step execution mode
2. Implement breakpoint system
3. Create simple REPL interface for debugging commands

**Phase 2D: Logging Infrastructure (1 day)**
1. Integrate `env_logger` or `tracing` crate
2. Add log statements throughout codebase
3. Create instruction trace mode

**Phase 2E: Advanced Features (Optional, 1-2 days)**
1. TUI (Terminal UI) using `ratatui` for better visualization
2. Save/load state functionality
3. Compare with reference implementation

### Benefits

- **Faster Development:** Immediately see what's wrong when a test fails
- **Better Understanding:** Visualize how the emulator works internally
- **Easier Testing:** Compare state with known-good emulators
- **Learning Tool:** See exactly how Game Boy hardware behaves

### Dependencies to Add

```toml
env_logger = "0.11"  # or tracing = "0.1"
log = "0.4"
# Optional:
ratatui = "0.26"  # For TUI
crossterm = "0.27"  # Terminal manipulation
```

### Example Usage

```bash
# Run with instruction trace
cargo run --release -- rom.gb --trace

# Run with debugger
cargo run --release -- rom.gb --debug

# Run with breakpoint
cargo run --release -- rom.gb --break 0x0150

# Dump memory range
cargo run --release -- rom.gb --dump-mem 0x8000:0x9FFF
```

**Estimated Effort:** 5-7 days

**This phase should be completed BEFORE extensive CPU opcode implementation!**

---

## Phase 3: CPU Completion (Critical Path)

**Goal:** Implement all 256+ CPU opcodes

### Priority 1: Most Common Opcodes (~50 opcodes)
The most frequently used instructions that appear in simple ROMs:

**Load Instructions (LD)**
- LD r, r' (8-bit register to register) - ~40 variations
- LD r, n (immediate 8-bit load)
- LD r, (HL) (load from memory)
- LD (HL), r (store to memory)
- LD A, (BC/DE) (load A from BC/DE pointer)
- LD (BC/DE), A (store A to BC/DE pointer)
- LD r, (nn) (load from 16-bit address)
- LD (nn), r (store to 16-bit address)
- LD rr, nn (16-bit immediate loads)
- PUSH/POP (stack operations)

**Arithmetic/Logic**
- ADD, ADC, SUB, SBC (8-bit arithmetic)
- AND, OR, XOR (logic operations)
- INC, DEC (8-bit and 16-bit)
- CP (compare)

**Control Flow**
- JP, JR (jumps)
- CALL, RET (subroutines)
- Conditional jumps/calls (JZ, JNZ, JC, JNC)
- RST (restart vectors)

**Bit Operations**
- RLCA, RLA, RRCA, RRA (rotates)
- DAA (decimal adjust)

### Priority 2: Extended Instructions (CB prefix)
- BIT, SET, RES (bit test/set/reset)
- RLC, RL, RRC, RR (rotates)
- SLA, SRA, SRL (shifts)
- SWAP

### Priority 3: Remaining Opcodes
- Interrupts (RETI, DI, EI)
- Special (HALT, STOP)
- 16-bit arithmetic (ADD HL, ...)

**Testing Strategy:**
- Unit test each opcode individually
- Test flag behavior (Z, N, H, C)
- Verify cycle counts
- Run Blargg's CPU test ROMs

**Estimated Effort:** 2-3 weeks

**Key Files to Modify:**
- `src/gb/cpu.rs` - Add opcode implementations
- `src/gb/opcode.rs` - Add opcode enum variants

---

## Phase 4: Memory Map Completion

**Goal:** Implement full Game Boy memory map

Current state: Only cartridge regions (0x0000-0x7FFF, 0xA000-0xBFFF) implemented

**Memory Regions to Add:**

```
0x0000-0x3FFF : ROM Bank 0 (fixed) ✅
0x4000-0x7FFF : ROM Bank N (switchable) ✅
0x8000-0x9FFF : VRAM (Video RAM) ❌
0xA000-0xBFFF : External RAM ✅
0xC000-0xDFFF : Work RAM (WRAM) ❌
0xE000-0xFDFF : Echo RAM (mirror of C000-DDFF) ❌
0xFE00-0xFE9F : OAM (Sprite Attribute Table) ❌
0xFEA0-0xFEFF : Unusable ❌
0xFF00-0xFF7F : I/O Registers ❌
0xFF80-0xFFFE : High RAM (HRAM) ❌
0xFFFF        : Interrupt Enable Register ❌
```

**Implementation Steps:**
1. Add WRAM (8KB) - Simple array
2. Add HRAM (127 bytes) - Simple array
3. Add OAM (160 bytes) - Will be used by PPU
4. Add VRAM (8KB) - Will be used by PPU
5. Implement Echo RAM (just mirror WRAM)
6. Stub out I/O registers (return 0xFF for now)

**Testing:**
- Memory read/write tests for each region
- Verify mirror behavior for Echo RAM
- Test boundary conditions

**Estimated Effort:** 3-4 days

**Key Files to Modify:**
- `src/gb/interconnect.rs` - Route memory accesses

---

## Phase 5: Timer Implementation

**Goal:** Implement the 4 timer-related registers

Timers are relatively simple but required for many games.

**Registers:**
- `0xFF04` DIV - Divider Register (increments at 16384 Hz)
- `0xFF05` TIMA - Timer Counter (programmable frequency)
- `0xFF06` TMA - Timer Modulo (reload value)
- `0xFF07` TAC - Timer Control

**Implementation:**
1. Create `Timer` struct
2. Track cycles and update timers
3. Generate timer interrupts when TIMA overflows
4. Integrate with CPU execution loop

**Testing:**
- Verify timer frequencies
- Test interrupt generation
- Run timer test ROMs

**Estimated Effort:** 2-3 days

**New Files:**
- `src/gb/timer.rs`

---

## Phase 6: Interrupt System

**Goal:** Implement interrupt handling mechanism

**Interrupt Types:**
- V-Blank (bit 0) - Most important
- LCD STAT (bit 1)
- Timer (bit 2)
- Serial (bit 3)
- Joypad (bit 4)

**Registers:**
- `0xFF0F` IF - Interrupt Flags
- `0xFFFF` IE - Interrupt Enable

**Implementation:**
1. Add interrupt flags to Interconnect
2. Implement IME (Interrupt Master Enable) in CPU
3. Implement EI, DI, RETI opcodes
4. Add interrupt check in CPU execution loop
5. Implement interrupt service routine call

**Testing:**
- Test each interrupt type
- Verify interrupt priority
- Test nested interrupts
- Test IME timing

**Estimated Effort:** 3-4 days

**Key Files to Modify:**
- `src/gb/cpu.rs` - Add IME, interrupt handling
- `src/gb/interconnect.rs` - Add IF register

---

## Phase 7: PPU (Graphics) - The Big One

**Goal:** Implement pixel processing unit for graphics

This is the largest and most complex component. The PPU has 4 modes and runs in parallel with the CPU.

### PPU Modes
- Mode 0: H-Blank
- Mode 1: V-Blank
- Mode 2: OAM Search
- Mode 3: Pixel Transfer

### Components

**6.1 Basic PPU Structure**
- Create PPU struct
- Implement mode switching
- Track scanline (LY register)
- Implement LY comparison (LYC)

**6.2 Background Rendering**
- Parse background tile map (9800-9BFF or 9C00-9FFF)
- Parse background tile data (8000-8FFF or 8800-97FF)
- Implement scrolling (SCX, SCY)
- Render background to framebuffer

**6.3 Window Rendering**
- Implement window layer
- Handle WX, WY registers
- Window priority over background

**6.4 Sprite (OBJ) Rendering**
- Parse OAM (sprite attribute table)
- Implement sprite rendering (8x8 and 8x16)
- Handle sprite priority and transparency
- Implement sprite limit (10 per scanline)

**6.5 PPU Registers**
Implement all LCD control registers:
- `0xFF40` LCDC - LCD Control
- `0xFF41` STAT - LCD Status
- `0xFF42` SCY - Scroll Y
- `0xFF43` SCX - Scroll X
- `0xFF44` LY - LCD Y Coordinate
- `0xFF45` LYC - LY Compare
- `0xFF47` BGP - BG Palette
- `0xFF48` OBP0 - OBJ Palette 0
- `0xFF49` OBP1 - OBJ Palette 1
- `0xFF4A` WY - Window Y
- `0xFF4B` WX - Window X

**6.6 Display Output**
- Create framebuffer (160x144)
- Integrate with a display library (e.g., `minifb`, `pixels`, or SDL2)
- Implement V-Blank timing

**Testing Strategy:**
1. Start with solid color background
2. Test single tile rendering
3. Test full background with scrolling
4. Add sprite rendering
5. Test priority and transparency
6. Run graphical test ROMs (dmg-acid2, etc.)

**Estimated Effort:** 3-4 weeks

**New Files:**
- `src/gb/ppu.rs` - Main PPU implementation
- `src/gb/display.rs` - Display/framebuffer interface

**Dependencies to Add:**
- Display library (minifb, pixels, or sdl2)

---

## Phase 8: Input Handling

**Goal:** Implement joypad input

**Buttons:**
- D-pad: Up, Down, Left, Right
- Action: A, B, Start, Select

**Register:**
- `0xFF00` P1/JOYP - Joypad register

**Implementation:**
1. Create input mapping system
2. Implement P1 register read/write
3. Generate joypad interrupts
4. Integrate with display library's input

**Testing:**
- Test button presses
- Test interrupt generation
- Play a game!

**Estimated Effort:** 2-3 days

**Key Files to Modify:**
- `src/gb/interconnect.rs` - Add P1 register
- `src/main.rs` - Input handling

---

## Phase 9: APU (Audio) - Optional

**Goal:** Implement audio processing unit

Audio is complex but not required for basic playability.

**Channels:**
- Channel 1: Pulse with sweep
- Channel 2: Pulse
- Channel 3: Wave
- Channel 4: Noise

**Registers:** 0xFF10-0xFF26 (23 registers)

**Implementation:**
1. Implement each sound channel
2. Implement audio mixing
3. Integrate with audio library (cpal, SDL2 audio)
4. Implement frame sequencer

**Estimated Effort:** 2-3 weeks

**New Files:**
- `src/gb/apu.rs`

---

## Phase 10: Advanced Features

**Goal:** Handle edge cases and advanced functionality

**9.1 Remaining MBC Implementation**
- Complete MBC2 (RAM is only 512x4 bits)
- Complete MBC3 (Real-Time Clock support)
- Add MBC5 support (used by later games)

**9.2 DMA Transfer**
- Implement OAM DMA (0xFF46)
- Implement HDMA for GBC (optional)

**9.3 Serial I/O**
- Basic serial implementation (0xFF01, 0xFF02)
- Not required for most games

**Estimated Effort:** 1 week

---

## Phase 11: Testing & Accuracy

**Goal:** Improve accuracy and compatibility

**Test ROM Suites:**
1. **Blargg's Test ROMs** (most important)
   - cpu_instrs - Tests all instructions
   - instr_timing - Tests instruction timing
   - mem_timing - Tests memory timing
   - dmg_sound - Audio tests

2. **Mooneye Test Suite**
   - Comprehensive accuracy tests
   - Timing tests

3. **dmg-acid2**
   - Visual rendering test

4. **Real Games**
   - Tetris (simplest)
   - Dr. Mario
   - Super Mario Land
   - Pokemon Red/Blue
   - The Legend of Zelda: Link's Awakening

**Debugging Tools:**
- Add debugger with breakpoints
- Add memory viewer
- Add CPU state display
- Add PPU debug viewer

**Estimated Effort:** Ongoing

---

## Phase 12: Performance & Polish

**Goal:** Optimize and add quality-of-life features

**Performance:**
- Profile hot paths
- Optimize PPU rendering
- Consider JIT compilation (advanced)

**Features:**
- Save states
- Fast-forward
- Screenshot capability
- Configurable key bindings
- Game Boy Color support (major undertaking)

**Estimated Effort:** Variable

---

## Summary Timeline

| Phase | Component | Estimated Time |
|-------|-----------|----------------|
| 1 | Foundation & Testing | 2-3 days ✅ |
| 2 | Debug & Visualization | 5-7 days (2A ✅, 2B-2E pending) |
| 3 | CPU Completion | 2-3 weeks |
| 4 | Memory Map | 3-4 days |
| 5 | Timer | 2-3 days |
| 6 | Interrupts | 3-4 days |
| 7 | PPU (Graphics) | 3-4 weeks |
| 8 | Input | 2-3 days |
| 9 | APU (Audio) | 2-3 weeks (optional) |
| 10 | Advanced Features | 1 week |
| 11 | Testing & Accuracy | Ongoing |
| 12 | Polish | Variable |

**Total Estimated Time (without audio):** 11-15 weeks of focused development

**Total Estimated Time (with audio):** 13-18 weeks

**Note:** Phase 2 (Debug & Visualization) is a high-value investment that will save significant debugging time in later phases.

---

## Quick Wins for Motivation

To see visible progress quickly, consider this alternative order:

1. ✅ **Foundation** (done!)
2. ✅ **Basic Debug Tools** - State display (Phase 2A done!)
3. ⚡ **Memory Map** - Quick implementation (Phase 4)
4. **Common CPU Opcodes** - Just enough to run simple code (with debug tools to verify!)
5. **Basic PPU** - Get pixels on screen ASAP (background only, no sprites)
6. **Timer** - Simple and satisfying
7. **Interrupts** - Unlocks V-Blank
8. **Full PPU** - Sprites and polish
9. **Complete CPU** - Fill in remaining opcodes
10. **Input** - Make it playable!
11. **Audio & Polish**

This approach prioritizes **visible progress** AND **developer efficiency** - you'll have debugging tools from the start and see pixels on screen sooner!

**Recommended Next Step:** Start with Phase 2A (Basic State Display) - just 1-2 days of work that will make everything else 10x easier.

---

## Recommended Resources

**Test ROMs:**
- [Blargg's Test ROMs](https://github.com/retrio/gb-test-roms)
- [Mooneye Test Suite](https://github.com/Gekkio/mooneye-test-suite)

**Documentation:**
- [Pan Docs](https://gbdev.io/pandocs/) - The definitive GB reference
- [GBEDG](https://hacktix.github.io/GBEDG/) - Game Boy Emulator Development Guide
- [The Cycle-Accurate Game Boy Docs](https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf)

**Other Emulators (for reference):**
- [mooneye-gb](https://github.com/Gekkio/mooneye-gb) - Rust, accuracy-focused
- [SameBoy](https://github.com/LIJI32/SameBoy) - C, very accurate
- [BGB](https://bgb.bircd.org/) - Excellent debugger

**Community:**
- [/r/EmuDev](https://www.reddit.com/r/EmuDev/)
- [EmuDev Discord](https://discord.gg/dkmJAes)
- [GBDev Discord](https://discord.gg/gbdev)

---

## Current Priority

**Phase 2A Complete! ✅** Basic debug infrastructure is in place.

**Current Phase: Phase 2B - Memory Viewer** ⬅️ **START HERE**

**What's been done (Phase 2A):**
- ✅ `--debug` flag shows CPU state after each instruction
- ✅ `--verbose` flag for multi-line debug format
- ✅ `--max-instructions` flag for limiting execution
- ✅ Pretty-printing for registers and flags

**Next Steps (Phase 2B):**
1. Create memory dump function
2. Add address range viewer (e.g., `--dump-mem 0x8000:0x9FFF`)
3. Implement hex dump formatting with ASCII representation

**After that (Phase 2C):**
- Step-by-step execution mode
- Breakpoint system
- Simple REPL interface for debugging commands

**Alternative Path:**
If you're eager to see more progress, you could jump to Phase 4 (Memory Map Completion) since it's relatively quick and unblocks CPU testing. The memory viewer can be added incrementally as needed.
