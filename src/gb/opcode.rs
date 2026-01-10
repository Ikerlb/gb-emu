use num_derive::{FromPrimitive, ToPrimitive};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum Opcode {
	////// 0x0X
	Nop = 0x00,
	Ld_Bc_d16 = 0x01,
	Dec_Bc = 0x0B,
	////// 0x1X
	////// 0x2X
	Dec_H = 0x25,
	Cpl = 0x2F,
	////// 0x3X
	//////0x4X
	Ld_C_B = 0x48,
	Ld_C_C = 0x49,
	Ld_C_D = 0x4A,
	//////
	Jp_a16 = 0xC3,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            Opcode::Nop => "NOP",
            Opcode::Ld_Bc_d16 => "LD BC,d16",
            Opcode::Dec_Bc => "DEC BC",
            Opcode::Dec_H => "DEC H",
            Opcode::Cpl => "CPL",
            Opcode::Ld_C_B => "LD C,B",
            Opcode::Ld_C_C => "LD C,C",
            Opcode::Ld_C_D => "LD C,D",
            Opcode::Jp_a16 => "JP a16",
        };
        write!(f, "{}", name)
    }
}
