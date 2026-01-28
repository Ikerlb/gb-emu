mod operands;
mod alu;
mod cb_ops;
#[cfg(test)]
mod tests;

pub use operands::{Reg8, Reg16, Reg16Stack, ConditionCode};

use crate::gb::interconnect::Interconnect;
use crate::gb::opcode::Opcode;
use num_traits::FromPrimitive;
use crate::gb::register::Register;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct Cpu {
    // Program counter
    pub(super) reg_pc: u16,
    // Stack pointer (simplified to u16)
    pub(super) reg_sp: u16,

    pub(super) regs_af: Register, // AF Register
    pub(super) regs_bc: Register, // BC Register
    pub(super) regs_de: Register, // DE Register
    pub(super) regs_hl: Register, // HL Register

    pub(super) flags: Flags,

    /// Interrupt Master Enable - when true, interrupts can be serviced
    ime: bool,
    /// Pending IME enable - EI sets this, IME is enabled after next instruction
    ime_pending: bool,
    /// CPU is halted, waiting for interrupt
    halted: bool,
}

#[derive(Debug)]
pub(super) struct Flags {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,
}

impl Display for Flags {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Z:{} N:{} H:{} C:{}",
            self.z as u8, self.n as u8, self.h as u8, self.c as u8)
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "PC:{:04X} SP:{:04X} AF:{} BC:{} DE:{} HL:{} [{}]",
            self.reg_pc,
            self.reg_sp,
            self.regs_af,
            self.regs_bc,
            self.regs_de,
            self.regs_hl,
            self.flags
        )
    }
}

impl Cpu {

    /// Get the program counter value
    pub fn pc(&self) -> u16 {
        self.reg_pc
    }

    /// Get the stack pointer value
    pub fn sp(&self) -> u16 {
        self.reg_sp
    }

    /// Get the AF register value
    pub fn af(&self) -> u16 {
        self.regs_af.get()
    }

    /// Get the BC register value
    pub fn bc(&self) -> u16 {
        self.regs_bc.get()
    }

    /// Get the DE register value
    pub fn de(&self) -> u16 {
        self.regs_de.get()
    }

    /// Get the HL register value
    pub fn hl(&self) -> u16 {
        self.regs_hl.get()
    }

    /// Format CPU state in verbose multi-line format
    pub fn format_verbose(&self) -> String {
        format!(
"=== CPU State ===
Registers:
  PC: 0x{:04X}  SP: 0x{:04X}
  AF: 0x{:04X} (A=0x{:02X}, F=0x{:02X})
  BC: 0x{:04X} (B=0x{:02X}, C=0x{:02X})
  DE: 0x{:04X} (D=0x{:02X}, E=0x{:02X})
  HL: 0x{:04X} (H=0x{:02X}, L=0x{:02X})
Flags:
  Z:{}
  N:{}
  H:{}
  C:{}",
            self.reg_pc,
            self.reg_sp,
            self.regs_af.get(), self.regs_af.get_hi(), self.regs_af.get_lo(),
            self.regs_bc.get(), self.regs_bc.get_hi(), self.regs_bc.get_lo(),
            self.regs_de.get(), self.regs_de.get_hi(), self.regs_de.get_lo(),
            self.regs_hl.get(), self.regs_hl.get_hi(), self.regs_hl.get_lo(),
            self.flags.z as u8,
            self.flags.n as u8,
            self.flags.h as u8,
            self.flags.c as u8
        )
    }

    // Flag setters (used in tests)
    #[allow(dead_code)]
    pub(super) fn set_zero_flag(&mut self, bit: bool) {
        self.flags.z = bit;
    }

    #[allow(dead_code)]
    pub(super) fn set_subtract_flag(&mut self, bit: bool) {
        self.flags.n = bit;
    }

    #[allow(dead_code)]
    pub(super) fn set_half_carry_flag(&mut self, bit: bool) {
        self.flags.h = bit;
    }

    #[allow(dead_code)]
    pub(super) fn set_carry_flag(&mut self, bit: bool) {
        self.flags.c = bit;
    }

    //set regs
    pub(super) fn set_reg_a(&mut self, num: u8) {
        self.regs_af.set_hi(num);
    }

    pub(super) fn set_reg_b(&mut self, num: u8) {
        self.regs_bc.set_hi(num);
    }

    pub(super) fn set_reg_c(&mut self, num: u8) {
        self.regs_bc.set_lo(num);
    }

    pub(super) fn set_reg_d(&mut self, num: u8) {
        self.regs_de.set_hi(num);
    }

    pub(super) fn set_reg_e(&mut self, num: u8) {
        self.regs_de.set_lo(num);
    }

    pub(super) fn set_reg_h(&mut self, num: u8) {
        self.regs_hl.set_hi(num);
    }

    pub(super) fn set_reg_l(&mut self, num: u8) {
        self.regs_hl.set_lo(num);
    }

    pub(super) fn get_reg_a(&self) -> u8 {
        self.regs_af.get_hi()
    }

    pub(super) fn get_reg_b(&self) -> u8 {
        self.regs_bc.get_hi()
    }

    pub(super) fn get_reg_c(&self) -> u8 {
        self.regs_bc.get_lo()
    }

    pub(super) fn get_reg_d(&self) -> u8 {
        self.regs_de.get_hi()
    }

    pub(super) fn get_reg_e(&self) -> u8 {
        self.regs_de.get_lo()
    }

    pub(super) fn get_reg_h(&self) -> u8 {
        self.regs_hl.get_hi()
    }

    pub(super) fn get_reg_l(&self) -> u8 {
        self.regs_hl.get_lo()
    }

    // ========== New Reg8/Reg16 access methods ==========

    /// Read 8-bit register (or memory at (HL) for HLRef)
    pub(super) fn read_r8(&self, inter: &Interconnect, r: Reg8) -> u8 {
        match r {
            Reg8::A => self.regs_af.get_hi(),
            Reg8::B => self.regs_bc.get_hi(),
            Reg8::C => self.regs_bc.get_lo(),
            Reg8::D => self.regs_de.get_hi(),
            Reg8::E => self.regs_de.get_lo(),
            Reg8::H => self.regs_hl.get_hi(),
            Reg8::L => self.regs_hl.get_lo(),
            Reg8::HLRef => inter.read(self.regs_hl.get()),
        }
    }

    /// Write 8-bit register (or memory at (HL) for HLRef)
    pub(super) fn write_r8(&mut self, inter: &mut Interconnect, r: Reg8, val: u8) {
        match r {
            Reg8::A => self.regs_af.set_hi(val),
            Reg8::B => self.regs_bc.set_hi(val),
            Reg8::C => self.regs_bc.set_lo(val),
            Reg8::D => self.regs_de.set_hi(val),
            Reg8::E => self.regs_de.set_lo(val),
            Reg8::H => self.regs_hl.set_hi(val),
            Reg8::L => self.regs_hl.set_lo(val),
            Reg8::HLRef => inter.write(self.regs_hl.get(), val),
        }
    }

    /// Read 16-bit register pair
    fn read_r16(&self, r: Reg16) -> u16 {
        match r {
            Reg16::BC => self.regs_bc.get(),
            Reg16::DE => self.regs_de.get(),
            Reg16::HL => self.regs_hl.get(),
            Reg16::SP => self.reg_sp,
        }
    }

    /// Write 16-bit register pair
    fn write_r16(&mut self, r: Reg16, val: u16) {
        match r {
            Reg16::BC => self.regs_bc.set(val),
            Reg16::DE => self.regs_de.set(val),
            Reg16::HL => self.regs_hl.set(val),
            Reg16::SP => self.reg_sp = val,
        }
    }

    /// Read 16-bit register pair (stack variant - AF instead of SP)
    fn read_r16_stack(&self, r: Reg16Stack) -> u16 {
        match r {
            Reg16Stack::BC => self.regs_bc.get(),
            Reg16Stack::DE => self.regs_de.get(),
            Reg16Stack::HL => self.regs_hl.get(),
            Reg16Stack::AF => {
                // Build AF from A register and flags
                let f = ((self.flags.z as u8) << 7)
                    | ((self.flags.n as u8) << 6)
                    | ((self.flags.h as u8) << 5)
                    | ((self.flags.c as u8) << 4);
                ((self.get_reg_a() as u16) << 8) | (f as u16)
            }
        }
    }

    /// Write 16-bit register pair (stack variant - AF instead of SP)
    fn write_r16_stack(&mut self, r: Reg16Stack, val: u16) {
        match r {
            Reg16Stack::BC => self.regs_bc.set(val),
            Reg16Stack::DE => self.regs_de.set(val),
            Reg16Stack::HL => self.regs_hl.set(val),
            Reg16Stack::AF => {
                // Lower 4 bits of F are always 0
                self.regs_af.set(val & 0xFFF0);
                // Update flags from F register
                let f = val as u8;
                self.flags.z = f & 0x80 != 0;
                self.flags.n = f & 0x40 != 0;
                self.flags.h = f & 0x20 != 0;
                self.flags.c = f & 0x10 != 0;
            }
        }
    }

    /// Check if condition is met
    fn check_condition(&self, cond: ConditionCode) -> bool {
        match cond {
            ConditionCode::NZ => !self.flags.z,
            ConditionCode::Z => self.flags.z,
            ConditionCode::NC => !self.flags.c,
            ConditionCode::C => self.flags.c,
        }
    }

    // ========== Stack operations ==========

    /// Push 16-bit value onto the stack
    fn push_16(&mut self, inter: &mut Interconnect, value: u16) {
        self.reg_sp = self.reg_sp.wrapping_sub(1);
        inter.write(self.reg_sp, (value >> 8) as u8); // High byte
        self.reg_sp = self.reg_sp.wrapping_sub(1);
        inter.write(self.reg_sp, value as u8); // Low byte
    }

    /// Pop 16-bit value from the stack
    fn pop_16(&mut self, inter: &Interconnect) -> u16 {
        let lo = inter.read(self.reg_sp) as u16;
        self.reg_sp = self.reg_sp.wrapping_add(1);
        let hi = inter.read(self.reg_sp) as u16;
        self.reg_sp = self.reg_sp.wrapping_add(1);
        (hi << 8) | lo
    }

    /// Read immediate 8-bit value and advance PC
    pub(super) fn read_imm8(&mut self, inter: &Interconnect) -> u8 {
        let val = inter.read(self.reg_pc);
        self.reg_pc = self.reg_pc.wrapping_add(1);
        val
    }

    /// Read immediate 16-bit value (little-endian) and advance PC
    fn read_imm16(&mut self, inter: &Interconnect) -> u16 {
        let lo = inter.read(self.reg_pc) as u16;
        self.reg_pc = self.reg_pc.wrapping_add(1);
        let hi = inter.read(self.reg_pc) as u16;
        self.reg_pc = self.reg_pc.wrapping_add(1);
        (hi << 8) | lo
    }

    // Initial state taken from codeslinger (as did almost everything that sounds tricky)
    pub fn new() -> Self {
        Cpu {
            reg_pc: 0x0100,
            reg_sp: 0xFFFE,
            regs_af: Register::new(0x01B0),
            regs_bc: Register::new(0x0013),
            regs_de: Register::new(0x00D8),
            regs_hl: Register::new(0x014D),
            flags: Flags { z: true, n: false, h: true, c: true }, // Match 0xB0 in F
            ime: false,        // Interrupts disabled at startup
            ime_pending: false,
            halted: false,
        }
    }

    /// Check if CPU is halted
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Get IME status
    #[allow(dead_code)] // Useful for debugging
    pub fn ime(&self) -> bool {
        self.ime
    }

    /// Handle pending interrupts, returns cycles consumed (0 if no interrupt)
    pub fn handle_interrupts(&mut self, inter: &mut Interconnect) -> usize {
        // Process pending IME enable (from EI instruction)
        if self.ime_pending {
            self.ime_pending = false;
            self.ime = true;
        }

        let pending = inter.pending_interrupts();

        // If halted and there's a pending interrupt, wake up
        if self.halted && pending != 0 {
            self.halted = false;
        }

        // Only service interrupts if IME is enabled
        if !self.ime || pending == 0 {
            return 0;
        }

        // Service highest priority interrupt (lowest bit number)
        // Priority: VBlank > STAT > Timer > Serial > Joypad
        let (interrupt_bit, vector) = if pending & 0x01 != 0 {
            (0x01, 0x0040) // V-Blank
        } else if pending & 0x02 != 0 {
            (0x02, 0x0048) // LCD STAT
        } else if pending & 0x04 != 0 {
            (0x04, 0x0050) // Timer
        } else if pending & 0x08 != 0 {
            (0x08, 0x0058) // Serial
        } else {
            (0x10, 0x0060) // Joypad
        };

        // Disable interrupts
        self.ime = false;

        // Clear the interrupt flag
        inter.clear_interrupt(interrupt_bit);

        // Push PC to stack
        self.push_16(inter, self.reg_pc);

        // Jump to interrupt vector
        self.reg_pc = vector;

        // Interrupt handling takes 20 cycles (5 M-cycles)
        20
    }

    pub fn execute_next_opcode(&mut self, inter: &mut Interconnect) -> usize {
        let op = inter.read(self.reg_pc);
        //wrapping add to prevent overflow
        self.reg_pc = self.reg_pc.wrapping_add(1);
        self.execute_opcode(inter, op)
    }

    pub(super) fn execute_opcode(&mut self, inter: &mut Interconnect, opcode: u8) -> usize {
        let value = Opcode::from_u8(opcode).unwrap_or_else(||
            panic!("Unrecognized Opcode: {:#X}", opcode)
        );

        match value {
            // ===== 0x0X =====
            Opcode::Nop => 4,
            // LD rr, d16 - consolidated using Reg16
            Opcode::Ld_Bc_d16 | Opcode::Ld_De_d16 | Opcode::Ld_Hl_d16 | Opcode::Ld_Sp_d16 => {
                let reg = Reg16::from_bits((opcode >> 4) & 0x03);
                let val = self.read_imm16(inter);
                self.write_r16(reg, val);
                12
            }
            Opcode::Ld_BCref_A => {
                inter.write(self.regs_bc.get(), self.get_reg_a());
                8
            }
            // INC rr - consolidated using Reg16
            Opcode::Inc_Bc | Opcode::Inc_De | Opcode::Inc_Hl | Opcode::Inc_Sp => {
                let reg = Reg16::from_bits((opcode >> 4) & 0x03);
                let val = self.read_r16(reg).wrapping_add(1);
                self.write_r16(reg, val);
                8
            }
            Opcode::Inc_B => {
                let val = self.alu_inc(self.get_reg_b());
                self.set_reg_b(val);
                4
            }
            Opcode::Dec_B => {
                let val = self.alu_dec(self.get_reg_b());
                self.set_reg_b(val);
                4
            }
            Opcode::Ld_B_d8 => {
                let val = self.read_imm8(inter);
                self.set_reg_b(val);
                8
            }
            Opcode::Rlca => {
                let a = self.get_reg_a();
                let c = (a >> 7) & 1;
                let result = (a << 1) | c;
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = c == 1;
                self.set_reg_a(result);
                4
            }
            Opcode::Ld_a16ref_Sp => {
                let addr = self.read_imm16(inter);
                inter.write_16bits(addr, self.reg_sp);
                20
            }
            // ADD HL, rr - consolidated using Reg16
            Opcode::Add_Hl_Bc | Opcode::Add_Hl_De | Opcode::Add_Hl_Hl | Opcode::Add_Hl_Sp => {
                let reg = Reg16::from_bits((opcode >> 4) & 0x03);
                self.alu_add_hl(self.read_r16(reg));
                8
            }
            Opcode::Ld_A_BCref => {
                let val = inter.read(self.regs_bc.get());
                self.set_reg_a(val);
                8
            }
            // DEC rr - consolidated using Reg16
            Opcode::Dec_Bc | Opcode::Dec_De | Opcode::Dec_Hl | Opcode::Dec_Sp => {
                let reg = Reg16::from_bits((opcode >> 4) & 0x03);
                let val = self.read_r16(reg).wrapping_sub(1);
                self.write_r16(reg, val);
                8
            }
            Opcode::Inc_C => {
                let val = self.alu_inc(self.get_reg_c());
                self.set_reg_c(val);
                4
            }
            Opcode::Dec_C => {
                let val = self.alu_dec(self.get_reg_c());
                self.set_reg_c(val);
                4
            }
            Opcode::Ld_C_d8 => {
                let val = self.read_imm8(inter);
                self.set_reg_c(val);
                8
            }
            Opcode::Rrca => {
                let a = self.get_reg_a();
                let c = a & 1;
                let result = (a >> 1) | (c << 7);
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = c == 1;
                self.set_reg_a(result);
                4
            }

            // ===== 0x1X =====
            Opcode::Stop => 4, // TODO: Proper STOP implementation
            Opcode::Ld_DEref_A => {
                inter.write(self.regs_de.get(), self.get_reg_a());
                8
            }
            Opcode::Inc_D => {
                let val = self.alu_inc(self.get_reg_d());
                self.set_reg_d(val);
                4
            }
            Opcode::Dec_D => {
                let val = self.alu_dec(self.get_reg_d());
                self.set_reg_d(val);
                4
            }
            Opcode::Ld_D_d8 => {
                let val = self.read_imm8(inter);
                self.set_reg_d(val);
                8
            }
            Opcode::Rla => {
                let a = self.get_reg_a();
                let old_c = if self.flags.c { 1 } else { 0 };
                let new_c = (a >> 7) & 1;
                let result = (a << 1) | old_c;
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = new_c == 1;
                self.set_reg_a(result);
                4
            }
            Opcode::Jr_r8 => {
                let offset = self.read_imm8(inter) as i8;
                self.reg_pc = self.reg_pc.wrapping_add(offset as u16);
                12
            }
            Opcode::Ld_A_DEref => {
                let val = inter.read(self.regs_de.get());
                self.set_reg_a(val);
                8
            }
            Opcode::Inc_E => {
                let val = self.alu_inc(self.get_reg_e());
                self.set_reg_e(val);
                4
            }
            Opcode::Dec_E => {
                let val = self.alu_dec(self.get_reg_e());
                self.set_reg_e(val);
                4
            }
            Opcode::Ld_E_d8 => {
                let val = self.read_imm8(inter);
                self.set_reg_e(val);
                8
            }
            Opcode::Rra => {
                let a = self.get_reg_a();
                let old_c = if self.flags.c { 0x80 } else { 0 };
                let new_c = a & 1;
                let result = (a >> 1) | old_c;
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = new_c == 1;
                self.set_reg_a(result);
                4
            }

            // ===== 0x2X =====
            // JR cc, r8 - consolidated using ConditionCode
            Opcode::Jr_Nz_r8 | Opcode::Jr_Z_r8 | Opcode::Jr_Nc_r8 | Opcode::Jr_C_r8 => {
                let cond = ConditionCode::from_bits((opcode >> 3) & 0x03);
                let offset = self.read_imm8(inter) as i8;
                if self.check_condition(cond) {
                    self.reg_pc = self.reg_pc.wrapping_add(offset as u16);
                    12
                } else {
                    8
                }
            }
            Opcode::Ld_HLIref_A => {
                inter.write(self.regs_hl.get(), self.get_reg_a());
                self.regs_hl.set(self.regs_hl.get().wrapping_add(1));
                8
            }
            Opcode::Inc_H => {
                let val = self.alu_inc(self.get_reg_h());
                self.set_reg_h(val);
                4
            }
            Opcode::Dec_H => {
                let val = self.alu_dec(self.get_reg_h());
                self.set_reg_h(val);
                4
            }
            Opcode::Ld_H_d8 => {
                let val = self.read_imm8(inter);
                self.set_reg_h(val);
                8
            }
            Opcode::Daa => {
                // DAA - Decimal adjust accumulator
                let mut a = self.get_reg_a();
                if !self.flags.n {
                    // After addition
                    if self.flags.c || a > 0x99 {
                        a = a.wrapping_add(0x60);
                        self.flags.c = true;
                    }
                    if self.flags.h || (a & 0x0F) > 0x09 {
                        a = a.wrapping_add(0x06);
                    }
                } else {
                    // After subtraction
                    if self.flags.c {
                        a = a.wrapping_sub(0x60);
                    }
                    if self.flags.h {
                        a = a.wrapping_sub(0x06);
                    }
                }
                self.flags.z = a == 0;
                self.flags.h = false;
                self.set_reg_a(a);
                4
            }
            Opcode::Ld_A_HLIref => {
                let val = inter.read(self.regs_hl.get());
                self.set_reg_a(val);
                self.regs_hl.set(self.regs_hl.get().wrapping_add(1));
                8
            }
            Opcode::Inc_L => {
                let val = self.alu_inc(self.get_reg_l());
                self.set_reg_l(val);
                4
            }
            Opcode::Dec_L => {
                let val = self.alu_dec(self.get_reg_l());
                self.set_reg_l(val);
                4
            }
            Opcode::Ld_L_d8 => {
                let val = self.read_imm8(inter);
                self.set_reg_l(val);
                8
            }
            Opcode::Cpl => {
                self.set_reg_a(!self.get_reg_a());
                self.flags.n = true;
                self.flags.h = true;
                4
            }

            // ===== 0x3X =====
            Opcode::Ld_HLDref_A => {
                inter.write(self.regs_hl.get(), self.get_reg_a());
                self.regs_hl.set(self.regs_hl.get().wrapping_sub(1));
                8
            }
            Opcode::Inc_HLref => {
                let addr = self.regs_hl.get();
                let val = self.alu_inc(inter.read(addr));
                inter.write(addr, val);
                12
            }
            Opcode::Dec_HLref => {
                let addr = self.regs_hl.get();
                let val = self.alu_dec(inter.read(addr));
                inter.write(addr, val);
                12
            }
            Opcode::Ld_HLref_d8 => {
                let val = self.read_imm8(inter);
                inter.write(self.regs_hl.get(), val);
                12
            }
            Opcode::Scf => {
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = true;
                4
            }
            Opcode::Ld_A_HLDref => {
                let val = inter.read(self.regs_hl.get());
                self.set_reg_a(val);
                self.regs_hl.set(self.regs_hl.get().wrapping_sub(1));
                8
            }
            Opcode::Inc_A => {
                let val = self.alu_inc(self.get_reg_a());
                self.set_reg_a(val);
                4
            }
            Opcode::Dec_A => {
                let val = self.alu_dec(self.get_reg_a());
                self.set_reg_a(val);
                4
            }
            Opcode::Ld_A_d8 => {
                let val = self.read_imm8(inter);
                self.set_reg_a(val);
                8
            }
            Opcode::Ccf => {
                self.flags.n = false;
                self.flags.h = false;
                self.flags.c = !self.flags.c;
                4
            }

            // ===== 0x4X - LD B/C,r =====
            Opcode::Ld_B_B => 4,
            Opcode::Ld_B_C => { self.set_reg_b(self.get_reg_c()); 4 }
            Opcode::Ld_B_D => { self.set_reg_b(self.get_reg_d()); 4 }
            Opcode::Ld_B_E => { self.set_reg_b(self.get_reg_e()); 4 }
            Opcode::Ld_B_H => { self.set_reg_b(self.get_reg_h()); 4 }
            Opcode::Ld_B_L => { self.set_reg_b(self.get_reg_l()); 4 }
            Opcode::Ld_B_HLref => { self.set_reg_b(inter.read(self.regs_hl.get())); 8 }
            Opcode::Ld_B_A => { self.set_reg_b(self.get_reg_a()); 4 }
            Opcode::Ld_C_B => { self.set_reg_c(self.get_reg_b()); 4 }
            Opcode::Ld_C_C => 4,
            Opcode::Ld_C_D => { self.set_reg_c(self.get_reg_d()); 4 }
            Opcode::Ld_C_E => { self.set_reg_c(self.get_reg_e()); 4 }
            Opcode::Ld_C_H => { self.set_reg_c(self.get_reg_h()); 4 }
            Opcode::Ld_C_L => { self.set_reg_c(self.get_reg_l()); 4 }
            Opcode::Ld_C_HLref => { self.set_reg_c(inter.read(self.regs_hl.get())); 8 }
            Opcode::Ld_C_A => { self.set_reg_c(self.get_reg_a()); 4 }

            // ===== 0x5X - LD D/E,r =====
            Opcode::Ld_D_B => { self.set_reg_d(self.get_reg_b()); 4 }
            Opcode::Ld_D_C => { self.set_reg_d(self.get_reg_c()); 4 }
            Opcode::Ld_D_D => 4,
            Opcode::Ld_D_E => { self.set_reg_d(self.get_reg_e()); 4 }
            Opcode::Ld_D_H => { self.set_reg_d(self.get_reg_h()); 4 }
            Opcode::Ld_D_L => { self.set_reg_d(self.get_reg_l()); 4 }
            Opcode::Ld_D_HLref => { self.set_reg_d(inter.read(self.regs_hl.get())); 8 }
            Opcode::Ld_D_A => { self.set_reg_d(self.get_reg_a()); 4 }
            Opcode::Ld_E_B => { self.set_reg_e(self.get_reg_b()); 4 }
            Opcode::Ld_E_C => { self.set_reg_e(self.get_reg_c()); 4 }
            Opcode::Ld_E_D => { self.set_reg_e(self.get_reg_d()); 4 }
            Opcode::Ld_E_E => 4,
            Opcode::Ld_E_H => { self.set_reg_e(self.get_reg_h()); 4 }
            Opcode::Ld_E_L => { self.set_reg_e(self.get_reg_l()); 4 }
            Opcode::Ld_E_HLref => { self.set_reg_e(inter.read(self.regs_hl.get())); 8 }
            Opcode::Ld_E_A => { self.set_reg_e(self.get_reg_a()); 4 }

            // ===== 0x6X - LD H/L,r =====
            Opcode::Ld_H_B => { self.set_reg_h(self.get_reg_b()); 4 }
            Opcode::Ld_H_C => { self.set_reg_h(self.get_reg_c()); 4 }
            Opcode::Ld_H_D => { self.set_reg_h(self.get_reg_d()); 4 }
            Opcode::Ld_H_E => { self.set_reg_h(self.get_reg_e()); 4 }
            Opcode::Ld_H_H => 4,
            Opcode::Ld_H_L => { self.set_reg_h(self.get_reg_l()); 4 }
            Opcode::Ld_H_HLref => { self.set_reg_h(inter.read(self.regs_hl.get())); 8 }
            Opcode::Ld_H_A => { self.set_reg_h(self.get_reg_a()); 4 }
            Opcode::Ld_L_B => { self.set_reg_l(self.get_reg_b()); 4 }
            Opcode::Ld_L_C => { self.set_reg_l(self.get_reg_c()); 4 }
            Opcode::Ld_L_D => { self.set_reg_l(self.get_reg_d()); 4 }
            Opcode::Ld_L_E => { self.set_reg_l(self.get_reg_e()); 4 }
            Opcode::Ld_L_H => { self.set_reg_l(self.get_reg_h()); 4 }
            Opcode::Ld_L_L => 4,
            Opcode::Ld_L_HLref => { self.set_reg_l(inter.read(self.regs_hl.get())); 8 }
            Opcode::Ld_L_A => { self.set_reg_l(self.get_reg_a()); 4 }

            // ===== 0x7X - LD (HL)/A,r =====
            Opcode::Ld_HLref_B => { inter.write(self.regs_hl.get(), self.get_reg_b()); 8 }
            Opcode::Ld_HLref_C => { inter.write(self.regs_hl.get(), self.get_reg_c()); 8 }
            Opcode::Ld_HLref_D => { inter.write(self.regs_hl.get(), self.get_reg_d()); 8 }
            Opcode::Ld_HLref_E => { inter.write(self.regs_hl.get(), self.get_reg_e()); 8 }
            Opcode::Ld_HLref_H => { inter.write(self.regs_hl.get(), self.get_reg_h()); 8 }
            Opcode::Ld_HLref_L => { inter.write(self.regs_hl.get(), self.get_reg_l()); 8 }
            Opcode::Halt => {
                self.halted = true;
                4
            }
            Opcode::Ld_HLref_A => { inter.write(self.regs_hl.get(), self.get_reg_a()); 8 }
            Opcode::Ld_A_B => { self.set_reg_a(self.get_reg_b()); 4 }
            Opcode::Ld_A_C => { self.set_reg_a(self.get_reg_c()); 4 }
            Opcode::Ld_A_D => { self.set_reg_a(self.get_reg_d()); 4 }
            Opcode::Ld_A_E => { self.set_reg_a(self.get_reg_e()); 4 }
            Opcode::Ld_A_H => { self.set_reg_a(self.get_reg_h()); 4 }
            Opcode::Ld_A_L => { self.set_reg_a(self.get_reg_l()); 4 }
            Opcode::Ld_A_HLref => { self.set_reg_a(inter.read(self.regs_hl.get())); 8 }
            Opcode::Ld_A_A => 4,

            // ===== 0x8X - ADD/ADC A,r =====
            Opcode::Add_A_B => { self.alu_add(self.get_reg_b()); 4 }
            Opcode::Add_A_C => { self.alu_add(self.get_reg_c()); 4 }
            Opcode::Add_A_D => { self.alu_add(self.get_reg_d()); 4 }
            Opcode::Add_A_E => { self.alu_add(self.get_reg_e()); 4 }
            Opcode::Add_A_H => { self.alu_add(self.get_reg_h()); 4 }
            Opcode::Add_A_L => { self.alu_add(self.get_reg_l()); 4 }
            Opcode::Add_A_HLref => { self.alu_add(inter.read(self.regs_hl.get())); 8 }
            Opcode::Add_A_A => { self.alu_add(self.get_reg_a()); 4 }
            Opcode::Adc_A_B => { self.alu_adc(self.get_reg_b()); 4 }
            Opcode::Adc_A_C => { self.alu_adc(self.get_reg_c()); 4 }
            Opcode::Adc_A_D => { self.alu_adc(self.get_reg_d()); 4 }
            Opcode::Adc_A_E => { self.alu_adc(self.get_reg_e()); 4 }
            Opcode::Adc_A_H => { self.alu_adc(self.get_reg_h()); 4 }
            Opcode::Adc_A_L => { self.alu_adc(self.get_reg_l()); 4 }
            Opcode::Adc_A_HLref => { self.alu_adc(inter.read(self.regs_hl.get())); 8 }
            Opcode::Adc_A_A => { self.alu_adc(self.get_reg_a()); 4 }

            // ===== 0x9X - SUB/SBC A,r =====
            Opcode::Sub_A_B => { self.alu_sub(self.get_reg_b()); 4 }
            Opcode::Sub_A_C => { self.alu_sub(self.get_reg_c()); 4 }
            Opcode::Sub_A_D => { self.alu_sub(self.get_reg_d()); 4 }
            Opcode::Sub_A_E => { self.alu_sub(self.get_reg_e()); 4 }
            Opcode::Sub_A_H => { self.alu_sub(self.get_reg_h()); 4 }
            Opcode::Sub_A_L => { self.alu_sub(self.get_reg_l()); 4 }
            Opcode::Sub_A_HLref => { self.alu_sub(inter.read(self.regs_hl.get())); 8 }
            Opcode::Sub_A_A => { self.alu_sub(self.get_reg_a()); 4 }
            Opcode::Sbc_A_B => { self.alu_sbc(self.get_reg_b()); 4 }
            Opcode::Sbc_A_C => { self.alu_sbc(self.get_reg_c()); 4 }
            Opcode::Sbc_A_D => { self.alu_sbc(self.get_reg_d()); 4 }
            Opcode::Sbc_A_E => { self.alu_sbc(self.get_reg_e()); 4 }
            Opcode::Sbc_A_H => { self.alu_sbc(self.get_reg_h()); 4 }
            Opcode::Sbc_A_L => { self.alu_sbc(self.get_reg_l()); 4 }
            Opcode::Sbc_A_HLref => { self.alu_sbc(inter.read(self.regs_hl.get())); 8 }
            Opcode::Sbc_A_A => { self.alu_sbc(self.get_reg_a()); 4 }

            // ===== 0xAX - AND/XOR A,r =====
            Opcode::And_A_B => { self.alu_and(self.get_reg_b()); 4 }
            Opcode::And_A_C => { self.alu_and(self.get_reg_c()); 4 }
            Opcode::And_A_D => { self.alu_and(self.get_reg_d()); 4 }
            Opcode::And_A_E => { self.alu_and(self.get_reg_e()); 4 }
            Opcode::And_A_H => { self.alu_and(self.get_reg_h()); 4 }
            Opcode::And_A_L => { self.alu_and(self.get_reg_l()); 4 }
            Opcode::And_A_HLref => { self.alu_and(inter.read(self.regs_hl.get())); 8 }
            Opcode::And_A_A => { self.alu_and(self.get_reg_a()); 4 }
            Opcode::Xor_A_B => { self.alu_xor(self.get_reg_b()); 4 }
            Opcode::Xor_A_C => { self.alu_xor(self.get_reg_c()); 4 }
            Opcode::Xor_A_D => { self.alu_xor(self.get_reg_d()); 4 }
            Opcode::Xor_A_E => { self.alu_xor(self.get_reg_e()); 4 }
            Opcode::Xor_A_H => { self.alu_xor(self.get_reg_h()); 4 }
            Opcode::Xor_A_L => { self.alu_xor(self.get_reg_l()); 4 }
            Opcode::Xor_A_HLref => { self.alu_xor(inter.read(self.regs_hl.get())); 8 }
            Opcode::Xor_A_A => { self.alu_xor(self.get_reg_a()); 4 }

            // ===== 0xBX - OR/CP A,r =====
            Opcode::Or_A_B => { self.alu_or(self.get_reg_b()); 4 }
            Opcode::Or_A_C => { self.alu_or(self.get_reg_c()); 4 }
            Opcode::Or_A_D => { self.alu_or(self.get_reg_d()); 4 }
            Opcode::Or_A_E => { self.alu_or(self.get_reg_e()); 4 }
            Opcode::Or_A_H => { self.alu_or(self.get_reg_h()); 4 }
            Opcode::Or_A_L => { self.alu_or(self.get_reg_l()); 4 }
            Opcode::Or_A_HLref => { self.alu_or(inter.read(self.regs_hl.get())); 8 }
            Opcode::Or_A_A => { self.alu_or(self.get_reg_a()); 4 }
            Opcode::Cp_A_B => { self.alu_cp(self.get_reg_b()); 4 }
            Opcode::Cp_A_C => { self.alu_cp(self.get_reg_c()); 4 }
            Opcode::Cp_A_D => { self.alu_cp(self.get_reg_d()); 4 }
            Opcode::Cp_A_E => { self.alu_cp(self.get_reg_e()); 4 }
            Opcode::Cp_A_H => { self.alu_cp(self.get_reg_h()); 4 }
            Opcode::Cp_A_L => { self.alu_cp(self.get_reg_l()); 4 }
            Opcode::Cp_A_HLref => { self.alu_cp(inter.read(self.regs_hl.get())); 8 }
            Opcode::Cp_A_A => { self.alu_cp(self.get_reg_a()); 4 }

            // ===== 0xCX - Control flow, stack, immediate ALU =====
            // RET cc - consolidated using ConditionCode
            Opcode::Ret_Nz | Opcode::Ret_Z | Opcode::Ret_Nc | Opcode::Ret_C => {
                let cond = ConditionCode::from_bits((opcode >> 3) & 0x03);
                if self.check_condition(cond) {
                    self.reg_pc = self.pop_16(inter);
                    20
                } else {
                    8
                }
            }
            // POP rr - consolidated using Reg16Stack
            Opcode::Pop_Bc | Opcode::Pop_De | Opcode::Pop_Hl | Opcode::Pop_Af => {
                let reg = Reg16Stack::from_bits((opcode >> 4) & 0x03);
                let val = self.pop_16(inter);
                self.write_r16_stack(reg, val);
                12
            }
            // JP cc, a16 - consolidated using ConditionCode
            Opcode::Jp_Nz_a16 | Opcode::Jp_Z_a16 | Opcode::Jp_Nc_a16 | Opcode::Jp_C_a16 => {
                let cond = ConditionCode::from_bits((opcode >> 3) & 0x03);
                let addr = self.read_imm16(inter);
                if self.check_condition(cond) {
                    self.reg_pc = addr;
                    16
                } else {
                    12
                }
            }
            Opcode::Jp_a16 => {
                self.reg_pc = self.read_imm16(inter);
                16
            }
            // CALL cc, a16 - consolidated using ConditionCode
            Opcode::Call_Nz_a16 | Opcode::Call_Z_a16 | Opcode::Call_Nc_a16 | Opcode::Call_C_a16 => {
                let cond = ConditionCode::from_bits((opcode >> 3) & 0x03);
                let addr = self.read_imm16(inter);
                if self.check_condition(cond) {
                    self.push_16(inter, self.reg_pc);
                    self.reg_pc = addr;
                    24
                } else {
                    12
                }
            }
            // PUSH rr - consolidated using Reg16Stack
            Opcode::Push_Bc | Opcode::Push_De | Opcode::Push_Hl | Opcode::Push_Af => {
                let reg = Reg16Stack::from_bits((opcode >> 4) & 0x03);
                self.push_16(inter, self.read_r16_stack(reg));
                16
            }
            Opcode::Add_A_d8 => {
                let val = self.read_imm8(inter);
                self.alu_add(val);
                8
            }
            Opcode::Rst_00 => {
                self.push_16(inter, self.reg_pc);
                self.reg_pc = 0x0000;
                16
            }
            Opcode::Ret => {
                self.reg_pc = self.pop_16(inter);
                16
            }
            Opcode::Prefix_Cb => {
                self.execute_cb(inter)
            }
            Opcode::Call_a16 => {
                let addr = self.read_imm16(inter);
                self.push_16(inter, self.reg_pc);
                self.reg_pc = addr;
                24
            }
            Opcode::Adc_A_d8 => {
                let val = self.read_imm8(inter);
                self.alu_adc(val);
                8
            }
            Opcode::Rst_08 => {
                self.push_16(inter, self.reg_pc);
                self.reg_pc = 0x0008;
                16
            }

            // ===== 0xDX - More control flow =====
            Opcode::Sub_A_d8 => {
                let val = self.read_imm8(inter);
                self.alu_sub(val);
                8
            }
            Opcode::Rst_10 => {
                self.push_16(inter, self.reg_pc);
                self.reg_pc = 0x0010;
                16
            }
            Opcode::Reti => {
                self.ime = true; // Enable interrupts immediately
                self.reg_pc = self.pop_16(inter);
                16
            }
            Opcode::Sbc_A_d8 => {
                let val = self.read_imm8(inter);
                self.alu_sbc(val);
                8
            }
            Opcode::Rst_18 => {
                self.push_16(inter, self.reg_pc);
                self.reg_pc = 0x0018;
                16
            }

            // ===== 0xEX - I/O, stack, JP =====
            Opcode::Ldh_a8ref_A => {
                let offset = self.read_imm8(inter) as u16;
                inter.write(0xFF00 + offset, self.get_reg_a());
                12
            }
            Opcode::Ld_Cref_A => {
                inter.write(0xFF00 + self.get_reg_c() as u16, self.get_reg_a());
                8
            }
            Opcode::And_A_d8 => {
                let val = self.read_imm8(inter);
                self.alu_and(val);
                8
            }
            Opcode::Rst_20 => {
                self.push_16(inter, self.reg_pc);
                self.reg_pc = 0x0020;
                16
            }
            Opcode::Add_Sp_r8 => {
                let offset = self.read_imm8(inter) as i8 as i16 as u16;
                let sp = self.reg_sp;
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = (sp & 0x0F) + (offset & 0x0F) > 0x0F;
                self.flags.c = (sp & 0xFF) + (offset & 0xFF) > 0xFF;
                self.reg_sp = sp.wrapping_add(offset);
                16
            }
            Opcode::Jp_Hl => {
                self.reg_pc = self.regs_hl.get();
                4
            }
            Opcode::Ld_a16ref_A => {
                let addr = self.read_imm16(inter);
                inter.write(addr, self.get_reg_a());
                16
            }
            Opcode::Xor_A_d8 => {
                let val = self.read_imm8(inter);
                self.alu_xor(val);
                8
            }
            Opcode::Rst_28 => {
                self.push_16(inter, self.reg_pc);
                self.reg_pc = 0x0028;
                16
            }

            // ===== 0xFX - I/O, stack, misc =====
            Opcode::Ldh_A_a8ref => {
                let offset = self.read_imm8(inter) as u16;
                self.set_reg_a(inter.read(0xFF00 + offset));
                12
            }
            Opcode::Ld_A_Cref => {
                self.set_reg_a(inter.read(0xFF00 + self.get_reg_c() as u16));
                8
            }
            Opcode::Di => {
                self.ime = false;
                self.ime_pending = false;
                4
            }
            Opcode::Or_A_d8 => {
                let val = self.read_imm8(inter);
                self.alu_or(val);
                8
            }
            Opcode::Rst_30 => {
                self.push_16(inter, self.reg_pc);
                self.reg_pc = 0x0030;
                16
            }
            Opcode::Ld_Hl_SpPlusr8 => {
                let offset = self.read_imm8(inter) as i8 as i16 as u16;
                let sp = self.reg_sp;
                self.flags.z = false;
                self.flags.n = false;
                self.flags.h = (sp & 0x0F) + (offset & 0x0F) > 0x0F;
                self.flags.c = (sp & 0xFF) + (offset & 0xFF) > 0xFF;
                self.regs_hl.set(sp.wrapping_add(offset));
                12
            }
            Opcode::Ld_Sp_Hl => {
                self.reg_sp = self.regs_hl.get();
                8
            }
            Opcode::Ld_A_a16ref => {
                let addr = self.read_imm16(inter);
                self.set_reg_a(inter.read(addr));
                16
            }
            Opcode::Ei => {
                // EI enables interrupts after the next instruction
                self.ime_pending = true;
                4
            }
            Opcode::Cp_A_d8 => {
                let val = self.read_imm8(inter);
                self.alu_cp(val);
                8
            }
            Opcode::Rst_38 => {
                self.push_16(inter, self.reg_pc);
                self.reg_pc = 0x0038;
                16
            }
        }
    }
}
