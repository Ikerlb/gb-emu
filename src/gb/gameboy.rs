use gb::cpu::*;
use gb::interconnect::*;
use std::fmt::{Display,Result,Formatter};

pub struct GameBoy{
	cpu: Cpu,
	interconnect: Interconnect,
}

impl GameBoy{

	pub fn new(ct_rom: Box<[u8]>)->Self{
		GameBoy{cpu: Cpu::new(),interconnect: Interconnect::new(ct_rom)}
	}

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn interconnect(&self) -> &Interconnect {
        &self.interconnect
    }

}

//Implementing display trait for debugging purposes.
impl Display for GameBoy {
    fn fmt(&self, f: &mut Formatter) -> Result {
    	write!(f,"{:#?}",self.cpu)
    }
}