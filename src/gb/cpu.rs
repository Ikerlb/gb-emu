use crate::gb::interconnect::*;
use crate::gb::opcode::Opcode;
use num_traits::FromPrimitive;
use crate::gb::register::Register;
use std::fmt::{Display, Formatter, Result};

/// 8-bit register identifiers, matching Game Boy opcode encoding
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum Reg8 {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    HLRef = 6, // (HL) - memory reference
    A = 7,
}

impl Reg8 {
    /// Decode 3-bit register field from opcode
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x07 {
            0 => Reg8::B,
            1 => Reg8::C,
            2 => Reg8::D,
            3 => Reg8::E,
            4 => Reg8::H,
            5 => Reg8::L,
            6 => Reg8::HLRef,
            7 => Reg8::A,
            _ => unreachable!(),
        }
    }
}

/// 16-bit register identifiers
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reg16 {
    BC,
    DE,
    HL,
    SP,
}

impl Reg16 {
    /// Decode 2-bit register pair field from opcode (for LD rr,d16, INC/DEC rr, etc.)
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Reg16::BC,
            1 => Reg16::DE,
            2 => Reg16::HL,
            3 => Reg16::SP,
            _ => unreachable!(),
        }
    }
}

/// 16-bit register identifiers for PUSH/POP (uses AF instead of SP)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Reg16Stack {
    BC,
    DE,
    HL,
    AF,
}

impl Reg16Stack {
    /// Decode 2-bit register pair field from opcode for stack operations
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Reg16Stack::BC,
            1 => Reg16Stack::DE,
            2 => Reg16Stack::HL,
            3 => Reg16Stack::AF,
            _ => unreachable!(),
        }
    }
}

/// Condition codes for conditional jumps/calls/returns
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Condition {
    NZ, // Not Zero
    Z,  // Zero
    NC, // Not Carry
    C,  // Carry
}

impl Condition {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Condition::NZ,
            1 => Condition::Z,
            2 => Condition::NC,
            3 => Condition::C,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    // Program counter
    reg_pc: u16,
    // Stack pointer (simplified to u16)
    reg_sp: u16,

    regs_af: Register, // AF Register
    regs_bc: Register, // BC Register
    regs_de: Register, // DE Register
    regs_hl: Register, // HL Register

    flags: Flags,

    /// Interrupt Master Enable - when true, interrupts can be serviced
    ime: bool,
    /// Pending IME enable - EI sets this, IME is enabled after next instruction
    ime_pending: bool,
    /// CPU is halted, waiting for interrupt
    halted: bool,
}

#[derive(Debug)]
struct Flags{
	z: bool,
	n: bool,
	h: bool,
	c: bool
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
	fn set_zero_flag(&mut self, bit: bool) {
		self.flags.z = bit;
	}

	#[allow(dead_code)]
	fn set_subtract_flag(&mut self, bit: bool) {
		self.flags.n = bit;
	}

	#[allow(dead_code)]
	fn set_half_carry_flag(&mut self, bit: bool) {
		self.flags.h = bit;
	}

	#[allow(dead_code)]
	fn set_carry_flag(&mut self, bit: bool) {
		self.flags.c = bit;
	}

	//set regs
	fn set_reg_a(&mut self,num: u8){
		self.regs_af.set_hi(num);
	}

	fn set_reg_b(&mut self,num: u8){
		self.regs_bc.set_hi(num);
	}

	fn set_reg_c(&mut self,num: u8){
		self.regs_bc.set_lo(num);
	}

	fn set_reg_d(&mut self,num: u8){
		self.regs_de.set_hi(num);
	}

	fn set_reg_e(&mut self,num: u8){
		self.regs_de.set_lo(num);
	}

	fn set_reg_h(&mut self,num: u8){
		self.regs_hl.set_hi(num);
	}

	fn set_reg_l(&mut self,num: u8){
		self.regs_hl.set_lo(num);
	}

	fn get_reg_a(&self) -> u8 {
		self.regs_af.get_hi()
	}

	fn get_reg_b(&self) -> u8 {
		self.regs_bc.get_hi()
	}

	fn get_reg_c(&self) -> u8 {
		self.regs_bc.get_lo()
	}

	fn get_reg_d(&self) -> u8 {
		self.regs_de.get_hi()
	}

	fn get_reg_e(&self) -> u8 {
		self.regs_de.get_lo()
	}

	fn get_reg_h(&self) -> u8 {
		self.regs_hl.get_hi()
	}

	fn get_reg_l(&self) -> u8 {
		self.regs_hl.get_lo()
	}

	// ========== New Reg8/Reg16 access methods ==========

	/// Read 8-bit register (or memory at (HL) for HLRef)
	fn read_r8(&self, inter: &Interconnect, r: Reg8) -> u8 {
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
	fn write_r8(&mut self, inter: &mut Interconnect, r: Reg8, val: u8) {
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
	fn check_condition(&self, cond: Condition) -> bool {
		match cond {
			Condition::NZ => !self.flags.z,
			Condition::Z => self.flags.z,
			Condition::NC => !self.flags.c,
			Condition::C => self.flags.c,
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

	// ========== ALU operations ==========

	/// ADD A, value - Sets Z=*, N=0, H=*, C=*
	fn alu_add(&mut self, value: u8) {
		let a = self.get_reg_a();
		let result = a.wrapping_add(value);
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = (a & 0x0F) + (value & 0x0F) > 0x0F;
		self.flags.c = (a as u16) + (value as u16) > 0xFF;
		self.set_reg_a(result);
	}

	/// ADC A, value - Add with carry. Sets Z=*, N=0, H=*, C=*
	fn alu_adc(&mut self, value: u8) {
		let a = self.get_reg_a();
		let carry = if self.flags.c { 1u8 } else { 0u8 };
		let result = a.wrapping_add(value).wrapping_add(carry);
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = (a & 0x0F) + (value & 0x0F) + carry > 0x0F;
		self.flags.c = (a as u16) + (value as u16) + (carry as u16) > 0xFF;
		self.set_reg_a(result);
	}

	/// SUB A, value - Sets Z=*, N=1, H=*, C=*
	fn alu_sub(&mut self, value: u8) {
		let a = self.get_reg_a();
		let result = a.wrapping_sub(value);
		self.flags.z = result == 0;
		self.flags.n = true;
		self.flags.h = (a & 0x0F) < (value & 0x0F);
		self.flags.c = a < value;
		self.set_reg_a(result);
	}

	/// SBC A, value - Subtract with carry. Sets Z=*, N=1, H=*, C=*
	fn alu_sbc(&mut self, value: u8) {
		let a = self.get_reg_a();
		let carry = if self.flags.c { 1u8 } else { 0u8 };
		let result = a.wrapping_sub(value).wrapping_sub(carry);
		self.flags.z = result == 0;
		self.flags.n = true;
		self.flags.h = (a & 0x0F) < (value & 0x0F) + carry;
		self.flags.c = (a as u16) < (value as u16) + (carry as u16);
		self.set_reg_a(result);
	}

	/// AND A, value - Sets Z=*, N=0, H=1, C=0
	fn alu_and(&mut self, value: u8) {
		let result = self.get_reg_a() & value;
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = true;
		self.flags.c = false;
		self.set_reg_a(result);
	}

	/// XOR A, value - Sets Z=*, N=0, H=0, C=0
	fn alu_xor(&mut self, value: u8) {
		let result = self.get_reg_a() ^ value;
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = false;
		self.set_reg_a(result);
	}

	/// OR A, value - Sets Z=*, N=0, H=0, C=0
	fn alu_or(&mut self, value: u8) {
		let result = self.get_reg_a() | value;
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = false;
		self.set_reg_a(result);
	}

	/// CP A, value - Compare (like SUB but discard result). Sets Z=*, N=1, H=*, C=*
	fn alu_cp(&mut self, value: u8) {
		let a = self.get_reg_a();
		self.flags.z = a == value;
		self.flags.n = true;
		self.flags.h = (a & 0x0F) < (value & 0x0F);
		self.flags.c = a < value;
	}

	// ========== CB-prefix operations ==========

	/// RLC - Rotate left, old bit 7 to carry. Z=*, N=0, H=0, C=*
	fn cb_rlc(&mut self, value: u8) -> u8 {
		let carry = (value >> 7) & 1;
		let result = (value << 1) | carry;
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = carry == 1;
		result
	}

	/// RRC - Rotate right, old bit 0 to carry. Z=*, N=0, H=0, C=*
	fn cb_rrc(&mut self, value: u8) -> u8 {
		let carry = value & 1;
		let result = (value >> 1) | (carry << 7);
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = carry == 1;
		result
	}

	/// RL - Rotate left through carry. Z=*, N=0, H=0, C=*
	fn cb_rl(&mut self, value: u8) -> u8 {
		let old_carry = if self.flags.c { 1 } else { 0 };
		let new_carry = (value >> 7) & 1;
		let result = (value << 1) | old_carry;
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = new_carry == 1;
		result
	}

	/// RR - Rotate right through carry. Z=*, N=0, H=0, C=*
	fn cb_rr(&mut self, value: u8) -> u8 {
		let old_carry = if self.flags.c { 1u8 } else { 0 };
		let new_carry = value & 1;
		let result = (value >> 1) | (old_carry << 7);
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = new_carry == 1;
		result
	}

	/// SLA - Shift left arithmetic (bit 0 = 0). Z=*, N=0, H=0, C=*
	fn cb_sla(&mut self, value: u8) -> u8 {
		let carry = (value >> 7) & 1;
		let result = value << 1;
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = carry == 1;
		result
	}

	/// SRA - Shift right arithmetic (bit 7 unchanged). Z=*, N=0, H=0, C=*
	fn cb_sra(&mut self, value: u8) -> u8 {
		let carry = value & 1;
		let result = (value >> 1) | (value & 0x80); // Preserve bit 7
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = carry == 1;
		result
	}

	/// SWAP - Swap upper and lower nibbles. Z=*, N=0, H=0, C=0
	fn cb_swap(&mut self, value: u8) -> u8 {
		let result = (value >> 4) | (value << 4);
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = false;
		result
	}

	/// SRL - Shift right logical (bit 7 = 0). Z=*, N=0, H=0, C=*
	fn cb_srl(&mut self, value: u8) -> u8 {
		let carry = value & 1;
		let result = value >> 1;
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = false;
		self.flags.c = carry == 1;
		result
	}

	/// BIT - Test bit n. Z=*, N=0, H=1, C=-
	fn cb_bit(&mut self, bit: u8, value: u8) {
		self.flags.z = (value & (1 << bit)) == 0;
		self.flags.n = false;
		self.flags.h = true;
		// C flag unchanged
	}

	/// RES - Reset bit n. No flags affected.
	fn cb_res(&self, bit: u8, value: u8) -> u8 {
		value & !(1 << bit)
	}

	/// SET - Set bit n. No flags affected.
	fn cb_set(&self, bit: u8, value: u8) -> u8 {
		value | (1 << bit)
	}

	/// Execute CB-prefixed opcode, returns cycle count
	fn execute_cb(&mut self, inter: &mut Interconnect) -> usize {
		let cb_op = self.read_imm8(inter);
		let reg = Reg8::from_bits(cb_op & 0x07);
		let is_hl_ref = reg == Reg8::HLRef;

		// Operations 0x40-0xFF use bits 5-3 as bit number
		let bit = (cb_op >> 3) & 0x07;

		match cb_op {
			// RLC r (0x00-0x07)
			0x00..=0x07 => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_rlc(val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
			// RRC r (0x08-0x0F)
			0x08..=0x0F => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_rrc(val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
			// RL r (0x10-0x17)
			0x10..=0x17 => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_rl(val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
			// RR r (0x18-0x1F)
			0x18..=0x1F => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_rr(val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
			// SLA r (0x20-0x27)
			0x20..=0x27 => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_sla(val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
			// SRA r (0x28-0x2F)
			0x28..=0x2F => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_sra(val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
			// SWAP r (0x30-0x37)
			0x30..=0x37 => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_swap(val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
			// SRL r (0x38-0x3F)
			0x38..=0x3F => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_srl(val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
			// BIT n, r (0x40-0x7F)
			0x40..=0x7F => {
				let val = self.read_r8(inter, reg);
				self.cb_bit(bit, val);
				if is_hl_ref { 12 } else { 8 }
			}
			// RES n, r (0x80-0xBF)
			0x80..=0xBF => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_res(bit, val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
			// SET n, r (0xC0-0xFF)
			0xC0..=0xFF => {
				let val = self.read_r8(inter, reg);
				let result = self.cb_set(bit, val);
				self.write_r8(inter, reg, result);
				if is_hl_ref { 16 } else { 8 }
			}
		}
	}

	/// Read immediate 8-bit value and advance PC
	fn read_imm8(&mut self, inter: &Interconnect) -> u8 {
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

	pub fn execute_next_opcode(&mut self,inter:&mut Interconnect)->usize{
		let op = inter.read(self.reg_pc);
		//wrapping add to prevent overflow
		self.reg_pc=self.reg_pc.wrapping_add(1);
		self.execute_opcode(inter,op)
	}

	/// 8-bit INC helper - sets Z, N=0, H, C unchanged
	fn alu_inc(&mut self, val: u8) -> u8 {
		let result = val.wrapping_add(1);
		self.flags.z = result == 0;
		self.flags.n = false;
		self.flags.h = (val & 0x0F) == 0x0F;
		result
	}

	/// 8-bit DEC helper - sets Z, N=1, H, C unchanged
	fn alu_dec(&mut self, val: u8) -> u8 {
		let result = val.wrapping_sub(1);
		self.flags.z = result == 0;
		self.flags.n = true;
		self.flags.h = (val & 0x0F) == 0x00;
		result
	}

	/// 16-bit ADD HL,rr helper - sets N=0, H, C (Z unchanged)
	fn alu_add_hl(&mut self, val: u16) {
		let hl = self.regs_hl.get();
		let result = hl.wrapping_add(val);
		self.flags.n = false;
		self.flags.h = (hl & 0x0FFF) + (val & 0x0FFF) > 0x0FFF;
		self.flags.c = hl > 0xFFFF - val;
		self.regs_hl.set(result);
	}

	fn execute_opcode(&mut self, inter: &mut Interconnect, opcode: u8) -> usize {
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
			// JR cc, r8 - consolidated using Condition
			Opcode::Jr_Nz_r8 | Opcode::Jr_Z_r8 | Opcode::Jr_Nc_r8 | Opcode::Jr_C_r8 => {
				let cond = Condition::from_bits((opcode >> 3) & 0x03);
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
			// RET cc - consolidated using Condition
			Opcode::Ret_Nz | Opcode::Ret_Z | Opcode::Ret_Nc | Opcode::Ret_C => {
				let cond = Condition::from_bits((opcode >> 3) & 0x03);
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
			// JP cc, a16 - consolidated using Condition
			Opcode::Jp_Nz_a16 | Opcode::Jp_Z_a16 | Opcode::Jp_Nc_a16 | Opcode::Jp_C_a16 => {
				let cond = Condition::from_bits((opcode >> 3) & 0x03);
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
			// CALL cc, a16 - consolidated using Condition
			Opcode::Call_Nz_a16 | Opcode::Call_Z_a16 | Opcode::Call_Nc_a16 | Opcode::Call_C_a16 => {
				let cond = Condition::from_bits((opcode >> 3) & 0x03);
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

#[cfg(test)]
mod tests {
	use super::*;

	fn create_test_interconnect() -> Interconnect {
		// Create a minimal ROM for testing
		let mut rom = vec![0; 0x8000];
		// Add some test data
		rom[0x0100] = 0x00; // NOP
		Interconnect::new(rom)
	}

	/// Create interconnect with specific ROM data at given addresses
	fn create_test_interconnect_with_rom(data: &[(usize, u8)]) -> Interconnect {
		let mut rom = vec![0; 0x8000];
		for &(addr, val) in data {
			if addr < rom.len() {
				rom[addr] = val;
			}
		}
		Interconnect::new(rom)
	}

	#[test]
	fn test_cpu_initialization() {
		let cpu = Cpu::new();
		// Test initial register values match Game Boy hardware
		assert_eq!(cpu.reg_pc, 0x0100);
		assert_eq!(cpu.reg_sp, 0xFFFE);
		assert_eq!(cpu.regs_af.get(), 0x01B0);
		assert_eq!(cpu.regs_bc.get(), 0x0013);
		assert_eq!(cpu.regs_de.get(), 0x00D8);
		assert_eq!(cpu.regs_hl.get(), 0x014D);
	}

	#[test]
	fn test_nop_opcode() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();

		// Execute NOP (0x00)
		let cycles = cpu.execute_opcode(&mut inter, 0x00);

		assert_eq!(cycles, 4);
		// NOP shouldn't change any registers
	}

	#[test]
	fn test_dec_bc_opcode() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();

		let initial_bc = cpu.regs_bc.get();

		// Execute DEC BC (0x0B)
		let cycles = cpu.execute_opcode(&mut inter, 0x0B);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.regs_bc.get(), initial_bc.wrapping_sub(1));
	}

	#[test]
	fn test_cpl_opcode() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();

		// Set A register to a known value
		cpu.set_reg_a(0b10101010);

		// Execute CPL (0x2F) - complement A register
		let cycles = cpu.execute_opcode(&mut inter, 0x2F);

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0b01010101);
		assert!(cpu.flags.n); // N flag should be set
		assert!(cpu.flags.h); // H flag should be set
	}

	#[test]
	fn test_ld_c_b_opcode() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();

		// Set B register to a known value
		cpu.set_reg_b(0x42);

		// Execute LD C, B (0x48)
		let cycles = cpu.execute_opcode(&mut inter, 0x48);

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_c(), 0x42);
		assert_eq!(cpu.get_reg_b(), 0x42); // B should remain unchanged
	}

	#[test]
	fn test_ld_c_d_opcode() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();

		// Set D register to a known value
		cpu.set_reg_d(0x37);

		// Execute LD C, D (0x4A)
		let cycles = cpu.execute_opcode(&mut inter, 0x4A);

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_c(), 0x37);
	}

	#[test]
	fn test_register_getters_setters() {
		let mut cpu = Cpu::new();

		// Test A register
		cpu.set_reg_a(0xFF);
		assert_eq!(cpu.get_reg_a(), 0xFF);

		// Test B register
		cpu.set_reg_b(0xAB);
		assert_eq!(cpu.get_reg_b(), 0xAB);

		// Test D register
		cpu.set_reg_d(0xCD);
		assert_eq!(cpu.get_reg_d(), 0xCD);
	}

	#[test]
	fn test_flag_setters() {
		let mut cpu = Cpu::new();

		cpu.set_zero_flag(true);
		assert!(cpu.flags.z);

		cpu.set_subtract_flag(true);
		assert!(cpu.flags.n);

		cpu.set_half_carry_flag(true);
		assert!(cpu.flags.h);

		cpu.set_carry_flag(true);
		assert!(cpu.flags.c);
	}

	// ========== 16-bit Load Tests ==========

	#[test]
	fn test_ld_bc_d16() {
		let mut cpu = Cpu::new();
		// Place immediate value 0x1234 at PC (little-endian: 0x34, 0x12)
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0x34),
			(0x0101, 0x12),
		]);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0x01); // LD BC,d16

		assert_eq!(cycles, 12);
		assert_eq!(cpu.regs_bc.get(), 0x1234);
		assert_eq!(cpu.reg_pc, 0x0102); // PC advanced by 2
	}

	#[test]
	fn test_ld_de_d16() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0xCD),
			(0x0101, 0xAB),
		]);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0x11); // LD DE,d16

		assert_eq!(cycles, 12);
		assert_eq!(cpu.regs_de.get(), 0xABCD);
	}

	#[test]
	fn test_ld_hl_d16() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0xFF),
			(0x0101, 0xDF),
		]);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0x21); // LD HL,d16

		assert_eq!(cycles, 12);
		assert_eq!(cpu.regs_hl.get(), 0xDFFF);
	}

	#[test]
	fn test_ld_sp_d16() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0x00),
			(0x0101, 0xC0),
		]);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0x31); // LD SP,d16

		assert_eq!(cycles, 12);
		assert_eq!(cpu.reg_sp, 0xC000);
	}

	// ========== 8-bit Immediate Load Tests ==========

	#[test]
	fn test_ld_b_d8() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x42)]);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0x06); // LD B,d8

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_b(), 0x42);
	}

	#[test]
	fn test_ld_a_d8() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0xFF)]);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0x3E); // LD A,d8

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_a(), 0xFF);
	}

	// ========== Memory Load Tests ==========

	#[test]
	fn test_ld_bcref_a() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x55);
		cpu.regs_bc.set(0xC000); // Point to WRAM

		let cycles = cpu.execute_opcode(&mut inter, 0x02); // LD (BC),A

		assert_eq!(cycles, 8);
		assert_eq!(inter.read(0xC000), 0x55);
	}

	#[test]
	fn test_ld_a_bcref() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		inter.write(0xC000, 0xAA);
		cpu.regs_bc.set(0xC000);

		let cycles = cpu.execute_opcode(&mut inter, 0x0A); // LD A,(BC)

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_a(), 0xAA);
	}

	#[test]
	fn test_ld_hli_a() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x77);
		cpu.regs_hl.set(0xC000);

		let cycles = cpu.execute_opcode(&mut inter, 0x22); // LD (HL+),A

		assert_eq!(cycles, 8);
		assert_eq!(inter.read(0xC000), 0x77);
		assert_eq!(cpu.regs_hl.get(), 0xC001); // HL incremented
	}

	#[test]
	fn test_ld_hld_a() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x88);
		cpu.regs_hl.set(0xC010);

		let cycles = cpu.execute_opcode(&mut inter, 0x32); // LD (HL-),A

		assert_eq!(cycles, 8);
		assert_eq!(inter.read(0xC010), 0x88);
		assert_eq!(cpu.regs_hl.get(), 0xC00F); // HL decremented
	}

	#[test]
	fn test_ld_a_hli() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		inter.write(0xC000, 0x99);
		cpu.regs_hl.set(0xC000);

		let cycles = cpu.execute_opcode(&mut inter, 0x2A); // LD A,(HL+)

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_a(), 0x99);
		assert_eq!(cpu.regs_hl.get(), 0xC001);
	}

	#[test]
	fn test_ldh_a8_a() {
		let mut cpu = Cpu::new();
		// Use 0xFF01 (serial data) instead of 0xFF44 (LY) which is PPU register
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x01)]); // offset
		cpu.set_reg_a(0x42);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0xE0); // LDH (a8),A

		assert_eq!(cycles, 12);
		assert_eq!(inter.read(0xFF01), 0x42);
	}

	#[test]
	fn test_ldh_a_a8() {
		let mut cpu = Cpu::new();
		// Use 0xFF01 (serial data) instead of 0xFF44 (LY) which is PPU register
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x01)]); // offset
		inter.write(0xFF01, 0x90);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0xF0); // LDH A,(a8)

		assert_eq!(cycles, 12);
		assert_eq!(cpu.get_reg_a(), 0x90);
	}

	// ========== Jump Tests ==========

	#[test]
	fn test_jp_a16() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0x50),
			(0x0101, 0x01),
		]);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0xC3); // JP a16

		assert_eq!(cycles, 16);
		assert_eq!(cpu.reg_pc, 0x0150);
	}

	#[test]
	fn test_jp_hl() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.regs_hl.set(0x0200);

		let cycles = cpu.execute_opcode(&mut inter, 0xE9); // JP HL

		assert_eq!(cycles, 4);
		assert_eq!(cpu.reg_pc, 0x0200);
	}

	#[test]
	fn test_jr_r8_forward() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x10)]); // offset +16
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0x18); // JR r8

		assert_eq!(cycles, 12);
		assert_eq!(cpu.reg_pc, 0x0111); // 0x0101 + 0x10
	}

	#[test]
	fn test_jr_r8_backward() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0xFE)]); // offset -2 (signed)
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0x18); // JR r8

		assert_eq!(cycles, 12);
		assert_eq!(cpu.reg_pc, 0x00FF); // 0x0101 + (-2)
	}

	#[test]
	fn test_jr_nz_taken() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x05)]);
		cpu.reg_pc = 0x0100;
		cpu.flags.z = false;

		let cycles = cpu.execute_opcode(&mut inter, 0x20); // JR NZ,r8

		assert_eq!(cycles, 12);
		assert_eq!(cpu.reg_pc, 0x0106);
	}

	#[test]
	fn test_jr_nz_not_taken() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x05)]);
		cpu.reg_pc = 0x0100;
		cpu.flags.z = true;

		let cycles = cpu.execute_opcode(&mut inter, 0x20); // JR NZ,r8

		assert_eq!(cycles, 8);
		assert_eq!(cpu.reg_pc, 0x0101); // Only advanced past immediate
	}

	#[test]
	fn test_jp_z_taken() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0x00),
			(0x0101, 0x02),
		]);
		cpu.reg_pc = 0x0100;
		cpu.flags.z = true;

		let cycles = cpu.execute_opcode(&mut inter, 0xCA); // JP Z,a16

		assert_eq!(cycles, 16);
		assert_eq!(cpu.reg_pc, 0x0200);
	}

	#[test]
	fn test_jp_z_not_taken() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0x00),
			(0x0101, 0x02),
		]);
		cpu.reg_pc = 0x0100;
		cpu.flags.z = false;

		let cycles = cpu.execute_opcode(&mut inter, 0xCA); // JP Z,a16

		assert_eq!(cycles, 12);
		assert_eq!(cpu.reg_pc, 0x0102); // Skipped the jump
	}

	// ========== Call/Return Tests ==========

	#[test]
	fn test_call_a16() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0x00),
			(0x0101, 0x02),
		]);
		cpu.reg_pc = 0x0100;
		cpu.reg_sp = 0xFFFE;

		let cycles = cpu.execute_opcode(&mut inter, 0xCD); // CALL a16

		assert_eq!(cycles, 24);
		assert_eq!(cpu.reg_pc, 0x0200);
		assert_eq!(cpu.reg_sp, 0xFFFC);
		// Return address should be on stack
		assert_eq!(inter.read(0xFFFC), 0x02); // Low byte
		assert_eq!(inter.read(0xFFFD), 0x01); // High byte
	}

	#[test]
	fn test_ret() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.reg_sp = 0xFFFC;
		inter.write(0xFFFC, 0x50); // Return address low (HRAM is writable)
		inter.write(0xFFFD, 0x01); // Return address high

		let cycles = cpu.execute_opcode(&mut inter, 0xC9); // RET

		assert_eq!(cycles, 16);
		assert_eq!(cpu.reg_pc, 0x0150);
		assert_eq!(cpu.reg_sp, 0xFFFE);
	}

	#[test]
	fn test_call_and_ret() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0x00),
			(0x0101, 0x02),
		]);
		cpu.reg_pc = 0x0100;
		cpu.reg_sp = 0xFFFE;

		// Execute CALL
		cpu.execute_opcode(&mut inter, 0xCD);
		assert_eq!(cpu.reg_pc, 0x0200);

		// Execute RET
		cpu.execute_opcode(&mut inter, 0xC9);
		assert_eq!(cpu.reg_pc, 0x0102); // Return to after CALL
	}

	#[test]
	fn test_rst_00() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.reg_pc = 0x0150;
		cpu.reg_sp = 0xFFFE;

		let cycles = cpu.execute_opcode(&mut inter, 0xC7); // RST 00H

		assert_eq!(cycles, 16);
		assert_eq!(cpu.reg_pc, 0x0000);
		assert_eq!(cpu.reg_sp, 0xFFFC);
	}

	#[test]
	fn test_rst_38() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.reg_pc = 0x0150;
		cpu.reg_sp = 0xFFFE;

		let cycles = cpu.execute_opcode(&mut inter, 0xFF); // RST 38H

		assert_eq!(cycles, 16);
		assert_eq!(cpu.reg_pc, 0x0038);
	}

	// ========== Stack Tests ==========

	#[test]
	fn test_push_bc() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.regs_bc.set(0x1234);
		cpu.reg_sp = 0xFFFE;

		let cycles = cpu.execute_opcode(&mut inter, 0xC5); // PUSH BC

		assert_eq!(cycles, 16);
		assert_eq!(cpu.reg_sp, 0xFFFC);
		assert_eq!(inter.read(0xFFFC), 0x34); // Low byte
		assert_eq!(inter.read(0xFFFD), 0x12); // High byte
	}

	#[test]
	fn test_pop_bc() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		inter.write(0xFFFC, 0xCD);
		inter.write(0xFFFD, 0xAB);
		cpu.reg_sp = 0xFFFC;

		let cycles = cpu.execute_opcode(&mut inter, 0xC1); // POP BC

		assert_eq!(cycles, 12);
		assert_eq!(cpu.regs_bc.get(), 0xABCD);
		assert_eq!(cpu.reg_sp, 0xFFFE);
	}

	#[test]
	fn test_push_pop_af() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x12);
		cpu.flags.z = true;
		cpu.flags.n = false;
		cpu.flags.h = true;
		cpu.flags.c = true;
		cpu.reg_sp = 0xFFFE;

		// PUSH AF
		cpu.execute_opcode(&mut inter, 0xF5);
		assert_eq!(cpu.reg_sp, 0xFFFC);

		// Modify registers
		cpu.set_reg_a(0x00);
		cpu.flags.z = false;
		cpu.flags.h = false;
		cpu.flags.c = false;

		// POP AF - should restore
		cpu.execute_opcode(&mut inter, 0xF1);
		assert_eq!(cpu.get_reg_a(), 0x12);
		assert!(cpu.flags.z);
		assert!(!cpu.flags.n);
		assert!(cpu.flags.h);
		assert!(cpu.flags.c);
	}

	#[test]
	fn test_pop_af_lower_bits_zero() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		// Set F with lower 4 bits set (should be masked)
		inter.write(0xFFFC, 0xFF); // F = 0xFF, but lower 4 bits ignored
		inter.write(0xFFFD, 0x12); // A = 0x12
		cpu.reg_sp = 0xFFFC;

		cpu.execute_opcode(&mut inter, 0xF1); // POP AF

		// Lower 4 bits should be 0
		assert_eq!(cpu.regs_af.get() & 0x000F, 0);
	}

	// ========== ADD Tests ==========

	#[test]
	fn test_add_a_b() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x10);
		cpu.set_reg_b(0x20);

		let cycles = cpu.execute_opcode(&mut inter, 0x80); // ADD A,B

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0x30);
		assert!(!cpu.flags.z);
		assert!(!cpu.flags.n);
		assert!(!cpu.flags.h);
		assert!(!cpu.flags.c);
	}

	#[test]
	fn test_add_a_zero_flag() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x00);
		cpu.set_reg_b(0x00);

		cpu.execute_opcode(&mut inter, 0x80); // ADD A,B

		assert_eq!(cpu.get_reg_a(), 0x00);
		assert!(cpu.flags.z);
	}

	#[test]
	fn test_add_a_half_carry() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x0F);
		cpu.set_reg_b(0x01);

		cpu.execute_opcode(&mut inter, 0x80); // ADD A,B

		assert_eq!(cpu.get_reg_a(), 0x10);
		assert!(cpu.flags.h); // Half carry from bit 3 to 4
	}

	#[test]
	fn test_add_a_carry() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0xFF);
		cpu.set_reg_b(0x01);

		cpu.execute_opcode(&mut inter, 0x80); // ADD A,B

		assert_eq!(cpu.get_reg_a(), 0x00);
		assert!(cpu.flags.z);
		assert!(cpu.flags.c);
		assert!(cpu.flags.h);
	}

	#[test]
	fn test_add_a_d8() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x25)]);
		cpu.set_reg_a(0x10);
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0xC6); // ADD A,d8

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_a(), 0x35);
	}

	// ========== ADC Tests ==========

	#[test]
	fn test_adc_without_carry() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x10);
		cpu.set_reg_b(0x20);
		cpu.flags.c = false;

		cpu.execute_opcode(&mut inter, 0x88); // ADC A,B

		assert_eq!(cpu.get_reg_a(), 0x30);
	}

	#[test]
	fn test_adc_with_carry() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x10);
		cpu.set_reg_b(0x20);
		cpu.flags.c = true;

		cpu.execute_opcode(&mut inter, 0x88); // ADC A,B

		assert_eq!(cpu.get_reg_a(), 0x31); // 0x10 + 0x20 + 1
	}

	// ========== SUB Tests ==========

	#[test]
	fn test_sub_a_b() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x30);
		cpu.set_reg_b(0x10);

		let cycles = cpu.execute_opcode(&mut inter, 0x90); // SUB A,B

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0x20);
		assert!(!cpu.flags.z);
		assert!(cpu.flags.n); // N always set for SUB
		assert!(!cpu.flags.h);
		assert!(!cpu.flags.c);
	}

	#[test]
	fn test_sub_a_borrow() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x10);
		cpu.set_reg_b(0x20);

		cpu.execute_opcode(&mut inter, 0x90); // SUB A,B

		assert_eq!(cpu.get_reg_a(), 0xF0); // Wraps around
		assert!(cpu.flags.c); // Borrow occurred
	}

	#[test]
	fn test_sub_half_borrow() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x10);
		cpu.set_reg_b(0x01);

		cpu.execute_opcode(&mut inter, 0x90); // SUB A,B

		assert_eq!(cpu.get_reg_a(), 0x0F);
		assert!(cpu.flags.h); // Half borrow from bit 4 to 3
	}

	// ========== SBC Tests ==========

	#[test]
	fn test_sbc_without_carry() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x30);
		cpu.set_reg_b(0x10);
		cpu.flags.c = false;

		cpu.execute_opcode(&mut inter, 0x98); // SBC A,B

		assert_eq!(cpu.get_reg_a(), 0x20);
	}

	#[test]
	fn test_sbc_with_carry() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x30);
		cpu.set_reg_b(0x10);
		cpu.flags.c = true;

		cpu.execute_opcode(&mut inter, 0x98); // SBC A,B

		assert_eq!(cpu.get_reg_a(), 0x1F); // 0x30 - 0x10 - 1
	}

	// ========== AND Tests ==========

	#[test]
	fn test_and_a_b() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0b11110000);
		cpu.set_reg_b(0b10101010);

		let cycles = cpu.execute_opcode(&mut inter, 0xA0); // AND A,B

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0b10100000);
		assert!(!cpu.flags.z);
		assert!(!cpu.flags.n);
		assert!(cpu.flags.h); // H always set for AND
		assert!(!cpu.flags.c);
	}

	#[test]
	fn test_and_zero_result() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0b11110000);
		cpu.set_reg_b(0b00001111);

		cpu.execute_opcode(&mut inter, 0xA0); // AND A,B

		assert_eq!(cpu.get_reg_a(), 0x00);
		assert!(cpu.flags.z);
	}

	// ========== OR Tests ==========

	#[test]
	fn test_or_a_b() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0b11110000);
		cpu.set_reg_b(0b00001111);

		let cycles = cpu.execute_opcode(&mut inter, 0xB0); // OR A,B

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0xFF);
		assert!(!cpu.flags.z);
		assert!(!cpu.flags.n);
		assert!(!cpu.flags.h);
		assert!(!cpu.flags.c);
	}

	// ========== XOR Tests ==========

	#[test]
	fn test_xor_a_a() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0xFF);

		let cycles = cpu.execute_opcode(&mut inter, 0xAF); // XOR A,A

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0x00);
		assert!(cpu.flags.z);
		assert!(!cpu.flags.n);
		assert!(!cpu.flags.h);
		assert!(!cpu.flags.c);
	}

	#[test]
	fn test_xor_a_b() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0b11110000);
		cpu.set_reg_b(0b10101010);

		cpu.execute_opcode(&mut inter, 0xA8); // XOR A,B

		assert_eq!(cpu.get_reg_a(), 0b01011010);
	}

	// ========== CP Tests ==========

	#[test]
	fn test_cp_equal() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x42);
		cpu.set_reg_b(0x42);

		let cycles = cpu.execute_opcode(&mut inter, 0xB8); // CP A,B

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0x42); // A unchanged
		assert!(cpu.flags.z); // Equal
		assert!(cpu.flags.n);
		assert!(!cpu.flags.c);
	}

	#[test]
	fn test_cp_less_than() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x10);
		cpu.set_reg_b(0x20);

		cpu.execute_opcode(&mut inter, 0xB8); // CP A,B

		assert_eq!(cpu.get_reg_a(), 0x10); // A unchanged
		assert!(!cpu.flags.z);
		assert!(cpu.flags.c); // Borrow occurred (A < B)
	}

	#[test]
	fn test_cp_greater_than() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0x30);
		cpu.set_reg_b(0x10);

		cpu.execute_opcode(&mut inter, 0xB8); // CP A,B

		assert!(!cpu.flags.z);
		assert!(!cpu.flags.c);
	}

	// ========== INC/DEC Tests ==========

	#[test]
	fn test_inc_b() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_b(0x0F);

		let cycles = cpu.execute_opcode(&mut inter, 0x04); // INC B

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_b(), 0x10);
		assert!(!cpu.flags.z);
		assert!(!cpu.flags.n);
		assert!(cpu.flags.h); // Half carry from 0x0F to 0x10
	}

	#[test]
	fn test_inc_overflow() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_b(0xFF);

		cpu.execute_opcode(&mut inter, 0x04); // INC B

		assert_eq!(cpu.get_reg_b(), 0x00);
		assert!(cpu.flags.z);
		assert!(cpu.flags.h);
	}

	#[test]
	fn test_dec_b() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_b(0x10);

		let cycles = cpu.execute_opcode(&mut inter, 0x05); // DEC B

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_b(), 0x0F);
		assert!(!cpu.flags.z);
		assert!(cpu.flags.n);
		assert!(cpu.flags.h); // Half borrow from 0x10 to 0x0F
	}

	#[test]
	fn test_dec_to_zero() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_b(0x01);

		cpu.execute_opcode(&mut inter, 0x05); // DEC B

		assert_eq!(cpu.get_reg_b(), 0x00);
		assert!(cpu.flags.z);
	}

	#[test]
	fn test_inc_bc() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.regs_bc.set(0x00FF);

		let cycles = cpu.execute_opcode(&mut inter, 0x03); // INC BC

		assert_eq!(cycles, 8);
		assert_eq!(cpu.regs_bc.get(), 0x0100);
		// 16-bit INC doesn't affect flags
	}

	#[test]
	fn test_inc_hl_ref() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.regs_hl.set(0xC000);
		inter.write(0xC000, 0x0F);

		let cycles = cpu.execute_opcode(&mut inter, 0x34); // INC (HL)

		assert_eq!(cycles, 12);
		assert_eq!(inter.read(0xC000), 0x10);
		assert!(cpu.flags.h);
	}

	// ========== Rotate Tests ==========

	#[test]
	fn test_rlca() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0b10000001);

		let cycles = cpu.execute_opcode(&mut inter, 0x07); // RLCA

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0b00000011);
		assert!(!cpu.flags.z); // Z always 0 for RLCA
		assert!(!cpu.flags.n);
		assert!(!cpu.flags.h);
		assert!(cpu.flags.c); // Bit 7 was 1
	}

	#[test]
	fn test_rrca() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0b00000011);

		let cycles = cpu.execute_opcode(&mut inter, 0x0F); // RRCA

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0b10000001);
		assert!(cpu.flags.c); // Bit 0 was 1
	}

	#[test]
	fn test_rla() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0b10000000);
		cpu.flags.c = true;

		let cycles = cpu.execute_opcode(&mut inter, 0x17); // RLA

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0b00000001); // Carry rotated in
		assert!(cpu.flags.c); // Old bit 7
	}

	#[test]
	fn test_rra() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.set_reg_a(0b00000001);
		cpu.flags.c = true;

		let cycles = cpu.execute_opcode(&mut inter, 0x1F); // RRA

		assert_eq!(cycles, 4);
		assert_eq!(cpu.get_reg_a(), 0b10000000); // Carry rotated in
		assert!(cpu.flags.c); // Old bit 0
	}

	// ========== Misc Tests ==========

	#[test]
	fn test_scf() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.flags.c = false;
		cpu.flags.n = true;
		cpu.flags.h = true;

		let cycles = cpu.execute_opcode(&mut inter, 0x37); // SCF

		assert_eq!(cycles, 4);
		assert!(cpu.flags.c);
		assert!(!cpu.flags.n);
		assert!(!cpu.flags.h);
	}

	#[test]
	fn test_ccf() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.flags.c = true;

		cpu.execute_opcode(&mut inter, 0x3F); // CCF

		assert!(!cpu.flags.c);

		cpu.execute_opcode(&mut inter, 0x3F); // CCF again

		assert!(cpu.flags.c);
	}

	#[test]
	fn test_ld_sp_hl() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.regs_hl.set(0xDFFF);

		let cycles = cpu.execute_opcode(&mut inter, 0xF9); // LD SP,HL

		assert_eq!(cycles, 8);
		assert_eq!(cpu.reg_sp, 0xDFFF);
	}

	#[test]
	fn test_add_hl_bc() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.regs_hl.set(0x0FFF);
		cpu.regs_bc.set(0x0001);

		let cycles = cpu.execute_opcode(&mut inter, 0x09); // ADD HL,BC

		assert_eq!(cycles, 8);
		assert_eq!(cpu.regs_hl.get(), 0x1000);
		assert!(!cpu.flags.n);
		assert!(cpu.flags.h); // Half carry from bit 11 to 12
		assert!(!cpu.flags.c);
	}

	#[test]
	fn test_add_hl_bc_carry() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect();
		cpu.regs_hl.set(0xFFFF);
		cpu.regs_bc.set(0x0001);

		cpu.execute_opcode(&mut inter, 0x09); // ADD HL,BC

		assert_eq!(cpu.regs_hl.get(), 0x0000);
		assert!(cpu.flags.c);
	}

	#[test]
	fn test_ld_a16_sp() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[
			(0x0100, 0x00),
			(0x0101, 0xC0),
		]);
		cpu.reg_sp = 0x1234;
		cpu.reg_pc = 0x0100;

		let cycles = cpu.execute_opcode(&mut inter, 0x08); // LD (a16),SP

		assert_eq!(cycles, 20);
		assert_eq!(inter.read(0xC000), 0x34); // Low byte
		assert_eq!(inter.read(0xC001), 0x12); // High byte
	}

	// ========== CB-Prefix Opcode Tests ==========

	#[test]
	fn test_cb_rlc_b() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x00)]); // CB opcode for RLC B
		cpu.reg_pc = 0x0100;
		cpu.set_reg_b(0x85); // 1000_0101

		let cycles = cpu.execute_opcode(&mut inter, 0xCB); // PREFIX CB

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_b(), 0x0B); // 0000_1011 (rotated left, bit 7 -> bit 0)
		assert!(cpu.flags.c); // Old bit 7 was 1
		assert!(!cpu.flags.z);
		assert!(!cpu.flags.n);
		assert!(!cpu.flags.h);
	}

	#[test]
	fn test_cb_rlc_zero() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x00)]); // RLC B
		cpu.reg_pc = 0x0100;
		cpu.set_reg_b(0x00);

		cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cpu.get_reg_b(), 0x00);
		assert!(cpu.flags.z);
		assert!(!cpu.flags.c);
	}

	#[test]
	fn test_cb_rrc_a() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x0F)]); // RRC A
		cpu.reg_pc = 0x0100;
		cpu.set_reg_a(0x81); // 1000_0001

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_a(), 0xC0); // 1100_0000 (rotated right, bit 0 -> bit 7)
		assert!(cpu.flags.c); // Old bit 0 was 1
	}

	#[test]
	fn test_cb_rl_c() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x11)]); // RL C
		cpu.reg_pc = 0x0100;
		cpu.set_reg_c(0x80); // 1000_0000
		cpu.flags.c = true; // Carry set

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_c(), 0x01); // 0000_0001 (shifted left, old carry -> bit 0)
		assert!(cpu.flags.c); // Old bit 7 was 1
	}

	#[test]
	fn test_cb_rr_d() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x1A)]); // RR D
		cpu.reg_pc = 0x0100;
		cpu.set_reg_d(0x01); // 0000_0001
		cpu.flags.c = true; // Carry set

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_d(), 0x80); // 1000_0000 (shifted right, old carry -> bit 7)
		assert!(cpu.flags.c); // Old bit 0 was 1
	}

	#[test]
	fn test_cb_sla_e() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x23)]); // SLA E
		cpu.reg_pc = 0x0100;
		cpu.set_reg_e(0xC1); // 1100_0001

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_e(), 0x82); // 1000_0010 (shifted left, bit 0 = 0)
		assert!(cpu.flags.c); // Old bit 7 was 1
	}

	#[test]
	fn test_cb_sra_h() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x2C)]); // SRA H
		cpu.reg_pc = 0x0100;
		cpu.set_reg_h(0x81); // 1000_0001

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_h(), 0xC0); // 1100_0000 (shifted right, bit 7 preserved)
		assert!(cpu.flags.c); // Old bit 0 was 1
	}

	#[test]
	fn test_cb_swap_l() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x35)]); // SWAP L
		cpu.reg_pc = 0x0100;
		cpu.set_reg_l(0x12); // 0001_0010

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_l(), 0x21); // 0010_0001 (nibbles swapped)
		assert!(!cpu.flags.c);
		assert!(!cpu.flags.z);
	}

	#[test]
	fn test_cb_swap_zero() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x37)]); // SWAP A
		cpu.reg_pc = 0x0100;
		cpu.set_reg_a(0x00);

		cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cpu.get_reg_a(), 0x00);
		assert!(cpu.flags.z);
	}

	#[test]
	fn test_cb_srl_a() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x3F)]); // SRL A
		cpu.reg_pc = 0x0100;
		cpu.set_reg_a(0x81); // 1000_0001

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_a(), 0x40); // 0100_0000 (shifted right, bit 7 = 0)
		assert!(cpu.flags.c); // Old bit 0 was 1
	}

	#[test]
	fn test_cb_bit_0_b() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x40)]); // BIT 0,B
		cpu.reg_pc = 0x0100;
		cpu.set_reg_b(0xFE); // 1111_1110 (bit 0 is 0)
		cpu.flags.c = true; // Should be unchanged

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert!(cpu.flags.z); // Bit 0 is 0, so Z is set
		assert!(!cpu.flags.n);
		assert!(cpu.flags.h);
		assert!(cpu.flags.c); // Unchanged
	}

	#[test]
	fn test_cb_bit_7_a() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x7F)]); // BIT 7,A
		cpu.reg_pc = 0x0100;
		cpu.set_reg_a(0x80); // 1000_0000 (bit 7 is 1)

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert!(!cpu.flags.z); // Bit 7 is 1, so Z is clear
	}

	#[test]
	fn test_cb_res_3_c() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x99)]); // RES 3,C
		cpu.reg_pc = 0x0100;
		cpu.set_reg_c(0xFF); // All bits set

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_c(), 0xF7); // 1111_0111 (bit 3 cleared)
	}

	#[test]
	fn test_cb_set_5_d() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0xEA)]); // SET 5,D
		cpu.reg_pc = 0x0100;
		cpu.set_reg_d(0x00); // All bits clear

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 8);
		assert_eq!(cpu.get_reg_d(), 0x20); // 0010_0000 (bit 5 set)
	}

	#[test]
	fn test_cb_rlc_hl_ref() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x06)]); // RLC (HL)
		cpu.reg_pc = 0x0100;
		cpu.regs_hl.set(0xC000); // Point to WRAM
		inter.write(0xC000, 0x85); // 1000_0101

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 16); // (HL) takes 16 cycles
		assert_eq!(inter.read(0xC000), 0x0B); // 0000_1011
		assert!(cpu.flags.c);
	}

	#[test]
	fn test_cb_bit_hl_ref() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0x46)]); // BIT 0,(HL)
		cpu.reg_pc = 0x0100;
		cpu.regs_hl.set(0xC000);
		inter.write(0xC000, 0x01); // Bit 0 is set

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 12); // BIT (HL) takes 12 cycles
		assert!(!cpu.flags.z); // Bit 0 is 1
	}

	#[test]
	fn test_cb_set_hl_ref() {
		let mut cpu = Cpu::new();
		let mut inter = create_test_interconnect_with_rom(&[(0x0100, 0xFE)]); // SET 7,(HL)
		cpu.reg_pc = 0x0100;
		cpu.regs_hl.set(0xC000);
		inter.write(0xC000, 0x00);

		let cycles = cpu.execute_opcode(&mut inter, 0xCB);

		assert_eq!(cycles, 16);
		assert_eq!(inter.read(0xC000), 0x80); // Bit 7 set
	}
}
