# MBC Implementation Analysis: Comparison of Game Boy Emulators in Rust

This document analyzes how different Rust Game Boy emulators handle Memory Bank Controllers (MBCs), based on studying four major projects: mohanson/gameboy, mvdnes/rboy, simias/gb-rs, and our current implementation.

## Table of Contents
1. [Architectural Approaches](#architectural-approaches)
2. [Detailed Implementation Analysis](#detailed-implementation-analysis)
3. [Critical Implementation Details](#critical-implementation-details)
4. [Issues in Current Implementation](#issues-in-current-implementation)
5. [Recommendations](#recommendations)

---

## Architectural Approaches

### 1. Trait-Based with Dynamic Dispatch (rboy)

**Structure:**
```rust
#[typetag::serde(tag = "type")]
pub trait MBC: Send {
    fn readrom(&self, a: u16) -> u8;
    fn readram(&self, a: u16) -> u8;
    fn writerom(&mut self, a: u16, v: u8);
    fn writeram(&mut self, a: u16, v: u8);
    fn check_and_reset_ram_updated(&mut self) -> bool;
    fn is_battery_backed(&self) -> bool;
    fn loadram(&mut self, ramdata: &[u8]) -> Result<()>;
    fn dumpram(&self) -> Vec<u8>;
}

// In MMU:
mbc: Box<dyn MBC>
```

**Organization:**
- `src/mbc/mod.rs` - Trait definition and factory function
- `src/mbc/mbc0.rs` - MBC0 implementation
- `src/mbc/mbc1.rs` - MBC1 implementation
- `src/mbc/mbc2.rs` - MBC2 implementation
- `src/mbc/mbc3.rs` - MBC3 implementation
- `src/mbc/mbc5.rs` - MBC5 implementation

**Pros:**
- ✅ **Open/Closed Principle**: Add new MBC types without modifying existing code
- ✅ **Separation of Concerns**: Each MBC is ~200 lines in its own file
- ✅ **Clean MMU Interface**: MMU doesn't know about specific MBC implementations
- ✅ **Serialization Support**: Uses `typetag` for polymorphic serialization
- ✅ **Testability**: Each MBC can be tested independently

**Cons:**
- ❌ **Runtime Overhead**: Dynamic dispatch has minor performance cost
- ❌ **Heap Allocation**: Requires Box for trait object
- ❌ **Complexity**: More advanced Rust concepts (trait objects, lifetimes)

---

### 2. Multi-Trait Approach (mohanson/gameboy)

**Structure:**
```rust
trait Memory {
    fn get(&self, addr: u16) -> u8;
    fn set(&mut self, addr: u16, val: u8);
}

trait Stable {
    fn sav(&self) -> Vec<u8>;
}

trait Cartridge: Memory + Stable {
    fn title(&self) -> String { /* extract from ROM */ }
}

// Each MBC implements all three traits
struct Mbc1 { rom: Vec<u8>, ram: Vec<u8>, /* ... */ }
impl Memory for Mbc1 { /* ... */ }
impl Stable for Mbc1 { /* ... */ }
impl Cartridge for Mbc1 {}
```

**Key Insight**: Separates concerns into distinct traits
- `Memory`: Core read/write operations
- `Stable`: Persistence (save files)
- `Cartridge`: High-level metadata operations

**Pros:**
- ✅ **Clear Separation**: Each trait has single responsibility
- ✅ **Flexible Composition**: Can implement traits independently
- ✅ **Type Safety**: Compiler enforces trait bounds
- ✅ **Comprehensive**: Includes RTC, save file handling, validation

**Cons:**
- ❌ **More Boilerplate**: Must implement 3 traits per MBC
- ❌ **Still Dynamic Dispatch**: Uses trait objects internally
- ❌ **Learning Curve**: Complex trait system for newcomers

---

### 3. Function Pointer Approach (simias/gb-rs)

**Structure:**
```rust
pub struct Model {
    name: &'static str,
    write_rom: fn(&mut Cartridge, u16, u8),
    write_ram: fn(&mut Cartridge, u16, u8),
    read_ram: fn(&Cartridge, u16) -> u8,
}

pub struct Cartridge {
    model: Model,
    rom: Vec<u8>,
    ram: Vec<u8>,
    // ... state
}

// Usage:
(self.model.write_rom)(self, offset, val)
(self.model.read_ram)(self, addr)
```

**Pros:**
- ✅ **Zero Runtime Overhead**: Direct function calls, no vtable lookup
- ✅ **Simple Mental Model**: Easy to understand
- ✅ **Flexible**: Can swap functions at runtime if needed
- ✅ **Minimal Boilerplate**: Just define functions and struct

**Cons:**
- ❌ **Less Type-Safe**: Function pointers bypass trait system
- ❌ **Less Idiomatic Rust**: Traits are preferred pattern
- ❌ **Harder to Test**: Can't easily mock function pointers
- ❌ **No Compiler Guarantees**: Easy to forget to implement a function
- ❌ **Limited RTC Support**: Comments indicate "RTC unimplemented"

---

### 4. Enum-Based Dispatch (Current Implementation)

**Structure:**
```rust
enum MemoryBankController {
    Mbc0,
    Mbc1,
    Mbc2,
    Mbc3,
}

pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    mbc: MemoryBankController,
    current_rom: u16,
    current_ram: u16,
    enable_ram: bool,
    rom_mode: bool,
    // ... all state for all MBC types
}

impl Cartridge {
    pub fn write(&mut self, address: u16, data: u8) {
        match self.mbc {
            MemoryBankController::Mbc0 => panic!("..."),
            MemoryBankController::Mbc1 => self.write_mbc1(address, data),
            MemoryBankController::Mbc2 => self.write_mbc2(address, data),
            MemoryBankController::Mbc3 => self.write_mbc3(address, data),
        }
    }
}
```

**Pros:**
- ✅ **Simple**: Easiest to understand for beginners
- ✅ **Zero Overhead**: Compiler optimizes to direct calls
- ✅ **All in One Place**: Easy to see complete implementation
- ✅ **Pattern Matching**: Exhaustiveness checking

**Cons:**
- ❌ **Violates Open/Closed**: Must modify Cartridge to add MBC types
- ❌ **Large File**: All MBC logic in one file (grows to 1000+ lines)
- ❌ **Wasted Memory**: Struct contains state for ALL MBC types
- ❌ **Harder to Test**: Can't test MBC implementations in isolation
- ❌ **Code Duplication**: Similar logic repeated across match arms

---

## Detailed Implementation Analysis

### MBC1: Banking Modes and Quirks

MBC1 is the most complex because of its dual-mode system and bank selection quirks.

#### Key Concepts:

**Two Banking Modes:**
1. **ROM Banking Mode (mode=0)**: Access more ROM banks, limited RAM
   - 0x0000-0x3FFF reads from ROM bank 0
   - 0x4000-0x7FFF reads from selected bank (5 bits + 2 bits = 7 bits = 128 banks)
   - 0xA000-0xBFFF always accesses RAM bank 0

2. **RAM Banking Mode (mode=1)**: Limited ROM, access more RAM banks
   - 0x0000-0x3FFF reads from ROM bank (2 upper bits << 5)
   - 0x4000-0x7FFF reads from selected bank (5 bits only)
   - 0xA000-0xBFFF accesses selected RAM bank (2 bits = 4 banks)

**Bank Selection Quirks:**
- Writing 0x00 to ROM bank register → treat as 0x01
- Banks 0x20, 0x40, 0x60 can't be selected → wrap to 0x21, 0x41, 0x61

#### rboy MBC1 Implementation (Reference):

```rust
struct MBC1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rombank: u8,      // Lower 5 bits (0x2000-0x3FFF writes)
    rambank: u8,      // Upper 2 bits (0x4000-0x5FFF writes)
    banking_mode: u8, // 0 = ROM mode, 1 = RAM mode
    ram_on: bool,     // RAM enable flag
}

fn readrom(&self, a: u16) -> u8 {
    let i = match a {
        0x0000..=0x3FFF => {
            if self.banking_mode == 0 {
                a as usize  // Always bank 0 in ROM mode
            } else {
                // In RAM mode, use upper bits for bank 0 area
                ((self.rambank as usize) << 19) | (a as usize)
            }
        },
        0x4000..=0x7FFF => {
            // Combine lower 5 bits + upper 2 bits
            let bank = (self.rombank & 0x1F) | (self.rambank << 5);
            ((bank as usize) << 14) | ((a as usize) & 0x3FFF)
        },
        _ => panic!("Invalid address"),
    };
    self.rom[i % self.rom.len()]  // Wrap if ROM is smaller
}

fn writerom(&mut self, a: u16, v: u8) {
    match a {
        0x0000..=0x1FFF => {
            // RAM enable: only value 0x0A enables
            self.ram_on = (v & 0x0F) == 0x0A;
        },
        0x2000..=0x3FFF => {
            // ROM bank select (lower 5 bits)
            let mut bank = v & 0x1F;
            if bank == 0 { bank = 1; }  // Critical: 0 → 1
            self.rombank = bank;
        },
        0x4000..=0x5FFF => {
            // RAM bank / upper ROM bits (2 bits)
            self.rambank = v & 0x03;
        },
        0x6000..=0x7FFF => {
            // Banking mode select
            self.banking_mode = v & 0x01;
        },
        _ => {},
    }
}
```

**Key Insights:**
1. Bank 0 calculation differs by mode (lines 6-12 vs 14-18)
2. ROM bank combines two registers: `(rombank & 0x1F) | (rambank << 5)`
3. Bank 0 prevention is simple: `if bank == 0 { bank = 1; }`
4. Mode affects BOTH ROM reads and RAM access

---

### MBC3: Real-Time Clock Implementation

MBC3 adds RTC (Real-Time Clock) support, making it more complex than MBC1.

#### Key Concepts:

**RTC Registers (0x08-0x0C):**
- 0x08: Seconds (0-59)
- 0x09: Minutes (0-59)
- 0x0A: Hours (0-23)
- 0x0B: Day counter lower 8 bits
- 0x0C: Day counter upper bit + halt + carry flags

**RTC Features:**
- **Latching**: Write 0x00 then 0x01 to 0x6000-0x7FFF to freeze RTC values
- **Halt**: Bit 6 of register 0x0C stops the clock
- **Day Counter**: 9-bit counter (0-511 days)
- **Day Carry**: Bit 7 of register 0x0C indicates overflow

#### rboy MBC3 Implementation (Reference):

```rust
struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rombank: u8,
    rambank: u8,
    rtc_ram: [u8; 5],        // Current RTC values
    rtc_ram_latch: [u8; 5],  // Latched RTC values
    rtc_zero: u64,            // Epoch reference (Unix timestamp)
    selectrtc: bool,          // true = RTC selected, false = RAM
    latchstate: u8,           // Latch state machine
}

fn calc_rtc_reg(&self) -> [u8; 5] {
    let mut regs = [0u8; 5];

    // Check halt flag (bit 6 of register 4)
    if (self.rtc_ram[4] & 0x40) != 0 {
        return self.rtc_ram;  // Clock is halted
    }

    // Calculate elapsed time since epoch
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let elapsed = now.saturating_sub(self.rtc_zero);

    // Break down into time components
    let seconds = elapsed % 60;
    let minutes = (elapsed / 60) % 60;
    let hours = (elapsed / 3600) % 24;
    let days = elapsed / (3600 * 24);

    regs[0] = seconds as u8;
    regs[1] = minutes as u8;
    regs[2] = hours as u8;
    regs[3] = (days & 0xFF) as u8;           // Day lower 8 bits
    regs[4] = ((days >> 8) & 0x01) as u8;    // Day upper 1 bit

    // Handle day counter overflow (512 days)
    if days >= 512 {
        regs[4] |= 0x80;  // Set carry flag
    }

    regs
}

fn readram(&self, a: u16) -> u8 {
    if !self.ram_on { return 0xFF; }

    if self.selectrtc {
        // Reading RTC register: return latched value
        self.rtc_ram_latch[self.rambank as usize]
    } else {
        // Reading RAM
        let bank_offset = (self.rambank as usize) * 0x2000;
        let addr = (a as usize - 0xA000) + bank_offset;
        self.ram[addr % self.ram.len()]
    }
}

fn writerom(&mut self, a: u16, v: u8) {
    match a {
        0x0000..=0x1FFF => {
            self.ram_on = (v & 0x0F) == 0x0A;
        },
        0x2000..=0x3FFF => {
            // No bank 0 restriction for MBC3!
            let mut bank = v & 0x7F;
            if bank == 0 { bank = 1; }
            self.rombank = bank;
        },
        0x4000..=0x5FFF => {
            // 0x00-0x03: RAM banks
            // 0x08-0x0C: RTC registers
            if v <= 0x03 {
                self.rambank = v;
                self.selectrtc = false;
            } else if v >= 0x08 && v <= 0x0C {
                self.rambank = v - 0x08;
                self.selectrtc = true;
            }
        },
        0x6000..=0x7FFF => {
            // RTC latch: write 0x00 then 0x01
            if self.latchstate == 0 && v == 0x00 {
                self.latchstate = 1;
            } else if self.latchstate == 1 && v == 0x01 {
                // Latch current RTC values
                self.rtc_ram_latch = self.calc_rtc_reg();
                self.latchstate = 0;
            } else {
                self.latchstate = 0;
            }
        },
        _ => {},
    }
}

fn writeram(&mut self, a: u16, v: u8) {
    if !self.ram_on { return; }

    if self.selectrtc {
        // Writing to RTC register
        let reg = self.rambank as usize;
        self.rtc_ram[reg] = v;

        // If writing to register 4, recalculate epoch
        if reg == 4 {
            // Complex: adjust rtc_zero based on current values
            // ... (implementation omitted for brevity)
        }
    } else {
        // Writing to RAM
        let bank_offset = (self.rambank as usize) * 0x2000;
        let addr = (a as usize - 0xA000) + bank_offset;
        self.ram[addr % self.ram.len()] = v;
    }
}
```

**Key Insights:**
1. **Dual-purpose register**: 0x4000-0x5FFF selects RAM (0x00-0x03) or RTC (0x08-0x0C)
2. **Latching prevents race conditions**: Read from `rtc_ram_latch`, not live values
3. **Halt flag**: Checked in `calc_rtc_reg()` to stop time
4. **Unix epoch reference**: Store `rtc_zero`, calculate elapsed time on each read
5. **Save file format**: Include both RAM and RTC state (8-byte timestamp + RAM)

---

### MBC5: Simplified but Extended

MBC5 removes the quirks of MBC1 but supports more ROM/RAM.

#### Key Features:
- **9-bit ROM bank** (512 banks = 8MB max)
- **Split register**: Lower 8 bits at 0x2000-0x2FFF, upper 1 bit at 0x3000-0x3FFF
- **No bank restrictions**: Can select bank 0 freely
- **4-bit RAM bank** (16 banks = 128KB max)

#### mohanson Implementation (Reference):

```rust
impl Memory for Mbc5 {
    fn get(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                // Fixed bank 0
                self.rom[addr as usize]
            }
            0x4000..=0x7FFF => {
                // Switchable ROM bank (9-bit)
                let bank = self.rombank as usize;
                let offset = (addr - 0x4000) as usize;
                self.rom[(bank * 0x4000) + offset]
            }
            0xA000..=0xBFFF => {
                // Switchable RAM bank (4-bit)
                if !self.ram_enable { return 0xFF; }
                let bank = self.rambank as usize;
                let offset = (addr - 0xA000) as usize;
                self.ram[(bank * 0x2000) + offset]
            }
            _ => 0xFF,
        }
    }

    fn set(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enable = (val & 0x0F) == 0x0A;
            }
            0x2000..=0x2FFF => {
                // Lower 8 bits of ROM bank
                self.rombank = (self.rombank & 0x100) | (val as u16);
            }
            0x3000..=0x3FFF => {
                // Upper 1 bit of ROM bank
                self.rombank = (self.rombank & 0xFF) | (((val & 0x01) as u16) << 8);
            }
            0x4000..=0x5FFF => {
                // RAM bank select (4 bits)
                self.rambank = val & 0x0F;
            }
            0xA000..=0xBFFF => {
                if !self.ram_enable { return; }
                let bank = self.rambank as usize;
                let offset = (addr - 0xA000) as usize;
                self.ram[(bank * 0x2000) + offset] = val;
            }
            _ => {}
        }
    }
}
```

**Key Insights:**
1. **Simpler than MBC1**: No banking modes, no bank quirks
2. **Split ROM bank register**: Combine two writes to form 9-bit value
3. **Extended RAM**: 4 bits = 16 banks vs MBC1's 2 bits = 4 banks

---

## Critical Implementation Details

### 1. Save File Handling

**mohanson/gameboy approach:**
```rust
trait Stable {
    fn sav(&self) -> Vec<u8>;
}

impl Cartridge {
    pub fn save(&self, path: &str) {
        let data = self.sav();
        std::fs::write(path, data).unwrap();
    }

    pub fn load(&mut self, path: &str) {
        if let Ok(data) = std::fs::read(path) {
            // Load RAM
            self.ram[..data.len()].copy_from_slice(&data);
        }
    }
}
```

**rboy approach with RTC:**
```rust
fn dumpram(&self) -> Vec<u8> {
    let mut data = Vec::new();

    // Dump RAM
    data.extend_from_slice(&self.ram);

    // Dump RTC state (8 bytes: Unix timestamp as big-endian u64)
    let rtc_bytes = self.rtc_zero.to_be_bytes();
    data.extend_from_slice(&rtc_bytes);

    data
}

fn loadram(&mut self, data: &[u8]) -> Result<()> {
    if data.len() < self.ram.len() { return Err(...); }

    // Load RAM
    self.ram.copy_from_slice(&data[..self.ram.len()]);

    // Load RTC (if available)
    if data.len() >= self.ram.len() + 8 {
        let rtc_slice = &data[self.ram.len()..self.ram.len()+8];
        self.rtc_zero = u64::from_be_bytes(rtc_slice.try_into()?);
    }

    Ok(())
}
```

**Key Insight**: RTC state must be persisted alongside RAM for MBC3 cartridges.

---

### 2. ROM/RAM Boundary Checking

All implementations handle undersized ROMs/RAM differently:

**Wrap approach (safest):**
```rust
self.rom[addr % self.rom.len()]  // Wraps if ROM is too small
```

**Panic approach:**
```rust
self.rom[addr]  // Panics on out-of-bounds
```

**Fill approach:**
```rust
// Allocate full size, fill with 0xFF for missing portions
let mut rom = vec![0xFF; expected_size];
rom[..actual_data.len()].copy_from_slice(&actual_data);
```

**Recommendation**: Wrap for ROM (handles homebrew), panic for RAM (catches bugs).

---

### 3. Bank Address Calculations

**Correct pattern (used by all mature implementations):**

```rust
// For ROM banks (0x4000-0x7FFF with bank N):
let bank_offset = (selected_bank as usize) * 0x4000;
let addr_offset = (address - 0x4000) as usize;
let physical_addr = bank_offset + addr_offset;
self.rom[physical_addr]

// For RAM banks (0xA000-0xBFFF with bank N):
let bank_offset = (selected_bank as usize) * 0x2000;
let addr_offset = (address - 0xA000) as usize;
let physical_addr = bank_offset + addr_offset;
self.ram[physical_addr]
```

**Common mistakes:**
```rust
// ❌ WRONG: Doesn't handle bank 0 area (0x0000-0x3FFF)
let addr = (address as isize - 0x4000) + (0x4000 * bank as isize);

// ❌ WRONG: Negative addresses for 0x0000-0x3FFF
// This can wrap around or panic!
```

---

## Issues in Current Implementation

After analyzing the current `src/gb/cartridge.rs`, several issues were identified:

### Issue 1: Incorrect ROM Read Logic (Line 129-131)

**Current code:**
```rust
pub fn read(&self, address: u16) -> u8 {
    match address {
        0x0000..=0x7FFF => {
            let new_address: isize = (address as isize - 0x4000)
                                   + (0x4000 * self.current_rom as isize);
            self.rom[new_address as usize]
        },
        // ...
    }
}
```

**Problems:**
1. For addresses 0x0000-0x3FFF, this calculates negative addresses:
   - Example: `address=0x0100` → `0x0100 - 0x4000 = -0x3F00`
   - Even with bank 1: `-0x3F00 + 0x4000 = 0x0100` ✓ (accidentally works!)
   - But for bank 2: `-0x3F00 + 0x8000 = 0x4100` ❌ (reads wrong bank!)

2. Doesn't handle MBC1 banking modes:
   - In RAM banking mode, 0x0000-0x3FFF should read from bank 0
   - In ROM banking mode with upper bits, should read from higher bank

**Correct implementation:**
```rust
pub fn read(&self, address: u16) -> u8 {
    match address {
        0x0000..=0x3FFF => {
            // Bank 0 area - always fixed (or upper bits in MBC1 mode 1)
            self.rom[address as usize]
        },
        0x4000..=0x7FFF => {
            // Switchable bank area
            let bank_offset = (self.current_rom as usize) * 0x4000;
            let addr_offset = (address - 0x4000) as usize;
            self.rom[bank_offset + addr_offset]
        },
        0xA000..=0xBFFF => {
            if !self.enable_ram { return 0xFF; }
            let bank_offset = (self.current_ram as usize) * 0x2000;
            let addr_offset = (address - 0xA000) as usize;
            self.ram[bank_offset + addr_offset]
        },
        _ => panic!("Invalid cartridge address: 0x{:04X}", address),
    }
}
```

---

### Issue 2: Broken High/Low Bank Logic (Line 183-199)

**Current code:**
```rust
fn set_rombank_hi_lo(&mut self, data: u8, mode: SetRomBank) {
    match mode {
        SetRomBank::High => {
            self.current_rom &= 0x1F;
            let upper3 = data & 0xE0;  // ❌ Calculated but never used!
            self.current_rom |= data as u16;  // ❌ Wrong!
        },
        SetRomBank::Low => {
            let lower5 = data & 0x1F;
            self.current_rom &= 0xE0;
            self.current_rom |= lower5 as u16;
        },
    }
    if self.current_rom == 0 {
        self.current_rom = 1;
    }
}
```

**Problems:**
1. Line 187: `upper3` is calculated but never used
2. Line 188: Should be `|= (data as u16) << 5`, not just `|= data as u16`
3. The masking uses `0xE0` (3 bits) when it should use `0x60` (2 bits) for MBC1

**Correct implementation:**
```rust
fn set_rombank_hi_lo(&mut self, data: u8, mode: SetRomBank) {
    match mode {
        SetRomBank::High => {
            // Clear upper 2 bits, keep lower 5 bits
            self.current_rom &= 0x1F;
            // Set upper 2 bits (shifted to bits 5-6)
            let upper2 = (data & 0x03) as u16;
            self.current_rom |= upper2 << 5;
        },
        SetRomBank::Low => {
            // Clear lower 5 bits, keep upper bits
            self.current_rom &= !0x1F;
            // Set lower 5 bits
            let lower5 = (data & 0x1F) as u16;
            self.current_rom |= lower5;
        },
    }
    // Handle bank 0 → bank 1 quirk
    if self.current_rom == 0 || (self.current_rom & 0x1F) == 0 {
        self.current_rom = (self.current_rom & !0x1F) | 1;
    }
}
```

---

### Issue 3: Missing RAM Enable Check

**Current code:**
```rust
fn write_ram(&mut self, address: u16, data: u8) {
    let new_address: isize = (address as isize - 0xA000)
                           + ((self.ram_bank_size * self.current_ram) as isize);
    self.ram[new_address as usize] = data;
}
```

**Problem**: Doesn't check `self.enable_ram` flag!

**Fix:**
```rust
fn write_ram(&mut self, address: u16, data: u8) {
    if !self.enable_ram { return; }  // ← Add this check
    let bank_offset = (self.current_ram as usize) * (self.ram_bank_size as usize);
    let addr_offset = (address - 0xA000) as usize;
    self.ram[bank_offset + addr_offset] = data;
}
```

---

### Issue 4: MBC2 and MBC3 Not Implemented

**Current code:**
```rust
fn write_mbc2(&mut self, address: u16, data: u8) {
    // Commented out partial implementation
}

fn write_mbc3(&mut self, address: u16, data: u8) {
    // Empty
}
```

**Impact**: Games using MBC2 or MBC3 won't work at all.

---

### Issue 5: No Save File Support

None of the implementations include:
- Battery-backed RAM detection (check cartridge type for battery bit)
- Loading save files on boot
- Writing save files on exit
- RTC state persistence (for MBC3)

---

### Issue 6: Wasteful Struct Layout

**Current approach:**
```rust
pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    mbc: MemoryBankController,

    // These fields are only used by specific MBC types:
    current_rom: u16,     // Not used by MBC0
    current_ram: u16,     // Not used by MBC0, MBC2
    enable_ram: bool,     // Not used by MBC0
    rom_mode: bool,       // Only used by MBC1
    // Missing: RTC state for MBC3
    // Missing: Banking mode flag for MBC1
}
```

Every Cartridge instance allocates space for all MBC types, even if only using MBC0.

**Better approach** (trait-based):
```rust
pub struct Cartridge {
    mbc: Box<dyn MBC>,  // Only allocates space for active MBC type
}
```

---

## Recommendations

Based on this analysis, here are recommendations for improving the current implementation:

### Short-term (Quick Fixes)

1. **Fix ROM read logic** (lines 129-131)
   - Split into two match arms: 0x0000-0x3FFF and 0x4000-0x7FFF
   - Use proper offset calculations

2. **Fix bank selection** (lines 183-199)
   - Correct the bit shifting for upper bits
   - Fix masking (use 0x60 for MBC1, not 0xE0)

3. **Add RAM enable check** (line 202)
   - Check `enable_ram` before writing

4. **Implement MBC2 and MBC3**
   - MBC2: Simple, just needs 4-bit RAM handling
   - MBC3: Complex, needs RTC support

### Medium-term (Refactoring)

5. **Refactor to trait-based architecture**
   - Move to rboy-style `Box<dyn MBC>` approach
   - Create separate files: `mbc/mbc0.rs`, `mbc/mbc1.rs`, etc.
   - Benefits: cleaner code, easier testing, less memory waste

6. **Add save file support**
   - Detect battery-backed cartridges
   - Auto-load .sav files on boot
   - Auto-save on exit or periodically

7. **Add MBC5 support**
   - Needed for many GBC games
   - Simpler than MBC1 (no quirks)

### Long-term (Advanced Features)

8. **Add RTC support for MBC3**
   - Implement RTC registers (0x08-0x0C)
   - Add latching mechanism
   - Persist RTC state to .rtc files

9. **Add validation**
   - Verify Nintendo logo
   - Check header checksum
   - Detect ROM corruption

10. **Add support for other MBC types**
    - MBC6 (rare, used by a few games)
    - MBC7 (motion sensor)
    - HuC1 (Hudson cartridges)
    - MMM01 (multicart)

---

## Comparison Table

| Feature | Current | rboy | mohanson | gb-rs |
|---------|---------|------|----------|-------|
| **Architecture** | Enum dispatch | Trait (Box) | Multi-trait | Function ptr |
| **MBC0** | ✅ | ✅ | ✅ | ✅ |
| **MBC1** | ⚠️ Buggy | ✅ | ✅ | ✅ |
| **MBC2** | ❌ Empty | ✅ | ✅ | ✅ |
| **MBC3** | ❌ Empty | ✅ | ✅ | ⚠️ No RTC |
| **MBC5** | ❌ No enum | ✅ | ✅ | ❌ |
| **RTC Support** | ❌ | ✅ | ✅ | ❌ |
| **Save Files** | ❌ | ✅ | ✅ | ⚠️ Basic |
| **ROM Read Bug** | ❌ | ✅ | ✅ | ✅ |
| **Bank Select Bug** | ❌ | ✅ | ✅ | ✅ |
| **RAM Enable Check** | ❌ | ✅ | ✅ | ✅ |
| **Tests** | ⚠️ Basic | ✅ | ⚠️ Limited | ⚠️ Limited |
| **Code Organization** | ⚠️ Single file | ✅ Separate files | ✅ Separate files | ⚠️ 2 files |
| **Memory Efficiency** | ❌ Wastes space | ✅ Optimal | ✅ Optimal | ✅ Optimal |
| **Extensibility** | ❌ Hard to extend | ✅ Easy | ✅ Easy | ⚠️ Moderate |

**Legend:**
- ✅ = Fully implemented and correct
- ⚠️ = Partially implemented or has issues
- ❌ = Not implemented or broken

---

## Conclusion

The **trait-based approach (rboy-style)** is the clear winner for a production emulator:

1. **Clean architecture**: Each MBC type is independent
2. **Easy to test**: Test each MBC implementation separately
3. **Memory efficient**: Only allocate space for active MBC
4. **Extensible**: Add new MBC types without touching existing code
5. **Industry standard**: Most mature emulators use this pattern

However, for **learning purposes**, the current enum-based approach is acceptable as long as the bugs are fixed. It's simpler to understand and has zero runtime overhead.

### Recommended Path Forward:

1. **Phase 1**: Fix critical bugs in current implementation (ROM read, bank select, RAM enable)
2. **Phase 2**: Complete MBC2 and MBC3 implementations (without RTC initially)
3. **Phase 3**: Refactor to trait-based architecture
4. **Phase 4**: Add save file support and RTC
5. **Phase 5**: Add MBC5 and validation

This incremental approach allows continued progress while improving code quality.
