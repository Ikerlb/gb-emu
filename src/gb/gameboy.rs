use crate::gb::cpu::*;
use crate::gb::debug::DebugConfig;
use crate::gb::interconnect::*;
use crate::gb::opcode::Opcode;
use num_traits::FromPrimitive;
use std::fmt::{Display, Formatter, Result};

pub struct GameBoy {
    cpu: Cpu,
    interconnect: Interconnect,
    debug_config: DebugConfig,
    instruction_count: u64,
}

impl GameBoy {
    pub fn new(cart: Vec<u8>, debug_config: DebugConfig) -> Self {
        GameBoy {
            cpu: Cpu::new(),
            interconnect: Interconnect::new(cart),
            debug_config,
            instruction_count: 0,
        }
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn interconnect(&self) -> &Interconnect {
        &self.interconnect
    }

    pub fn run(&mut self) {
        loop {
            // Check max instructions limit
            if let Some(max) = self.debug_config.max_instructions {
                if self.instruction_count >= max {
                    if self.debug_config.enabled {
                        println!("Reached max instructions limit: {}", max);
                    }
                    break;
                }
            }

            // Get current instruction for debug output before execution
            let pc_before = self.cpu.pc();
            let opcode_byte = self.interconnect.read(pc_before);
            let opcode = Opcode::from_u8(opcode_byte);

            // Execute instruction
            self.cpu.execute_next_opcode(&mut self.interconnect);
            self.instruction_count += 1;

            // Debug output (only if enabled)
            if self.debug_config.enabled {
                self.print_debug_state(pc_before, opcode_byte, opcode.as_ref());
            }
        }
    }

    /// Execute a single instruction. Returns true if CPU is halted.
    pub fn step(&mut self) -> bool {
        self.cpu.execute_next_opcode(&mut self.interconnect);
        self.instruction_count += 1;
        // TODO: Return actual halt state when HALT opcode is implemented
        false
    }

    fn print_debug_state(&self, pc: u16, opcode_byte: u8, opcode: Option<&Opcode>) {
        let opcode_str = match opcode {
            Some(op) => format!("{}", op),
            None => format!("??? (0x{:02X})", opcode_byte),
        };

        if self.debug_config.verbose {
            println!("{}", self.cpu.format_verbose());
            println!("Instruction: {} at 0x{:04X}", opcode_str, pc);
            println!("Count: {}\n", self.instruction_count);
        } else {
            // Compact single-line format
            println!("{} | {} @{:04X} [#{}]",
                self.cpu, opcode_str, pc, self.instruction_count);
        }
    }
}

// Display trait for GameBoy (for external use)
impl Display for GameBoy {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.cpu)
    }
}
