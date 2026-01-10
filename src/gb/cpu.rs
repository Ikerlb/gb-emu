use crate::gb::interconnect::*;
use crate::gb::opcode::Opcode;
use num_traits::FromPrimitive;
use crate::gb::register::Register;
use std::fmt::{Display, Formatter, Result};

//FINISH TESTS!!!

#[derive(Debug)]
pub struct Cpu{
	//Program counter
	reg_pc: u16,
	//Stack pointer
	reg_sp: Register,

	regs_af: Register, //AF Register
	regs_bc: Register, //BC Register
	regs_de: Register, //DE Register
	regs_hl: Register, //HL Register

	flags: Flags,

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
        write!(f, "PC:{:04X} SP:{} AF:{} BC:{} DE:{} HL:{} [{}]",
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

//Get rid of all the sets? Good or bad practice? DOIT!
impl Cpu{

	/// Get the program counter value
	pub fn pc(&self) -> u16 {
		self.reg_pc
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
			self.reg_sp.get(),
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

	//set zero flag
	fn set_zero_flag(&mut self,bit: bool){
		self.flags.z=bit;
	}
	//set subtract flag
	fn set_subtract_flag(&mut self,bit: bool){
		self.flags.n=bit;
	}
	//set half carry flag
	fn set_half_carry_flag(&mut self,bit: bool){
		self.flags.h=bit;
	}
	//set carry flag
	fn set_carry_flag(&mut self,bit: bool){
		self.flags.c=bit;
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

	//gets
	fn get_reg_a(&mut self)->u8{
		self.regs_af.get_hi()
	}

	fn get_reg_b(&mut self)->u8{
		self.regs_bc.get_hi()
	}

	fn get_reg_c(&mut self)->u8{
		self.regs_bc.get_lo()
	}

	fn get_reg_d(&mut self)->u8{
		self.regs_de.get_hi()
	}

	fn get_reg_e(&mut self)->u8{
		self.regs_de.get_lo()
	}

	fn get_reg_h(&mut self)->u8{
		self.regs_hl.get_hi()
	}

	fn get_reg_l(&mut self)->u8{
		self.regs_hl.get_lo()
	}

	//initial state taken from codeslinger (as did almost everything that sounds tricky)
	pub fn new() -> Self{
		Cpu{
			reg_pc: 0x0100,
			reg_sp: Register::new(0xFFFE),
			regs_af: Register::new(0x01B0),
			regs_bc: Register::new(0x0013),
			regs_de: Register::new(0x00D8),
			regs_hl: Register::new(0x014D),
			flags: Flags{z:false,n:false,
							h:false,c:false}
		}
	}

	pub fn execute_next_opcode(&mut self,inter:&mut Interconnect)->usize{
		let op = inter.read(self.reg_pc);
		//wrapping add to prevent overflow
		self.reg_pc=self.reg_pc.wrapping_add(1);
		self.execute_opcode(inter,op)
	}

	//TODO move instructions to separate functions? Will it look better?
	fn execute_opcode(&mut self,inter:&mut Interconnect,opcode: u8) -> usize{
		//unwraps opcode and panics if none
		let value=Opcode::from_u8(opcode).unwrap_or_else(||
            panic!("Unrecognized Opcode: {:#X})",opcode)
        );
        //TODO: Look for a way to remove the Opcode::opcode
		match value{
			//0x00
			Opcode::Nop     => 4,
			//0x0B
			Opcode::Dec_Bc  => {
				////////////////////////////// !!!!!Unwrapped sub???! 
				let num=self.regs_bc.get().wrapping_sub(1);
				self.regs_bc.set(num);
				8
			},
			//TODO:0x25
			Opcode::Dec_H   => {
				4
			},
			//0x2F
			Opcode::Cpl     => {
				let num=self.get_reg_a();
				self.set_reg_a(!num);
				self.set_subtract_flag(true);
				self.set_half_carry_flag(true);
				4
			},
			//0xC3
			Opcode::Jp_a16  => {
				self.reg_pc=inter.read_16bits(self.reg_pc);
				16
			},
			//0x48
			Opcode::Ld_C_B  => {
				let num=self.get_reg_b();
				self.set_reg_c(num);
				4
			},
			//0x49 Is it there for completeness?
			Opcode::Ld_C_C  => 4,
			//0x4A
			Opcode::Ld_C_D  => {
				let num=self.get_reg_d();
				self.set_reg_c(num);
				4
			}
			_ => 0,
		}
	}

	//fn dec
	/*fn get_reg_l(&mut self)->u8{
		self.regs_hl.get_lo()
	}*/
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::gb::cartridge::Cartridge;

	fn create_test_interconnect() -> Interconnect {
		// Create a minimal ROM for testing
		let mut rom = vec![0; 0x8000];
		// Add some test data
		rom[0x0100] = 0x00; // NOP
		Interconnect::new(rom)
	}

	#[test]
	fn test_cpu_initialization() {
		let cpu = Cpu::new();
		// Test initial register values match Game Boy hardware
		assert_eq!(cpu.reg_pc, 0x0100);
		assert_eq!(cpu.reg_sp.get(), 0xFFFE);
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
}
