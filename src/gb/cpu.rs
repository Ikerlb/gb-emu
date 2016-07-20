use gb::interconnect::*;
use gb::opcode::Opcode;
use num::FromPrimitive;
use gb::register::{Register,RegisterAF};

#[derive(Debug)]
pub struct Cpu{
	//Program counter
	reg_pc: u16,
	//Stack pointer
	reg_sp: Register,

	regs_af: RegisterAF, //AF Register
	regs_bc: Register, //BC Register
	regs_de: Register, //DE Register
	regs_hl: Register, //HL Register
}


impl Cpu{

	//TODO make the initial states a constant??
	//initial state taken from codeslinger
	pub fn new() -> Self{
		Cpu{
			reg_pc: 0x100,
			reg_sp: Register::new(0xFFFE),
			regs_af: RegisterAF::new(0x01B0),
			regs_bc: Register::new(0x0013),
			regs_de: Register::new(0x00D8),
			regs_hl: Register::new(0x014D)
		}
	}

	fn execute_next_instruction(&self,inter: &mut Interconnect)->usize{
		0
	}

	fn execute_instruction(&mut self, inter: &mut Interconnect ,opcode: u8) -> usize{
		//unwraps opcode and panics if none
		let value=Opcode::from_u8(opcode).unwrap_or_else(||
            panic!("Unrecognized op: {:#X})",opcode)
        );
        //TODO: Look for a way to remove the Opcode::opcode
		match value{
			//0x00
			Opcode::Nop => 4,
			_ => 0,
		}
	}
}
