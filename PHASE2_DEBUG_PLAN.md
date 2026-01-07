# Phase 2: Debug & Visualization Infrastructure - Implementation Plan

**Status:** 🎯 READY TO START
**Priority:** ⚡ HIGH PRIORITY - CRITICAL PATH
**Created:** 2026-01-07
**Target Completion:** 5-7 days of focused work

---

## Executive Summary

This document provides a comprehensive implementation plan for Phase 2 of the Game Boy Emulator project. Phase 2 focuses on building robust debugging and visualization infrastructure that will be **essential** for all subsequent development phases.

**Why Phase 2 is Critical:**
- CPU opcode implementation (Phase 3) requires immediate visibility into CPU state
- Without debugging tools, tracking down bugs becomes exponentially harder
- A 5-7 day investment now will save weeks of debugging time later
- Makes the development process significantly more intuitive and enjoyable

**Key Deliverables:**
1. Real-time CPU state visualization
2. Memory inspection and dump capabilities
3. Interactive step-by-step debugger
4. Comprehensive logging system with instruction tracing
5. Breakpoint support for efficient debugging

---

## Current State Analysis

### Project Metrics
- **Lines of Code:** ~934 lines across 8 Rust files
- **Completion:** ~5% of full emulator
- **Phase 1 Status:** ✅ Complete (Foundation & Testing)

### Existing Components
```
src/
├── gb/
│   ├── cpu.rs           (~250 lines) - Basic CPU with ~8 opcodes
│   ├── register.rs      (~80 lines)  - Register implementation
│   ├── opcode.rs        (~50 lines)  - Opcode enum definitions
│   ├── interconnect.rs  (~150 lines) - Memory bus (partial)
│   ├── cartridge.rs     (~200 lines) - ROM loading & MBC
│   ├── gameboy.rs       (~100 lines) - Main orchestrator
│   └── mod.rs           (~20 lines)  - Module exports
└── main.rs              (~30 lines)  - CLI entry point
```

### Current CLI Support
- **Implemented:** Basic ROM file loading (`--file-path <FILE>`)
- **Missing:** All debugging flags and features

### Current Limitations
- No visibility into CPU state during execution
- No way to pause or step through execution
- No memory inspection capabilities
- Panics immediately on unimplemented opcodes
- No logging or tracing infrastructure
- Debugging requires manual `println!` statements and recompilation

---

## Objectives & Success Criteria

### Primary Objectives
1. **Enable visibility** into emulator internals without code modification
2. **Support step-by-step debugging** for opcode implementation
3. **Provide memory inspection** for verifying reads/writes
4. **Create instruction tracing** for understanding execution flow
5. **Implement breakpoints** for efficient bug hunting

### Success Criteria
- ✅ Can view all CPU registers and flags in real-time
- ✅ Can step through execution one instruction at a time
- ✅ Can set breakpoints on memory addresses
- ✅ Can inspect memory regions (ROM, RAM, etc.)
- ✅ Can trace instruction execution to a log file
- ✅ Can run emulator with different log levels
- ✅ Zero impact on execution when debugging is disabled

---

## Detailed Implementation Plan

### Phase 2A: Basic State Display (Days 1-2)
**Goal:** Implement pretty-printing and basic state visualization

#### Task 2A.1: CPU State Display
**File:** `src/gb/cpu.rs`

**Implementation:**
1. Add `Display` trait implementation for `Cpu` struct
2. Add `Display` trait implementation for `Flags` struct
3. Create formatted output showing:
   - All registers (PC, SP, AF, BC, DE, HL) in hex and decimal
   - All flags (Z, N, H, C) with visual indicators (✓/✗)
   - Current instruction opcode
   - Cycle counter (if available)

**Example Output:**
```
┌─────────────── CPU State ───────────────┐
│ PC: 0x0150 (336)    SP: 0xFFFE (65534) │
│ AF: 0x01B0          BC: 0x0013          │
│ DE: 0x00D8          HL: 0x014D          │
│                                          │
│ Flags: [Z✓ N✗ H✓ C✗]                   │
│ Current: 0x3E (LD A, d8)                │
│ Cycles: 1,234,567                        │
└──────────────────────────────────────────┘
```

**Testing:**
- Unit tests for `Display` implementations
- Visual verification with sample CPU states

#### Task 2A.2: Register Helper Methods
**File:** `src/gb/cpu.rs`

Add public inspection methods (non-mutating):
```rust
impl Cpu {
    pub fn debug_state(&self) -> String { /* ... */ }
    pub fn get_pc(&self) -> u16 { /* ... */ }
    pub fn get_sp(&self) -> u16 { /* ... */ }
    pub fn get_flags(&self) -> (bool, bool, bool, bool) { /* ... */ }
    // ... etc
}
```

#### Task 2A.3: Memory Viewer Foundation
**File:** `src/gb/interconnect.rs`

**Implementation:**
1. Add method to dump memory range:
```rust
impl Interconnect {
    pub fn dump_memory(&self, start: u16, end: u16) -> Vec<u8> { /* ... */ }
    pub fn format_hex_dump(&self, start: u16, length: u16) -> String { /* ... */ }
}
```

2. Create hex dump formatter with ASCII representation:
```
0x8000: 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F  ................
0x8010: 10 11 12 13 14 15 16 17 18 19 1A 1B 1C 1D 1E 1F  ................
```

**Testing:**
- Test memory dump with known data
- Verify boundary conditions (start/end of regions)

#### Task 2A.4: CLI Debug Flag
**File:** `src/main.rs`

Add `--debug` flag using clap:
```rust
#[derive(Parser, Debug)]
struct Args {
    /// Path to the ROM file to load
    #[arg(short, long)]
    file_path: PathBuf,

    /// Enable debug mode (show state after each instruction)
    #[arg(long)]
    debug: bool,

    /// Number of instructions to execute before stopping (0 = unlimited)
    #[arg(long, default_value = "0")]
    step_limit: u64,
}
```

**Implementation:**
1. Pass debug flag to GameBoy
2. Modify execution loop to print state when debug=true
3. Add instruction counter and step limit support

#### Task 2A.5: GameBoy Debug Integration
**File:** `src/gb/gameboy.rs`

**Modification:**
```rust
pub struct GameBoy {
    cpu: Cpu,
    interconnect: Interconnect,
    debug_mode: bool,
    instruction_count: u64,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>, debug: bool) -> Self { /* ... */ }

    pub fn run(&mut self, step_limit: u64) {
        loop {
            if self.debug_mode {
                println!("{}", self.cpu.debug_state());
                // Wait for user input in future phase
            }

            self.step();
            self.instruction_count += 1;

            if step_limit > 0 && self.instruction_count >= step_limit {
                break;
            }
        }
    }
}
```

**Testing:**
- Run with `--debug` flag and verify output
- Test with `--step-limit` to verify it stops correctly

**Deliverables:**
- CPU state visualization
- Memory hex dump capability
- `--debug` flag functional
- `--step-limit` flag functional

**Estimated Time:** 1.5-2 days

---

### Phase 2B: Logging Infrastructure (Day 3)
**Goal:** Implement comprehensive logging system with multiple levels

#### Task 2B.1: Add Logging Dependencies
**File:** `Cargo.toml`

Add dependencies:
```toml
[dependencies]
# ... existing dependencies ...
log = "0.4"
env_logger = "0.11"
```

#### Task 2B.2: Initialize Logger
**File:** `src/main.rs`

**Implementation:**
```rust
fn main() {
    // Initialize logger (reads RUST_LOG env var)
    env_logger::Builder::from_default_env()
        .format_timestamp_millis()
        .init();

    let args = Args::parse();
    // ... rest of main
}
```

Add CLI flag for log level:
```rust
#[arg(long, default_value = "info")]
log_level: String,  // trace, debug, info, warn, error
```

#### Task 2B.3: Add Logging Throughout Codebase
**Files:** All `src/gb/*.rs` files

**Implementation Pattern:**
```rust
use log::{trace, debug, info, warn, error};

// In CPU execution:
trace!("Executing opcode: {:02X} at PC: {:04X}", opcode, self.reg_pc);
debug!("Register A after LD: {:02X}", self.get_reg_a());

// In memory operations:
trace!("Memory read: addr={:04X} value={:02X}", address, value);
trace!("Memory write: addr={:04X} value={:02X}", address, value);

// In cartridge:
info!("Loaded ROM: {} ({} banks, MBC: {:?})", title, num_banks, mbc_type);
debug!("ROM bank switch: {} -> {}", old_bank, new_bank);

// Error cases:
warn!("Attempted to write to ROM at {:04X}", address);
error!("Invalid opcode: {:02X} at PC: {:04X}", opcode, pc);
```

**Locations to Add Logging:**

**`src/gb/cpu.rs`:**
- Opcode execution (TRACE level)
- Register modifications (DEBUG level)
- Flag changes (TRACE level)
- Unimplemented opcodes (ERROR level)

**`src/gb/interconnect.rs`:**
- Memory reads (TRACE level)
- Memory writes (TRACE level)
- Memory region access (DEBUG level)

**`src/gb/cartridge.rs`:**
- ROM loading (INFO level)
- Bank switching (DEBUG level)
- Invalid operations (WARN level)

**`src/gb/gameboy.rs`:**
- Emulator initialization (INFO level)
- Execution start/stop (INFO level)
- Frame/cycle timing (DEBUG level)

#### Task 2B.4: Instruction Trace Mode
**File:** `src/main.rs` and `src/gb/cpu.rs`

Add `--trace` flag:
```rust
/// Enable instruction trace (logs every instruction to file)
#[arg(long)]
trace: bool,

/// Output file for instruction trace
#[arg(long, default_value = "trace.log")]
trace_file: PathBuf,
```

Create instruction trace logger:
```rust
// In cpu.rs
pub fn log_instruction(&self, opcode: u8, cycles: u32) {
    info!("[{:08}] PC:{:04X} OP:{:02X} {:20} | A:{:02X} F:{:02X} BC:{:04X} DE:{:04X} HL:{:04X} SP:{:04X}",
        self.cycle_count,
        self.reg_pc,
        opcode,
        format!("{:?}", Opcode::from_u8(opcode)),
        self.get_reg_a(),
        self.get_flags_as_byte(),
        self.regs_bc.get(),
        self.regs_de.get(),
        self.regs_hl.get(),
        self.reg_sp.get()
    );
}
```

**Example Trace Output:**
```
[00000123] PC:0150 OP:3E LD A, d8             | A:00 F:B0 BC:0013 DE:00D8 HL:014D SP:FFFE
[00000127] PC:0152 OP:E0 LD (FF00+a8), A     | A:01 F:B0 BC:0013 DE:00D8 HL:014D SP:FFFE
[00000130] PC:0154 OP:AF XOR A               | A:01 F:B0 BC:0013 DE:00D8 HL:014D SP:FFFE
```

**Testing:**
- Run with `RUST_LOG=trace` and verify output
- Run with `--trace` and verify log file creation
- Test different log levels (info, debug, trace)

**Deliverables:**
- Logging infrastructure integrated
- Log statements throughout codebase
- Instruction trace mode working
- Multiple log levels supported

**Estimated Time:** 1 day

---

### Phase 2C: Interactive Debugger (Days 4-5)
**Goal:** Implement step-by-step execution and breakpoints

#### Task 2C.1: Debugger State Management
**File:** Create `src/gb/debugger.rs`

**Implementation:**
```rust
use std::collections::HashSet;
use std::io::{self, Write};

pub struct Debugger {
    breakpoints: HashSet<u16>,
    memory_watchpoints: HashSet<u16>,
    step_mode: bool,
    enabled: bool,
}

impl Debugger {
    pub fn new(enabled: bool) -> Self {
        Self {
            breakpoints: HashSet::new(),
            memory_watchpoints: HashSet::new(),
            step_mode: enabled,
            enabled,
        }
    }

    pub fn add_breakpoint(&mut self, addr: u16) {
        self.breakpoints.insert(addr);
    }

    pub fn remove_breakpoint(&mut self, addr: u16) {
        self.breakpoints.remove(&addr);
    }

    pub fn should_break(&self, pc: u16) -> bool {
        self.breakpoints.contains(&pc)
    }

    pub fn add_watchpoint(&mut self, addr: u16) {
        self.memory_watchpoints.insert(addr);
    }

    pub fn should_watch(&self, addr: u16) -> bool {
        self.memory_watchpoints.contains(&addr)
    }
}
```

#### Task 2C.2: Debugger REPL Interface
**File:** `src/gb/debugger.rs`

**Implementation:**
```rust
impl Debugger {
    pub fn handle_breakpoint(&mut self, cpu: &Cpu, interconnect: &Interconnect) {
        println!("\n🔴 Breakpoint hit at PC: {:04X}", cpu.get_pc());
        println!("{}", cpu.debug_state());

        loop {
            print!("\ndbg> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            match self.parse_command(input, cpu, interconnect) {
                DebugCommand::Continue => break,
                DebugCommand::Step => {
                    self.step_mode = true;
                    break;
                }
                DebugCommand::Quit => std::process::exit(0),
                DebugCommand::Unknown => {
                    println!("Unknown command. Type 'help' for available commands.");
                }
            }
        }
    }

    fn parse_command(&mut self, input: &str, cpu: &Cpu, interconnect: &Interconnect)
        -> DebugCommand {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return DebugCommand::Unknown;
        }

        match parts[0] {
            "c" | "continue" => DebugCommand::Continue,
            "s" | "step" => DebugCommand::Step,
            "n" | "next" => DebugCommand::Step,
            "q" | "quit" => DebugCommand::Quit,
            "r" | "registers" => {
                println!("{}", cpu.debug_state());
                DebugCommand::Unknown
            }
            "m" | "memory" => {
                if parts.len() >= 2 {
                    if let Ok(addr) = parse_hex(parts[1]) {
                        let length = if parts.len() >= 3 {
                            parse_hex(parts[2]).unwrap_or(64)
                        } else {
                            64
                        };
                        println!("{}", interconnect.format_hex_dump(addr, length));
                    }
                }
                DebugCommand::Unknown
            }
            "b" | "break" => {
                if parts.len() >= 2 {
                    if let Ok(addr) = parse_hex(parts[1]) {
                        self.add_breakpoint(addr);
                        println!("Breakpoint added at {:04X}", addr);
                    }
                }
                DebugCommand::Unknown
            }
            "d" | "delete" => {
                if parts.len() >= 2 {
                    if let Ok(addr) = parse_hex(parts[1]) {
                        self.remove_breakpoint(addr);
                        println!("Breakpoint removed at {:04X}", addr);
                    }
                }
                DebugCommand::Unknown
            }
            "l" | "list" => {
                println!("Breakpoints:");
                for bp in &self.breakpoints {
                    println!("  - {:04X}", bp);
                }
                DebugCommand::Unknown
            }
            "w" | "watch" => {
                if parts.len() >= 2 {
                    if let Ok(addr) = parse_hex(parts[1]) {
                        self.add_watchpoint(addr);
                        println!("Watchpoint added at {:04X}", addr);
                    }
                }
                DebugCommand::Unknown
            }
            "h" | "help" => {
                print_help();
                DebugCommand::Unknown
            }
            _ => DebugCommand::Unknown,
        }
    }
}

enum DebugCommand {
    Continue,
    Step,
    Quit,
    Unknown,
}

fn parse_hex(s: &str) -> Result<u16, std::num::ParseIntError> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    u16::from_str_radix(s, 16)
}

fn print_help() {
    println!(r#"
Available Commands:
  c, continue       - Continue execution until next breakpoint
  s, step, n, next  - Execute one instruction
  q, quit           - Exit emulator
  r, registers      - Show CPU registers and flags
  m, memory <addr> [len] - Dump memory (hex: 0x1234, default len: 64)
  b, break <addr>   - Set breakpoint at address
  d, delete <addr>  - Remove breakpoint at address
  l, list           - List all breakpoints
  w, watch <addr>   - Set memory watchpoint
  h, help           - Show this help message

Examples:
  m 0x8000          - Dump 64 bytes from 0x8000
  m 0xC000 0x100    - Dump 256 bytes from 0xC000
  b 0x0150          - Break at 0x0150
"#);
}
```

#### Task 2C.3: Integrate Debugger with GameBoy
**File:** `src/gb/gameboy.rs`

**Modification:**
```rust
use crate::gb::debugger::Debugger;

pub struct GameBoy {
    cpu: Cpu,
    interconnect: Interconnect,
    debugger: Debugger,
    instruction_count: u64,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>, debug: bool) -> Self {
        // ...
        debugger: Debugger::new(debug),
        // ...
    }

    pub fn run(&mut self, step_limit: u64) {
        loop {
            let pc = self.cpu.get_pc();

            // Check breakpoints
            if self.debugger.should_break(pc) {
                self.debugger.handle_breakpoint(&self.cpu, &self.interconnect);
            }

            // Step mode
            if self.debugger.is_step_mode() {
                println!("{}", self.cpu.debug_state());
                self.debugger.handle_breakpoint(&self.cpu, &self.interconnect);
            }

            self.step();
            self.instruction_count += 1;

            if step_limit > 0 && self.instruction_count >= step_limit {
                break;
            }
        }
    }
}
```

#### Task 2C.4: CLI Breakpoint Support
**File:** `src/main.rs`

Add flags:
```rust
/// Enable interactive debugger
#[arg(long)]
debugger: bool,

/// Set breakpoint at address (can be used multiple times)
#[arg(long = "break", value_name = "ADDRESS")]
breakpoints: Vec<String>,

/// Set memory watchpoint (can be used multiple times)
#[arg(long = "watch", value_name = "ADDRESS")]
watchpoints: Vec<String>,
```

Parse and pass to GameBoy:
```rust
let mut gb = GameBoy::new(file_buf, args.debugger);

for bp in &args.breakpoints {
    if let Ok(addr) = parse_hex(bp) {
        gb.add_breakpoint(addr);
    }
}

for wp in &args.watchpoints {
    if let Ok(addr) = parse_hex(wp) {
        gb.add_watchpoint(addr);
    }
}

gb.run(args.step_limit);
```

**Testing:**
- Test breakpoint triggering
- Test step-by-step execution
- Test REPL commands (registers, memory, continue, etc.)
- Test watchpoints
- Test multiple breakpoints

**Deliverables:**
- Interactive debugger with REPL
- Breakpoint support
- Memory watchpoints
- Step-by-step execution
- CLI integration

**Estimated Time:** 2 days

---

### Phase 2D: Memory Dump Utility (Day 6)
**Goal:** Add comprehensive memory inspection tools

#### Task 2D.1: Enhanced Memory Viewer
**File:** `src/gb/interconnect.rs`

**Implementation:**
```rust
impl Interconnect {
    /// Dump memory with annotations showing regions
    pub fn format_annotated_dump(&self, start: u16, end: u16) -> String {
        let mut output = String::new();

        output.push_str(&format!("\n┌─────────── Memory Dump: {:04X}-{:04X} ───────────┐\n",
            start, end));

        for addr in (start..=end).step_by(16) {
            let region = self.get_region_name(addr);
            output.push_str(&format!("{:04X} [{}]: ", addr, region));

            // Hex bytes
            for i in 0..16 {
                let byte_addr = addr + i;
                if byte_addr <= end {
                    let byte = self.read_byte(byte_addr);
                    output.push_str(&format!("{:02X} ", byte));
                } else {
                    output.push_str("   ");
                }
            }

            output.push_str(" | ");

            // ASCII representation
            for i in 0..16 {
                let byte_addr = addr + i;
                if byte_addr <= end {
                    let byte = self.read_byte(byte_addr);
                    let ch = if byte >= 0x20 && byte <= 0x7E {
                        byte as char
                    } else {
                        '.'
                    };
                    output.push(ch);
                }
            }

            output.push('\n');
        }

        output.push_str("└────────────────────────────────────────────────────┘\n");
        output
    }

    fn get_region_name(&self, addr: u16) -> &'static str {
        match addr {
            0x0000..=0x3FFF => "ROM0",
            0x4000..=0x7FFF => "ROMX",
            0x8000..=0x9FFF => "VRAM",
            0xA000..=0xBFFF => "ERAM",
            0xC000..=0xDFFF => "WRAM",
            0xE000..=0xFDFF => "ECHO",
            0xFE00..=0xFE9F => "OAM ",
            0xFEA0..=0xFEFF => "----",
            0xFF00..=0xFF7F => "IO  ",
            0xFF80..=0xFFFE => "HRAM",
            0xFFFF => "IE  ",
        }
    }

    /// Compare two memory regions
    pub fn compare_memory(&self, other: &Self, start: u16, end: u16) -> Vec<MemoryDiff> {
        let mut diffs = Vec::new();

        for addr in start..=end {
            let val1 = self.read_byte(addr);
            let val2 = other.read_byte(addr);

            if val1 != val2 {
                diffs.push(MemoryDiff {
                    address: addr,
                    old_value: val1,
                    new_value: val2,
                });
            }
        }

        diffs
    }
}

#[derive(Debug)]
pub struct MemoryDiff {
    pub address: u16,
    pub old_value: u8,
    pub new_value: u8,
}
```

#### Task 2D.2: Add Memory Dump CLI Flag
**File:** `src/main.rs`

Add flags:
```rust
/// Dump memory region and exit (format: START:END, e.g., 0x8000:0x9FFF)
#[arg(long = "dump-memory")]
dump_memory: Option<String>,

/// Save memory dump to file
#[arg(long)]
dump_file: Option<PathBuf>,
```

**Implementation:**
```rust
fn main() {
    // ... init code ...

    let mut gb = GameBoy::new(file_buf, args.debugger);

    // Handle memory dump mode
    if let Some(range) = args.dump_memory {
        let (start, end) = parse_memory_range(&range);
        let dump = gb.dump_memory(start, end);

        if let Some(file) = args.dump_file {
            std::fs::write(file, &dump).unwrap();
        } else {
            println!("{}", dump);
        }
        return;
    }

    gb.run(args.step_limit);
}

fn parse_memory_range(s: &str) -> (u16, u16) {
    let parts: Vec<&str> = s.split(':').collect();
    let start = parse_hex(parts[0]).unwrap_or(0);
    let end = parse_hex(parts.get(1).unwrap_or(&"FFFF")).unwrap_or(0xFFFF);
    (start, end)
}
```

**Testing:**
- Test memory dump with various ranges
- Test file output
- Test annotated format
- Test region detection

**Deliverables:**
- Enhanced memory viewer with region annotations
- CLI memory dump functionality
- File output support

**Estimated Time:** 0.5 days

---

### Phase 2E: Advanced Features (Day 7) - OPTIONAL
**Goal:** Add quality-of-life improvements

#### Task 2E.1: State Save/Load
**File:** Create `src/gb/snapshot.rs`

**Implementation:**
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct EmulatorSnapshot {
    cpu_state: CpuState,
    memory: Vec<u8>,
    instruction_count: u64,
}

impl GameBoy {
    pub fn save_snapshot(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let snapshot = EmulatorSnapshot {
            cpu_state: self.cpu.get_state(),
            memory: self.interconnect.dump_all(),
            instruction_count: self.instruction_count,
        };

        let file = File::create(path)?;
        bincode::serialize_into(file, &snapshot)?;
        Ok(())
    }

    pub fn load_snapshot(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let snapshot: EmulatorSnapshot = bincode::deserialize_from(file)?;

        self.cpu.set_state(snapshot.cpu_state);
        self.interconnect.load_all(&snapshot.memory);
        self.instruction_count = snapshot.instruction_count;
        Ok(())
    }
}
```

Add dependencies:
```toml
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
```

Add CLI support:
```rust
/// Save emulator state to file
#[arg(long)]
save_state: Option<PathBuf>,

/// Load emulator state from file
#[arg(long)]
load_state: Option<PathBuf>,
```

#### Task 2E.2: Disassembly View
**File:** `src/gb/cpu.rs`

**Implementation:**
```rust
impl Cpu {
    /// Disassemble next N instructions from current PC
    pub fn disassemble(&self, interconnect: &Interconnect, count: usize) -> String {
        let mut output = String::new();
        let mut pc = self.reg_pc;

        for _ in 0..count {
            let opcode = interconnect.read_byte(pc);
            let (mnemonic, size) = self.get_instruction_info(opcode);

            output.push_str(&format!("{:04X}: {:02X} {}\n",
                pc, opcode, mnemonic));

            pc += size as u16;
        }

        output
    }

    fn get_instruction_info(&self, opcode: u8) -> (String, u8) {
        // Return (mnemonic, instruction size in bytes)
        match opcode {
            0x00 => ("NOP".to_string(), 1),
            0x3E => ("LD A, d8".to_string(), 2),
            0xAF => ("XOR A".to_string(), 1),
            // ... add all opcodes
            _ => (format!("??? {:02X}", opcode), 1),
        }
    }
}
```

Add to debugger REPL:
```rust
"dis" | "disassemble" => {
    let count = if parts.len() >= 2 {
        parts[1].parse().unwrap_or(10)
    } else {
        10
    };
    println!("{}", cpu.disassemble(interconnect, count));
    DebugCommand::Unknown
}
```

#### Task 2E.3: Terminal UI (TUI) - Advanced
**File:** Create `src/gb/tui.rs`

Add dependency:
```toml
ratatui = "0.26"
crossterm = "0.27"
```

**Implementation:**
Create split-screen TUI with:
- CPU register panel (top-left)
- Memory viewer panel (top-right)
- Disassembly panel (bottom-left)
- Log output panel (bottom-right)

This is a significant undertaking and may warrant its own sub-phase.

**Note:** TUI is optional and can be deferred to a later phase if time is constrained.

**Deliverables:**
- State save/load functionality
- Disassembly viewer
- (Optional) Terminal UI

**Estimated Time:** 1 day (without TUI), 2-3 days (with TUI)

---

## Technical Design Details

### Architecture Additions

```
src/
├── gb/
│   ├── debugger.rs        [NEW] - Debugger state and REPL
│   ├── snapshot.rs        [NEW] - Save/load state
│   ├── tui.rs            [NEW] - Terminal UI (optional)
│   ├── cpu.rs            [MODIFY] - Add Display, debug methods
│   ├── interconnect.rs   [MODIFY] - Add memory dump methods
│   ├── gameboy.rs        [MODIFY] - Integrate debugger
│   └── mod.rs            [MODIFY] - Export new modules
└── main.rs               [MODIFY] - Add CLI flags
```

### CLI Flag Summary

| Flag | Type | Description | Example |
|------|------|-------------|---------|
| `--debug` | bool | Enable debug mode | `--debug` |
| `--step-limit <N>` | u64 | Execute N instructions | `--step-limit 1000` |
| `--log-level <LEVEL>` | string | Set log level | `--log-level trace` |
| `--trace` | bool | Enable instruction trace | `--trace` |
| `--trace-file <FILE>` | path | Trace output file | `--trace-file cpu.log` |
| `--debugger` | bool | Enable interactive debugger | `--debugger` |
| `--break <ADDR>` | string | Set breakpoint | `--break 0x0150` |
| `--watch <ADDR>` | string | Set watchpoint | `--watch 0xFF44` |
| `--dump-memory <RANGE>` | string | Dump memory | `--dump-memory 0x8000:0x9FFF` |
| `--dump-file <FILE>` | path | Save dump to file | `--dump-file vram.bin` |
| `--save-state <FILE>` | path | Save state | `--save-state save.bin` |
| `--load-state <FILE>` | path | Load state | `--load-state save.bin` |

### Performance Considerations

**Zero-cost Debugging:**
- Use conditional compilation for trace logs: `log::trace!()` compiles to nothing at release
- Debugger checks only occur when enabled
- No overhead when debug flags are not used

**Optimization Strategy:**
```rust
// Fast path when debugging disabled
if !self.debug_mode && !self.debugger.enabled {
    return self.execute_fast();
}

// Slow path with debugging
self.execute_with_debug();
```

---

## Testing Strategy

### Unit Tests
- Test `Display` implementations with known CPU states
- Test memory dump formatting
- Test breakpoint add/remove
- Test watchpoint triggering
- Test snapshot serialization/deserialization

### Integration Tests
- Run simple ROM with `--debug` and verify output
- Test breakpoint stops execution at correct PC
- Test step mode advances exactly one instruction
- Test memory dump matches expected values

### Manual Testing
1. Load Tetris ROM with debugger
2. Set breakpoint at boot sequence
3. Step through first 10 instructions
4. Inspect memory regions
5. Verify register changes
6. Test continue/quit commands

### Test ROM Validation
Once CPU opcodes are implemented, use debug tools to validate against:
- Blargg's test ROMs
- Compare traces with reference emulator (BGB, SameBoy)

---

## Dependencies to Add

```toml
[dependencies]
# Existing
clap = { version = "4.4.6", features = ["derive"] }
num-traits = "0.2"
num-derive = "0.4"

# Phase 2 additions
log = "0.4"                    # Logging facade
env_logger = "0.11"            # Logger implementation

# Optional (Phase 2E)
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
ratatui = "0.26"              # TUI (optional)
crossterm = "0.27"            # Terminal control (optional)
```

---

## Usage Examples

### Basic Debug Mode
```bash
# Show state after each instruction (first 100 instructions)
cargo run --release -- --file-path tetris.gb --debug --step-limit 100
```

### Instruction Tracing
```bash
# Trace all instructions to file
RUST_LOG=trace cargo run --release -- --file-path tetris.gb --trace --trace-file cpu_trace.log

# View trace
tail -f cpu_trace.log
```

### Interactive Debugging
```bash
# Start with breakpoint at 0x0150 (past boot ROM)
cargo run --release -- --file-path tetris.gb --debugger --break 0x0150

# Interactive session:
dbg> r                    # Show registers
dbg> m 0x8000 0x100       # Dump VRAM
dbg> b 0x0200             # Add another breakpoint
dbg> c                    # Continue
dbg> s                    # Step one instruction
dbg> q                    # Quit
```

### Memory Inspection
```bash
# Dump VRAM region
cargo run --release -- --file-path tetris.gb --dump-memory 0x8000:0x9FFF

# Dump to file
cargo run --release -- --file-path tetris.gb --dump-memory 0x0000:0x7FFF --dump-file rom_dump.bin
```

### Combined Usage
```bash
# Run with debug, logging, and breakpoint
RUST_LOG=debug cargo run --release -- \
    --file-path tetris.gb \
    --debugger \
    --break 0x0150 \
    --trace \
    --trace-file execution.log
```

---

## Success Metrics

### Quantitative Metrics
- ✅ 100% of CPU state is visible (all registers, all flags)
- ✅ Can inspect any memory address (0x0000-0xFFFF)
- ✅ Breakpoints trigger with 100% accuracy
- ✅ Step mode advances exactly 1 instruction
- ✅ Log output includes all memory reads/writes
- ✅ Zero overhead when debugging disabled (verified via benchmarks)

### Qualitative Metrics
- ✅ Developer can debug opcodes without adding `println!()` statements
- ✅ Can identify incorrect flag behavior within seconds
- ✅ Can verify memory writes happen at correct addresses
- ✅ Can compare execution trace with reference emulator
- ✅ Debugging is intuitive and doesn't require recompilation

---

## Risk Assessment & Mitigation

### Risk 1: Performance Degradation
**Likelihood:** Medium
**Impact:** High
**Mitigation:**
- Use conditional compilation for trace logs
- Benchmark release builds before/after
- Add fast-path execution when debugging disabled
- Profile hot paths and optimize

### Risk 2: Scope Creep
**Likelihood:** High
**Impact:** Medium
**Mitigation:**
- Stick to defined phases 2A-2D
- Phase 2E (TUI) is explicitly optional
- Can defer advanced features to later
- Focus on "good enough" before "perfect"

### Risk 3: Complexity of Debugger REPL
**Likelihood:** Low
**Impact:** Low
**Mitigation:**
- Start with basic commands only
- Add advanced features incrementally
- Use existing REPL libraries if needed
- Keep command parser simple

### Risk 4: Integration Issues with Existing Code
**Likelihood:** Low
**Impact:** Medium
**Mitigation:**
- Write comprehensive unit tests first
- Make minimal changes to existing code
- Use traits for clean interfaces
- Test integration at each step

---

## Timeline & Milestones

### Week 1 (Days 1-5)
- **Day 1-2:** Phase 2A (Basic State Display)
  - Milestone: Can view CPU state with `--debug` flag
- **Day 3:** Phase 2B (Logging Infrastructure)
  - Milestone: Instruction trace working
- **Day 4-5:** Phase 2C (Interactive Debugger)
  - Milestone: Can set breakpoints and step through code

### Week 2 (Days 6-7)
- **Day 6:** Phase 2D (Memory Dump Utility)
  - Milestone: Can dump and inspect memory regions
- **Day 7:** Phase 2E (Optional - Advanced Features)
  - Milestone: State save/load working (if time permits)

### Final Deliverable (End of Day 7)
- ✅ All core debugging features working
- ✅ Documentation updated
- ✅ Examples added to README
- ✅ Ready to start Phase 3 (CPU implementation) with confidence

---

## Documentation Updates Required

### README.md
Add section:
```markdown
## Debugging

The emulator includes comprehensive debugging tools:

### Basic Debug Mode
\`\`\`bash
cargo run -- --file-path rom.gb --debug --step-limit 100
\`\`\`

### Interactive Debugger
\`\`\`bash
cargo run -- --file-path rom.gb --debugger --break 0x0150
\`\`\`

See [DEBUG.md](DEBUG.md) for full documentation.
```

### Create DEBUG.md
New file with complete debugging guide:
- All CLI flags explained
- Debugger command reference
- Memory map quick reference
- Instruction trace format
- Tips for debugging opcodes
- Common debugging workflows

### Update ROADMAP.md
Mark Phase 2 as complete when done:
```markdown
## Phase 2: Debug & Visualization Infrastructure ✅ (COMPLETED)
```

---

## Next Steps After Phase 2

Once Phase 2 is complete, you'll be ready for:

**Phase 3: CPU Completion**
- Implement remaining 248+ opcodes
- Use debugger to verify each opcode
- Use instruction trace to compare with reference
- Use breakpoints to debug failing test ROMs

**Benefits of Completed Phase 2:**
- Opcode bugs are trivial to identify (see exact register/flag state)
- Can compare traces with BGB or other emulators
- Test ROM failures show exactly where things went wrong
- Development speed increases 5-10x

---

## Appendix A: Debugger Command Reference

| Command | Aliases | Args | Description |
|---------|---------|------|-------------|
| `continue` | `c` | - | Continue execution until next breakpoint |
| `step` | `s`, `n`, `next` | - | Execute one instruction |
| `quit` | `q` | - | Exit emulator |
| `registers` | `r` | - | Show CPU registers and flags |
| `memory` | `m` | `<addr> [len]` | Dump memory region |
| `break` | `b` | `<addr>` | Set breakpoint at address |
| `delete` | `d` | `<addr>` | Remove breakpoint |
| `list` | `l` | - | List all breakpoints |
| `watch` | `w` | `<addr>` | Set memory watchpoint |
| `disassemble` | `dis` | `[count]` | Disassemble next N instructions |
| `help` | `h` | - | Show help message |

---

## Appendix B: Memory Map Quick Reference

| Range | Size | Description | Access |
|-------|------|-------------|--------|
| 0x0000-0x00FF | 256B | Boot ROM (if enabled) | R |
| 0x0000-0x3FFF | 16KB | ROM Bank 0 | R |
| 0x4000-0x7FFF | 16KB | ROM Bank 1-N (switchable) | R |
| 0x8000-0x9FFF | 8KB | Video RAM (VRAM) | RW |
| 0xA000-0xBFFF | 8KB | External RAM | RW |
| 0xC000-0xDFFF | 8KB | Work RAM (WRAM) | RW |
| 0xE000-0xFDFF | ~8KB | Echo RAM (mirrors 0xC000-0xDDFF) | RW |
| 0xFE00-0xFE9F | 160B | Object Attribute Memory (OAM) | RW |
| 0xFEA0-0xFEFF | 96B | Unusable | - |
| 0xFF00-0xFF7F | 128B | I/O Registers | RW |
| 0xFF80-0xFFFE | 127B | High RAM (HRAM) | RW |
| 0xFFFF | 1B | Interrupt Enable Register | RW |

---

## Appendix C: Important I/O Registers

| Address | Name | Description |
|---------|------|-------------|
| 0xFF00 | P1/JOYP | Joypad input |
| 0xFF04 | DIV | Divider register |
| 0xFF05 | TIMA | Timer counter |
| 0xFF06 | TMA | Timer modulo |
| 0xFF07 | TAC | Timer control |
| 0xFF0F | IF | Interrupt flags |
| 0xFF40 | LCDC | LCD control |
| 0xFF41 | STAT | LCD status |
| 0xFF42 | SCY | Scroll Y |
| 0xFF43 | SCX | Scroll X |
| 0xFF44 | LY | LCD Y coordinate |
| 0xFF45 | LYC | LY compare |
| 0xFF46 | DMA | DMA transfer |
| 0xFF47 | BGP | Background palette |
| 0xFF48 | OBP0 | Object palette 0 |
| 0xFF49 | OBP1 | Object palette 1 |
| 0xFF4A | WY | Window Y |
| 0xFF4B | WX | Window X |
| 0xFFFF | IE | Interrupt enable |

---

## Appendix D: Flag Behavior Reference

Game Boy flags (F register bits 7-4):

| Bit | Flag | Name | Set When |
|-----|------|------|----------|
| 7 | Z | Zero | Result is zero |
| 6 | N | Subtract | Instruction was subtraction |
| 5 | H | Half Carry | Carry from bit 3 to 4 |
| 4 | C | Carry | Carry from bit 7 / borrow |
| 3-0 | - | Unused | Always 0 |

**Examples:**
- `ADD A, B` - Sets Z if result is 0, resets N, sets H/C if carry
- `SUB A, B` - Sets Z if result is 0, sets N, sets H/C if borrow
- `XOR A` - Always sets Z, resets N/H/C (common idiom for `LD A, 0`)

---

## Conclusion

Phase 2 is the critical foundation for all future development. While it may feel like a detour from "making progress" on the emulator itself, the debugging infrastructure will save countless hours and make development significantly more enjoyable.

**Investment:** 5-7 days
**Return:** Faster development, fewer bugs, better understanding of Game Boy internals

Once Phase 2 is complete, you'll have a world-class debugging environment that rivals or exceeds commercial emulators like BGB.

**Ready to start? Begin with Phase 2A: Basic State Display!**

---

*Last Updated: 2026-01-07*
*Author: Claude (Anthropic)*
*Status: Ready for Implementation*
