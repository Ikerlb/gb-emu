enum_from_primitive!{
	#[derive(Debug)]
	pub enum Opcode{
		Nop = 0x00,
		LD_BC_d16 = 0x01,
	}
}
