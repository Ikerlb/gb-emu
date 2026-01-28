# Development Plan

Current development priorities for the Game Boy emulator.

## Milestone: Tetris Playable - ACHIEVED

---

## Phase 1: CB-Prefix Opcodes - COMPLETE

All 256 CB-prefixed extended opcodes implemented in `src/gb/cpu.rs`.

- [x] RLC, RRC, RL, RR (rotates)
- [x] SLA, SRA, SRL (shifts)
- [x] SWAP (nibble swap)
- [x] BIT, RES, SET (bit operations)

## Phase 2: PPU Rendering - COMPLETE

Full scanline-based rendering implemented in `src/gb/ppu.rs`.

- [x] 160x144 framebuffer with minifb display
- [x] Background rendering with tile maps and scrolling (SCX/SCY)
- [x] Window layer rendering (WX, WY)
- [x] Sprite rendering (8x8 and 8x16, priority, flipping)
- [x] DMG green color palette (BGP, OBP0, OBP1)

## Phase 3: Timer - COMPLETE

Timer subsystem implemented in `src/gb/timer.rs`.

- [x] DIV register (0xFF04) - increments every 256 cycles
- [x] TIMA register (0xFF05) - programmable counter
- [x] TMA register (0xFF06) - reload value
- [x] TAC register (0xFF07) - clock select and enable
- [x] Timer overflow interrupt generation

## Phase 4: Interrupt System - COMPLETE

Interrupt handling implemented in `src/gb/cpu.rs`.

- [x] IME (Interrupt Master Enable) flag
- [x] EI/DI/RETI instructions
- [x] HALT instruction
- [x] VBlank interrupt on frame completion
- [x] Timer interrupt on TIMA overflow
- [x] IF/IE register handling

## Phase 5: Input - COMPLETE

Joypad implemented in `src/gb/joypad.rs`.

- [x] P1/JOYP register (0xFF00) with selection logic
- [x] Keyboard mapping (arrows, Z, X, Enter, Backspace)
- [x] Integration with minifb window

## Phase 6: OAM DMA - COMPLETE

OAM DMA transfer implemented in `src/gb/interconnect.rs`.

- [x] 0xFF46 write triggers 160-byte copy to OAM
- [x] Essential for sprite rendering

---

## Future Phases

### Phase 7: Audio (APU)

**Status:** Not Started

The Game Boy has 4 sound channels:

- [ ] Channel 1: Square wave with sweep
- [ ] Channel 2: Square wave
- [ ] Channel 3: Custom waveform
- [ ] Channel 4: Noise
- [ ] Audio registers (0xFF10-0xFF3F)
- [ ] Audio output via cpal or rodio crate

### Phase 8: Additional MBCs

**Status:** Not Started

For broader game compatibility:

- [ ] MBC2 - Simple mapper with built-in RAM
- [ ] MBC3 - Real-time clock support (Pokemon Gold/Silver)
- [ ] MBC5 - Larger ROM support, rumble

### Phase 9: Save Files

**Status:** Not Started

- [ ] Detect battery-backed cartridges
- [ ] Save external RAM to .sav file on exit
- [ ] Load .sav file on startup
- [ ] Auto-save periodically

### Phase 10: Polish

**Status:** Not Started

- [ ] Configurable key bindings
- [ ] Fullscreen toggle
- [ ] Fast forward (hold key for 2x-4x speed)
- [ ] Pause/resume
- [ ] Save states
- [ ] Screenshot capture

### Phase 11: Accuracy

**Status:** Not Started

- [ ] Run Blargg's CPU test ROMs
- [ ] Run Blargg's timing test ROMs
- [ ] STAT interrupt sources (mode changes, LYC=LY)
- [ ] Pixel FIFO for more accurate PPU timing
- [ ] Cycle-accurate instruction timing

---

## Known Issues

- Some games may have graphical glitches (need more testing)
- No audio
- Only MBC0/MBC1 cartridges supported

## Test ROMs to Try

- [x] Tetris (World) - Works!
- [ ] Dr. Mario
- [ ] Super Mario Land
- [ ] Kirby's Dream Land
- [ ] Pokemon Red/Blue (needs MBC3)
- [ ] Blargg's cpu_instrs
- [ ] Blargg's instr_timing
