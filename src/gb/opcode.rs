use num_derive::{FromPrimitive, ToPrimitive};

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
