# TODO List

This document tracks known issues and planned improvements in the codebase.

## High Priority

### CPU (`src/gb/cpu.rs`)
- **Line 149**: Implement DEC_H opcode (0x25) - Currently returns hardcoded 4 cycles
- **Line 131**: Consider refactoring: Move instruction implementations to separate functions for better organization
- **Line 138**: Improve opcode enum usage to avoid redundant `Opcode::` prefixes

### Memory (`src/gb/interconnect.rs`)
- **Line 17**: Complete memory map implementation
  - Add VRAM (0x8000-0x9FFF)
  - Add WRAM (0xC000-0xDFFF)
  - Add OAM (0xFE00-0xFE9F)
  - Add I/O registers (0xFF00-0xFF7F)
  - Add HRAM (0xFF80-0xFFFE)
- **Line 31**: Complete write handling for all memory regions

### Cartridge (`src/gb/cartridge.rs`)
- **Line 164**: Implement MBC3 write operations (currently stubbed)
- **Line 126**: Thoroughly test cartridge read/write operations
- **Line 1**: Review and optimize integer types to minimize casting

## Low Priority

### Register (`src/gb/register.rs`)
- **Line 1**: Decision needed: Should register attributes be public?
  - Current design uses getter/setter methods (encapsulation)
  - Making public would allow direct access (less safe but potentially faster)
  - Recommendation: Keep private for safety

## Completed ✓

- ✅ Migrated to Rust 2021 edition
- ✅ Updated dependencies (num-traits, num-derive)
- ✅ Fixed Register byte ordering bug in new() and set() methods
- ✅ Added comprehensive test coverage for CPU, Interconnect, Cartridge, and Register
- ✅ Created README.md and ROADMAP.md
- ✅ Fixed deprecated range patterns throughout codebase

## Next Steps (from ROADMAP.md)

1. **Complete CPU opcodes** - Implement remaining 248+ opcodes
2. **Implement Timer** - Add timer subsystem (relatively simple)
3. **Implement Interrupts** - Critical for game execution
4. **Implement PPU** - Graphics rendering (largest component)
5. **Add Input** - Joypad handling
6. **Implement APU** - Audio (optional for initial playability)

---

**Note:** Many TODOs will be addressed as part of the roadmap implementation. This list focuses on technical debt and immediate improvements to existing code.
