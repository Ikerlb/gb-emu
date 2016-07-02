use gb::cpu::*;

//TODO COMPLETE STRUCT
#[derive(Debug)]
pub struct GameBoy{
	cpu: Cpu,
	cart_rom: Box<[u8]>,
}

impl GameBoy{

	pub fn new(ct_rom: Box<[u8]>)->Self{
		GameBoy{cpu: Cpu::new(),cart_rom: ct_rom}
	}

		

}